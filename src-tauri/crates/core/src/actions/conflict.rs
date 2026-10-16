//! 합치기 마치기·그만두기(D-069, D-070). 충돌 파일을 덮어쓰기 전에 작업 폴더 내용을 백업한다.

use std::collections::HashSet;

use git::GitOp;

use super::{paths, repo_of};
use crate::conflict::{ConflictPath, Context, Pick, assemble, conflict_paths, load_file};
use crate::view::{ActionInput, ConflictFileKind, FileChoice, HunkChoice, SideState};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 입력 상한.
const MAX_FILES: usize = 1000;
const MAX_CUSTOM: usize = 1024 * 1024;
const MAX_TOTAL: usize = 8 * 1024 * 1024;

pub struct ConflictAction {
    meta: ActionMeta,
}

impl ConflictAction {
    pub fn finish() -> Self {
        Self {
            meta: ActionMeta {
                id: "conflict-finish",
                glossary_id: "conflict",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::RestoreTree,
            },
        }
    }

    pub fn abort() -> Self {
        Self {
            meta: ActionMeta {
                id: "conflict-abort",
                glossary_id: "merge",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

/// 다른 도구가 시작한 리베이스·cherry-pick·revert 중이면 앱이 끝내지 않는다.
fn refuse_other_operation(git: &dyn git::GitBackend, repo: &git::RepoPath) -> Result<()> {
    if crate::conflict::other_operation(&git.git_dir(repo)?).is_some() {
        return Err(CoreError::OperationInProgress);
    }
    Ok(())
}

/// 충돌 파일의 지금 작업 폴더 내용을 백업 ref로 남긴다(외부 편집기로 고친 것을 잃지 않게, D-070).
/// 못 남기면 덮어쓰지 않는다.
fn back_up_conflicts(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    conflicts: &[ConflictPath],
) -> Result<()> {
    if conflicts.is_empty() {
        return Ok(());
    }
    let out = git.run(
        repo,
        &GitOp::SnapshotFiles {
            paths: conflicts.iter().map(|c| c.path.clone()).collect(),
            message: "higit conflict files".into(),
        },
    )?;
    let oid = out
        .oid
        .ok_or(CoreError::InvalidInput("충돌 파일을 백업하지 못했어요"))?;
    let branch = git
        .current_branch(repo)?
        .unwrap_or_else(|| "HEAD".to_string());
    crate::undo::backup::backup_oid(git, repo, &oid, &format!("conflict-files-{branch}"))?;
    Ok(())
}

/// 합칠 쪽 이름표: 그 버전을 가리키는 로컬 브랜치, 없으면 원격 브랜치, 없으면 짧은 해시.
fn theirs_label(git: &dyn git::GitBackend, repo: &git::RepoPath) -> Result<String> {
    let Some(oid) = git.resolve_ref(repo, "MERGE_HEAD")? else {
        return Ok(String::new());
    };
    let branches = git.branches(repo)?;
    Ok(branches
        .iter()
        .filter(|b| b.oid == oid.as_str() && !b.is_head)
        .min_by_key(|b| b.kind != git::BranchKind::Local)
        .map(|b| b.name.clone())
        .unwrap_or_else(|| oid.as_str().chars().take(7).collect()))
}

impl Action for ConflictAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        self.meta.id == "conflict-finish"
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        refuse_other_operation(git, repo)?;
        let Some(context) = crate::conflict::context(git, repo, ctx.key)? else {
            return Err(CoreError::InvalidInput("합치는 중이 아니에요"));
        };
        let conflicts = conflict_paths(git.index_entries(repo)?, context.flipped());
        match (self.meta.id, input, context) {
            ("conflict-abort", ActionInput::ConflictAbort, Context::Merge) => {
                back_up_conflicts(git, repo, &conflicts)?;
                git.run(repo, &GitOp::MergeAbort)?;
                Ok(Outcome::ConflictAborted)
            }
            ("conflict-abort", ActionInput::ConflictAbort, Context::Unpark(marker)) => {
                back_up_conflicts(git, repo, &conflicts)?;
                let oid = git::Oid::parse(&marker.oid).ok_or(CoreError::ConflictChanged)?;
                // 맡긴 ref는 그대로 두고 꺼내기 전으로 돌린다.
                git.run(repo, &GitOp::UnparkAbort { oid })?;
                crate::parked::clear_marker(&git.git_dir(repo)?);
                Ok(Outcome::ConflictAborted)
            }
            ("conflict-finish", ActionInput::ConflictFinish(input), Context::Merge) => {
                write_resolutions(git, repo, &conflicts, input)?;
                commit_merge(git, repo, &input.locale)?;
                Ok(Outcome::ConflictFinished)
            }
            ("conflict-finish", ActionInput::ConflictFinish(input), Context::Unpark(marker)) => {
                write_resolutions(git, repo, &conflicts, input)?;
                finish_unpark(git, repo, &marker)?;
                Ok(Outcome::ParkedResolved)
            }
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}

/// 꺼내다 겹친 맡긴 파일을 마친다: 넣은 상태로 남은 것을 모두 고친 파일로 돌리고, 맡긴 ref는
/// 백업으로 옮긴 뒤 지우고, 표식을 지운다(D-083).
fn finish_unpark(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    marker: &crate::parked::UnparkMarker,
) -> Result<()> {
    use git::Change;
    let staged: Vec<String> = git
        .status(repo)?
        .entries
        .iter()
        .filter(|e| !matches!(e.index, Change::Unmodified | Change::Untracked))
        .map(|e| e.path.clone())
        .collect();
    if !staged.is_empty() {
        git.run(repo, &GitOp::Unstage { paths: staged })?;
    }
    let oid = git::Oid::parse(&marker.oid).ok_or(CoreError::ConflictChanged)?;
    let name = crate::parked::ref_name(&marker.branch);
    crate::undo::backup::backup_oid(git, repo, &oid, &format!("parked-{}", marker.branch))?;
    git.delete_ref(repo, &name, &git::RefExpect::Is(oid))?;
    crate::parked::clear_marker(&git.git_dir(repo)?);
    Ok(())
}

/// 합친 버전을 만든다. 저장이 hook에 막혀도 해결한 것은 남아 다시 마치면 저장만 한다.
fn commit_merge(git: &dyn git::GitBackend, repo: &git::RepoPath, locale: &str) -> Result<()> {
    let label = theirs_label(git, repo)?;
    let message = if locale.starts_with("ko") {
        format!("{label} 가져와 합침")
    } else {
        format!("Merge {label}")
    };
    git.run(repo, &GitOp::Commit { message })?;
    Ok(())
}

fn write_resolutions(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    conflicts: &[ConflictPath],
    input: &crate::view::ConflictFinishInput,
) -> Result<()> {
    // 입력 검사: 상한, 경로 형식, 같은 경로 두 번.
    if input.resolutions.len() > MAX_FILES {
        return Err(CoreError::InvalidInput("고른 파일이 너무 많아요"));
    }
    let mut total = 0usize;
    let mut seen = HashSet::new();
    for r in &input.resolutions {
        paths::validate_one(&r.path)?;
        if !seen.insert(r.path.as_str()) {
            return Err(CoreError::InvalidInput("같은 파일을 두 번 골랐어요"));
        }
        if let FileChoice::Hunks { choices } = &r.choice {
            for c in choices {
                if let HunkChoice::Custom { text } = c {
                    if text.len() > MAX_CUSTOM {
                        return Err(CoreError::InvalidInput("직접 고친 글이 너무 길어요"));
                    }
                    total += text.len();
                }
            }
        }
    }
    if total > MAX_TOTAL {
        return Err(CoreError::InvalidInput("직접 고친 글이 너무 길어요"));
    }
    // 남은 충돌 파일은 모두 꼭 한 번씩 골라야 한다. 이미 해결된 경로(앞선 마치기가 쓰다 멈춤)는
    // 입력에 있어도 건너뛴다. 충돌 목록에 없는 경로는 쓰지 않는다(화면이 아무 파일이나 못 쓴다).
    let choice_of = |path: &str| input.resolutions.iter().find(|r| r.path == path);
    if conflicts.iter().any(|c| choice_of(&c.path).is_none()) {
        return Err(CoreError::InvalidInput("아직 고르지 않은 곳이 있어요"));
    }
    // 쓰기 전에 모두 계산해 둔다. 하나라도 화면이 본 것과 다르면 아무것도 쓰지 않는다.
    let mut writes = Vec::new();
    for c in conflicts {
        let Some(r) = choice_of(&c.path) else {
            continue;
        };
        let (view, segments) = load_file(git, repo, c)?;
        let op = match (&r.choice, view.kind, segments) {
            (FileChoice::Hunks { choices }, ConflictFileKind::Text, Some(segments)) => {
                let picks: Vec<Pick<'_>> = choices
                    .iter()
                    .map(|h| match h {
                        HunkChoice::Ours => Pick::Ours,
                        HunkChoice::Theirs => Pick::Theirs,
                        HunkChoice::Both => Pick::Both,
                        HunkChoice::Custom { text } => Pick::Custom(text),
                    })
                    .collect();
                let bytes = assemble(&segments, &picks).ok_or(CoreError::ConflictChanged)?;
                let mode = c
                    .ours
                    .as_ref()
                    .map(|e| e.mode.clone())
                    .unwrap_or_else(|| "100644".into());
                GitOp::ResolveFile {
                    path: c.path.clone(),
                    mode,
                    bytes,
                }
            }
            (FileChoice::Hunks { .. }, _, _) => return Err(CoreError::ConflictChanged),
            (FileChoice::Ours, _, _) if view.ours_side == SideState::Deleted => {
                GitOp::ResolveDelete {
                    path: c.path.clone(),
                }
            }
            (FileChoice::Theirs, _, _) if view.theirs_side == SideState::Deleted => {
                GitOp::ResolveDelete {
                    path: c.path.clone(),
                }
            }
            (FileChoice::Ours, _, _) => GitOp::ResolveSide {
                path: c.path.clone(),
                ours: true,
            },
            (FileChoice::Theirs, _, _) => GitOp::ResolveSide {
                path: c.path.clone(),
                ours: false,
            },
        };
        writes.push(op);
    }
    back_up_conflicts(git, repo, conflicts)?;
    for op in &writes {
        git.run(repo, op)?;
    }
    Ok(())
}
