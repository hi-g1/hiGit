//! 합치기(`merge`, D-048, D-069)와 "main에 합치기"(`merge-into`, D-071).

use git::{GitOp, Oid, RefExpect};

use super::{names, repo_of};
use crate::actions::switch::refuse_during_operation;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct MergeAction {
    meta: ActionMeta,
}

impl MergeAction {
    pub fn merge() -> Self {
        Self {
            meta: ActionMeta {
                id: "merge",
                glossary_id: "merge",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::RestoreTree,
            },
        }
    }
}

/// 합칠 브랜치 이름 검사와 전체 ref 이름. 같은 이름의 태그가 먼저 풀리지 않게 `refs/heads/`로 넘긴다.
pub(crate) fn local_branch_ref(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    branch: &str,
) -> Result<(String, Oid)> {
    names::validate_ref_like(branch)?;
    if !git.check_branch_name(branch)? {
        return Err(CoreError::InvalidInput("쓸 수 없는 브랜치 이름이에요"));
    }
    let full = format!("refs/heads/{branch}");
    let oid = git
        .resolve_ref(repo, &full)?
        .ok_or(CoreError::InvalidInput("그 브랜치가 없어요"))?;
    Ok((full, oid))
}

/// 이번 버전에 넣은 파일이 있으면 거절한다(합친 버전에 섞이지 않게, D-069).
pub(crate) fn refuse_staged(git: &dyn git::GitBackend, repo: &git::RepoPath) -> Result<()> {
    use git::Change;
    let staged = git
        .status(repo)?
        .entries
        .iter()
        .any(|e| !matches!(e.index, Change::Unmodified | Change::Untracked));
    if staged {
        return Err(CoreError::StagedChanges);
    }
    Ok(())
}

/// 합치는 중인 충돌 파일 경로.
pub(crate) fn conflict_paths(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
) -> Result<Vec<String>> {
    let mut paths: Vec<String> = git
        .index_entries(repo)?
        .into_iter()
        .filter(|e| e.stage > 0)
        .map(|e| e.path)
        .collect();
    paths.dedup();
    Ok(paths)
}

impl Action for MergeAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Merge(input) = input else {
            return Err(CoreError::InputMismatch("merge"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let (full, _) = local_branch_ref(git, repo, &input.branch)?;
        if git.current_branch(repo)?.as_deref() == Some(input.branch.as_str()) {
            return Err(CoreError::InvalidInput("지금 브랜치와 같아요"));
        }
        refuse_during_operation(git, repo)?;
        refuse_staged(git, repo)?;
        let (ahead, behind) = git.count_between(repo, "HEAD", &full)?;
        if behind == 0 {
            return Err(CoreError::InvalidInput("가져올 버전이 없어요"));
        }
        if ahead == 0 {
            git.run(repo, &GitOp::MergeFf { rev: full })?;
            return Ok(Outcome::Merged {
                branch: input.branch.clone(),
                fast_forward: true,
            });
        }
        let message = if input.locale.starts_with("ko") {
            format!("{} 가져와 합침", input.branch)
        } else {
            format!("Merge {}", input.branch)
        };
        match git.run(repo, &GitOp::Merge { rev: full, message }) {
            Ok(_) => Ok(Outcome::Merged {
                branch: input.branch.clone(),
                fast_forward: false,
            }),
            // 겹치면 그만두지 않고 충돌 해결 화면으로 간다. hook이 합친 버전을 막아도 합치는 중으로
            // 남는다(충돌 0곳, 마치기만 남음).
            Err(e) if merging(git, repo)? => {
                let _ = e;
                Ok(Outcome::MergeConflicts {
                    paths: conflict_paths(git, repo)?,
                })
            }
            Err(e) => Err(e.into()),
        }
    }
}

/// 합치는 중인지(`MERGE_HEAD`가 있는지).
pub(crate) fn merging(git: &dyn git::GitBackend, repo: &git::RepoPath) -> Result<bool> {
    Ok(git.resolve_ref(repo, "MERGE_HEAD")?.is_some())
}

/// "main에 합치기": 갈아타지 않고 기본 브랜치를 지금 버전으로 앞으로 감는다(D-071).
pub struct MergeIntoAction {
    meta: ActionMeta,
}

impl MergeIntoAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "merge-into",
                glossary_id: "merge",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::RestoreRef,
            },
        }
    }
}

impl Default for MergeIntoAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for MergeIntoAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn tracked_refs(&self, input: &ActionInput) -> Vec<String> {
        match input {
            ActionInput::MergeInto(i) => vec![format!("refs/heads/{}", i.branch)],
            _ => Vec::new(),
        }
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::MergeInto(input) = input else {
            return Err(CoreError::InputMismatch("merge-into"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let (full, target) = local_branch_ref(git, repo, &input.branch)?;
        if git.current_branch(repo)?.as_deref() == Some(input.branch.as_str()) {
            return Err(CoreError::InvalidInput("지금 브랜치와 같아요"));
        }
        refuse_during_operation(git, repo)?;
        let head = git
            .resolve_ref(repo, "HEAD")?
            .ok_or(CoreError::InvalidInput("저장한 버전이 없어요"))?;
        if head == target {
            return Err(CoreError::InvalidInput("합칠 버전이 없어요"));
        }
        // 기본 브랜치가 지금 버전의 조상일 때만(뒤처지지 않았을 때만) 앞으로 감는다.
        if !git.is_ancestor(repo, target.as_str(), head.as_str())? {
            return Err(git::GitError::Refused {
                kind: git::FailureKind::NotFastForward,
                detail: "not a fast-forward".into(),
            }
            .into());
        }
        if checked_out_elsewhere(git, repo, &input.branch)? {
            return Err(CoreError::InvalidInput(
                "다른 작업 폴더에서 쓰는 브랜치예요",
            ));
        }
        git.update_ref(repo, &full, &head, &RefExpect::Is(target))?;
        Ok(Outcome::Merged {
            branch: input.branch.clone(),
            fast_forward: true,
        })
    }
}

/// 다른 worktree가 그 브랜치를 쓰는 중인지. 그 작업 폴더를 몰래 바꾸지 않게 막는다.
fn checked_out_elsewhere(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    branch: &str,
) -> Result<bool> {
    let common = git.common_dir(repo)?;
    let Ok(read) = std::fs::read_dir(common.join("worktrees")) else {
        return Ok(false);
    };
    let want = format!("ref: refs/heads/{branch}");
    for entry in read.filter_map(|e| e.ok()) {
        if let Ok(head) = std::fs::read_to_string(entry.path().join("HEAD"))
            && head.trim() == want
        {
            return Ok(true);
        }
    }
    Ok(false)
}
