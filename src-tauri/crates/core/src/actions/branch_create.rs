//! 새 브랜치(D-047). 지금 버전에서 만들고 바로 갈아탄다. 고친 파일은 따라간다.

use git::GitOp;

use super::{needs_repo, repo_of};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct BranchCreateAction {
    meta: ActionMeta,
}

impl BranchCreateAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "branch-create",
                glossary_id: "branch",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::DeleteCreated,
            },
        }
    }
}

impl Default for BranchCreateAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for BranchCreateAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::BranchCreate(input) = input else {
            return Err(CoreError::InputMismatch("branch-create"));
        };
        let repo = repo_of(ctx)?;
        let name = input.name.trim();
        if !ctx.git.check_branch_name(name)? {
            return Err(CoreError::InvalidInput("쓸 수 없는 브랜치 이름이에요"));
        }
        // 대소문자만 다른 이름은 Windows에서 같은 파일이 되어 부딪힌다(H-18).
        let taken = ctx.git.branches(repo)?.iter().any(|b| {
            b.kind == git::BranchKind::Local && b.name.to_lowercase() == name.to_lowercase()
        });
        if taken {
            return Err(CoreError::BranchExists);
        }
        super::switch::refuse_during_operation(ctx.git, repo)?;
        ctx.git.run(
            repo,
            &GitOp::CreateAndSwitch {
                name: name.to_string(),
            },
        )?;
        Ok(Outcome::BranchCreated {
            name: name.to_string(),
        })
    }
}
