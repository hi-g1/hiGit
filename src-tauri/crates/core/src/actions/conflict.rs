//! 합치기 마치기·그만두기(D-069, D-070). 충돌 파일을 덮어쓰기 전에 작업 폴더 내용을 백업한다.
//! 앱이 시작한 복사·되돌리기가 멈춘 것도 같은 화면에서 마치거나 그만둔다(D-101).

use std::collections::HashSet;

use git::GitOp;

use super::{paths, repo_of};
use crate::conflict::{ConflictPath, Context, Pick, assemble, conflict_paths, load_file};
use crate::operation::{self, OpKind, OpMarker};
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

/// 다른 도구가 시작한 리베이스·cherry-pick·revert 중이면 앱이 끝내지 않는다. 앱이 시작해 서명한
/// 표식과 git 상태가 맞는 작업만 통과한다(D-102).
fn refuse_other_operation(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    key: &crate::undo::JournalKey,
) -> Result<()> {
    if crate::conflict::foreign_operation(git, repo, key)?.is_some() {
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
    back_up_files(
        git,
        repo,
        conflicts.iter().map(|c| c.path.clone()).collect(),
    )
}

/// 고른 파일의 지금 작업 폴더 내용을 `conflict-files-<브랜치>` 백업 ref로 남긴다.
fn back_up_files(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    paths: Vec<String>,
) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let out = git.run(
        repo,
        &GitOp::SnapshotFiles {
            paths,
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

    /// 기록 정리를 마치면 정리 중 떨어진 HEAD(브랜치 없음)가 아니라 정리를 시작할 때의 끝과 브랜치를
    /// 되돌리기 기준으로 남긴다(표식의 `head_before`, D-100).
    fn journal_before(
        &self,
        ctx: &ExecCtx<'_>,
        _input: &ActionInput,
        captured: crate::undo::UndoState,
    ) -> crate::undo::UndoState {
        let Some(repo) = ctx.state.repo.as_ref() else {
            return captured;
        };
        match crate::conflict::context(ctx.git, repo, ctx.key) {
            Ok(Some(Context::Operation(op))) if op.kind == OpKind::Rebase => {
                crate::undo::UndoState {
                    head: Some(op.head_before),
                    branch: Some(op.branch),
                    ..captured
                }
            }
            _ => captured,
        }
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        refuse_other_operation(git, repo, ctx.key)?;
        let Some(context) = crate::conflict::context(git, repo, ctx.key)? else {
            return Err(CoreError::InvalidInput("합치는 중이 아니에요"));
        };
        let flip = context.flipped();
        let conflicts = conflict_paths(git.index_entries(repo)?, flip);
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
            ("conflict-abort", ActionInput::ConflictAbort, Context::Operation(op))
                if op.kind != OpKind::Rebase =>
            {
                back_up_conflicts(git, repo, &conflicts)?;
                git.run(
                    repo,
                    &GitOp::OperationAbort {
                        kind: op.kind.git(),
                    },
                )?;
                operation::clear(&git.git_dir(repo)?);
                Ok(Outcome::ConflictAborted)
            }
            ("conflict-finish", ActionInput::ConflictFinish(input), Context::Operation(op))
                if op.kind != OpKind::Rebase =>
            {
                write_resolutions(git, repo, &conflicts, input, flip)?;
                finish_pick(git, repo, &op)?;
                Ok(Outcome::ConflictFinished)
            }
            ("conflict-abort", ActionInput::ConflictAbort, Context::Operation(op)) => {
                // 기록 정리를 그만두면 git이 원래 끝으로 되돌리므로(`reset --hard`) 충돌 파일뿐
                // 아니라 그사이 고친 추적 파일도 모두 백업한다.
                let mut paths: Vec<String> = conflicts.iter().map(|c| c.path.clone()).collect();
                for e in git.status(repo)?.entries {
                    let changed = |c: git::Change| {
                        !matches!(c, git::Change::Unmodified | git::Change::Untracked)
                    };
                    if (changed(e.index) || changed(e.worktree)) && !paths.contains(&e.path) {
                        paths.push(e.path);
                    }
                }
                paths.truncate(MAX_FILES);
                back_up_files(git, repo, paths)?;
                git.run(
                    repo,
                    &GitOp::OperationAbort {
                        kind: op.kind.git(),
                    },
                )?;
                operation::clear(&git.git_dir(repo)?);
                Ok(Outcome::ConflictAborted)
            }
            ("conflict-finish", ActionInput::ConflictFinish(input), Context::Operation(op)) => {
                // 할 일 목록이 앱이 쓴 모양이 아니면 아무것도 쓰지 않는다.
                verify_rewrite(git, repo, &op)?;
                write_resolutions(git, repo, &conflicts, input, flip)?;
                match finish_rewrite(ctx, repo, &op)? {
                    Some(paths) => Ok(Outcome::ConflictContinued { paths }),
                    None => Ok(Outcome::ConflictFinished),
                }
            }
            ("conflict-finish", ActionInput::ConflictFinish(input), Context::Merge) => {
                write_resolutions(git, repo, &conflicts, input, flip)?;
                commit_merge(git, repo, &input.locale)?;
                Ok(Outcome::ConflictFinished)
            }
            ("conflict-finish", ActionInput::ConflictFinish(input), Context::Unpark(marker)) => {
                write_resolutions(git, repo, &conflicts, input, flip)?;
                finish_unpark(git, repo, &marker)?;
                Ok(Outcome::ParkedResolved)
            }
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}

/// 멈춘 기록 정리가 표식과 맞고 남은 할 일 목록이 앱이 쓴 모양 그대로인지.
fn verify_rewrite(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    op: &OpMarker,
) -> Result<git::RebaseState> {
    let state = git.rebase_state(repo)?.ok_or(CoreError::ConflictChanged)?;
    if !operation::verify_state(git, repo, op)? || !operation::verify_todo(&state, op) {
        return Err(CoreError::OperationInProgress);
    }
    Ok(state)
}

/// 기록 정리를 이어간다. 표식 서명·git 상태·남은 할 일 목록이 모두 맞을 때만 git을 부른다(D-102).
/// `edit`으로 멈춰 있으면(앱이 꺼졌던 경우) 표식의 메시지로 고친 뒤 이어간다. 해결한 결과가 비면 그
/// 버전은 건너뛴다. 끝나면 `None`, 다시 충돌로 멈추면 그 경로.
fn finish_rewrite(
    ctx: &ExecCtx<'_>,
    repo: &git::RepoPath,
    op: &OpMarker,
) -> Result<Option<Vec<String>>> {
    let git = ctx.git;
    let state = verify_rewrite(git, repo, op)?;
    let unmerged = |git: &dyn git::GitBackend| -> Result<bool> {
        Ok(git.index_entries(repo)?.iter().any(|e| e.stage > 0))
    };
    if state.stopped_at_edit && !unmerged(git)? {
        return crate::actions::rewrite::drive(ctx, repo, op, None);
    }
    let kind = git::OperationKind::Rebase;
    let mut error = git.run(repo, &GitOp::OperationContinue { kind }).err();
    if error.is_some() && git.rebase_state(repo)?.is_some() && !unmerged(git)? {
        let staged = git
            .status(repo)?
            .entries
            .iter()
            .any(|e| !matches!(e.index, git::Change::Unmodified | git::Change::Untracked));
        if !staged {
            // 해결한 결과가 정리된 기록과 같아 비었다. 그 버전을 빼고 이어간다.
            error = git.run(repo, &GitOp::OperationSkip { kind }).err();
        }
    }
    crate::actions::rewrite::drive(ctx, repo, op, error)
}

/// 복사·되돌리기를 이어서 새 버전을 만들고 표식을 지운다. 해결한 결과가 지금과 같으면 그 버전은
/// 건너뛰고 `NoChange`. hook이 막으면 해결한 것은 남아 다시 마치면 이어진다.
fn finish_pick(git: &dyn git::GitBackend, repo: &git::RepoPath, op: &OpMarker) -> Result<()> {
    // `--continue`는 표식 서명과 git 상태가 둘 다 맞을 때만 부른다(D-102).
    if !operation::verify_state(git, repo, op)? {
        return Err(CoreError::ConflictChanged);
    }
    let kind = op.kind.git();
    let git_dir = git.git_dir(repo)?;
    if let Err(e) = git.run(repo, &GitOp::OperationContinue { kind }) {
        let staged = git
            .status(repo)?
            .entries
            .iter()
            .any(|e| !matches!(e.index, git::Change::Unmodified | git::Change::Untracked));
        let unmerged = git.index_entries(repo)?.iter().any(|e| e.stage > 0);
        if operation::verify_state(git, repo, op)? && !staged && !unmerged {
            git.run(repo, &GitOp::OperationSkip { kind })?;
            operation::clear(&git_dir);
            return Err(CoreError::NoChange);
        }
        return Err(e.into());
    }
    operation::clear(&git_dir);
    Ok(())
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
    flip: bool,
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
            // `ours`는 git의 stage 2다. 쪽을 뒤집은 맥락(기록 정리)에서는 화면의 내 쪽이 stage 3.
            (FileChoice::Ours, _, _) => GitOp::ResolveSide {
                path: c.path.clone(),
                ours: !flip,
            },
            (FileChoice::Theirs, _, _) => GitOp::ResolveSide {
                path: c.path.clone(),
                ours: flip,
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
