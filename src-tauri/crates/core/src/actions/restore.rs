//! 작업 내역의 "이 시점으로 되돌리기"(`restore`, D-085). 지금 브랜치를 그 버전으로 옮기되
//! 고친 파일은 두고(`reset --keep`), 옮기기 전에 지금 버전을 백업 ref로 남긴다.

use git::{GitOp, Oid};

use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::view::{ActionInput, RestoreKind};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct RestoreAction {
    meta: ActionMeta,
}

impl RestoreAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "restore",
                glossary_id: "reflog",
                // 용어표 reflog의 위험도(safe). 지금 버전 백업은 아래에서 직접 남긴다.
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::RestoreTree,
            },
        }
    }
}

impl Default for RestoreAction {
    fn default() -> Self {
        Self::new()
    }
}

/// 작업 내역에 "버전"으로 나오는 해시인지(reflog 또는 버전 모양 백업 ref). 화면이 아무 해시나 줄 수 없다.
fn listed_version(git: &dyn git::GitBackend, repo: &git::RepoPath, oid: &Oid) -> Result<bool> {
    if git.reflog(repo, 200)?.iter().any(|r| &r.oid == oid) {
        return Ok(true);
    }
    for r in git.list_refs(repo, crate::undo::backup::BACKUP_PREFIX)? {
        if r.oid == oid.as_str()
            && crate::recovery::kind_of(git, repo, &r.oid) == RestoreKind::Version
        {
            return Ok(true);
        }
    }
    Ok(false)
}

impl Action for RestoreAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Restore(input) = input else {
            return Err(CoreError::InputMismatch("restore"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let oid = Oid::parse(&input.oid).ok_or(CoreError::InvalidInput("없는 버전이에요"))?;
        let branch = git.current_branch(repo)?.ok_or(CoreError::InvalidInput(
            "특정 버전을 보는 중에는 할 수 없어요",
        ))?;
        refuse_during_operation(git, repo)?;
        if !listed_version(git, repo, &oid)? {
            return Err(CoreError::InvalidInput("없는 버전이에요"));
        }
        // 버전(커밋)이어야 한다.
        if git
            .resolve_ref(repo, &format!("{oid}^{{commit}}"))?
            .as_ref()
            != Some(&oid)
        {
            return Err(CoreError::InvalidInput("없는 버전이에요"));
        }
        let head = git
            .resolve_ref(repo, "HEAD")?
            .ok_or(CoreError::InvalidInput("없는 버전이에요"))?;
        if head == oid {
            return Err(CoreError::InvalidInput("지금 버전과 같아요"));
        }
        // 되살려도 지금 버전은 남는다(작업 내역에 새 줄로 보인다).
        crate::undo::backup::backup_oid(git, repo, &head, &format!("restore-{branch}"))?;
        git.run(repo, &GitOp::ResetKeep { oid })?;
        Ok(Outcome::Restored)
    }
}
