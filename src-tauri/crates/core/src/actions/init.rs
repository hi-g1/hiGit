use git::{GitOp, RepoPath};

use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
    UnavailableReason,
};

/// 관리 시작(init): 저장소가 아닌 폴더를 저장소로 만든다.
pub struct InitAction {
    meta: ActionMeta,
}

impl InitAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "init",
                glossary_id: "init",
                risk: Risk::Safe,
            },
        }
    }
}

impl Default for InitAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for InitAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        match state.repo {
            None => Availability::Available,
            Some(_) => Availability::Unavailable(UnavailableReason::AlreadyRepo),
        }
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Init = input;
        let folder = &ctx.state.folder;
        ctx.git.run(&RepoPath::new(folder), &GitOp::Init)?;
        let repo = ctx.git.discover(folder)?.ok_or(CoreError::NotARepo)?;
        Ok(Outcome::RepoCreated(repo))
    }
}
