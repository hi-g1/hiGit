//! 전역 이름·이메일 저장(`identity-set`, D-125). 저장소가 필요 없어 `Core::execute`가 폴더 없이 부른다.
//! 값은 core가 다시 검사하고 로그·결과에 담지 않는다.

use git::GlobalOp;

use super::names;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct IdentityAction {
    meta: ActionMeta,
}

impl IdentityAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "identity-set",
                glossary_id: "identity",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for IdentityAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for IdentityAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, _state: &RepoState) -> Availability {
        Availability::Available
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::IdentitySet(i) = input else {
            return Err(CoreError::InputMismatch("identity-set"));
        };
        let (name, email) = names::validate_identity(&i.name, &i.email)?;
        ctx.git.run_global(&GlobalOp::SetIdentity { name, email })?;
        Ok(Outcome::IdentitySaved)
    }
}
