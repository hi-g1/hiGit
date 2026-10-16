//! 고친 것 버리기(`discard`, D-074, D-196). 버리기 전에 스냅샷을 백업 ref로 남기고, 그 이름과 해시를
//! 작업 목록(journal)에 남겨 Ctrl+Z로 되살린다. 작업 내역에서도 꺼낼 수 있다. 확인 창은 없다.

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
                undo: crate::undo::UndoKind::Inverse,
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
        // 고른 경로만 담은 스냅샷을 먼저 백업 ref로 남기고, 그다음에 버린다. 파일을 잃을 틈이 없다.
        let out = git.run(
            repo,
            &GitOp::SnapshotPaths {
                paths: input.paths.clone(),
                message: format!("higit discard {branch}"),
            },
        )?;
        let Some(oid) = out.oid else {
            return Ok(Outcome::Discarded {
                count: 0,
                backup: None,
            });
        };
        // 백업 ref를 못 쓰면 버리지 않는다.
        let Ok(name) =
            crate::undo::backup::backup_oid(git, repo, &oid, &format!("discard-{branch}"))
        else {
            return Err(CoreError::BackupFailed);
        };
        git.run(
            repo,
            &GitOp::DiscardPaths {
                paths: input.paths.clone(),
            },
        )?;
        Ok(Outcome::Discarded {
            count: u32::try_from(input.paths.len()).unwrap_or(u32::MAX),
            backup: Some((name, oid)),
        })
    }
}
