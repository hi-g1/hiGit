//! 고친 것 버리기(`discard`, D-043, D-074). 되돌릴 수 없어 실행 전 확인을 받고, 버리기 전에
//! 스냅샷을 백업 ref로 남겨 작업 내역에서 꺼낼 수 있게 한다.

use git::GitOp;

use super::{paths, repo_of};
use crate::actions::switch::refuse_during_operation;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct DiscardAction {
    meta: ActionMeta,
}

impl DiscardAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "discard",
                glossary_id: "discard",
                risk: Risk::Caution,
                undo: crate::undo::UndoKind::Irreversible,
            },
        }
    }
}

impl Default for DiscardAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for DiscardAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    /// HEAD는 그대로라 실행기의 HEAD 백업이 필요 없다(버린 내용은 아래 스냅샷이 남긴다).
    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Discard(input) = input else {
            return Err(CoreError::InputMismatch("discard"));
        };
        paths::validate(&input.paths)?;
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        refuse_during_operation(git, repo)?;
        super::refuse_conflicts(git, repo)?;
        if git.resolve_ref(repo, "HEAD")?.is_none() {
            return Err(CoreError::InvalidInput(
                "버릴 기준이 되는 버전이 아직 없어요",
            ));
        }
        let branch = git
            .current_branch(repo)?
            .unwrap_or_else(|| "HEAD".to_string());
        // stash 한 번으로 버리기와 스냅샷을 함께 한다. 파일을 잃을 틈이 없다.
        let out = git.run(
            repo,
            &GitOp::DiscardPaths {
                paths: input.paths.clone(),
                message: format!("higit discard {branch}"),
            },
        )?;
        let Some(oid) = out.oid else {
            return Ok(Outcome::Discarded { count: 0 });
        };
        // 백업 ref를 못 쓰면 이미 버렸으므로 stash를 목록에 남기고 알린다(파일은 임시 보관 목록에 있다).
        if crate::undo::backup::backup_oid(git, repo, &oid, &format!("discard-{branch}")).is_err() {
            return Err(CoreError::BackupFailed);
        }
        git.run(repo, &GitOp::StashDropOid { oid })?;
        Ok(Outcome::Discarded {
            count: u32::try_from(input.paths.len()).unwrap_or(u32::MAX),
        })
    }
}
