//! 태그 올리기(`tag-push`, D-131). 태그 하나를 원격에 덧붙이기만 하고 덮어쓰지 않아 확인이 없다.
//! 같은 이름이 다른 버전으로 원격에 있으면 git이 거절하고 그대로 알린다(`RemoteRefExists`).

use super::remote::{Net, pick_remote};
use super::repo_of;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct TagPushAction {
    meta: ActionMeta,
}

impl TagPushAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "tag-push",
                glossary_id: "tag-push",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for TagPushAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for TagPushAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::TagPush(input) = input else {
            return Err(CoreError::InputMismatch("tag-push"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let name = super::tag::tag_name(git, &input.name)?;
        if git
            .resolve_ref(repo, &format!("refs/tags/{name}^{{commit}}"))?
            .is_none()
        {
            return Err(CoreError::InvalidInput("그 태그가 없어요"));
        }
        let remote = pick_remote(git, repo)?;
        let net = Net {
            git,
            repo,
            github: ctx.github,
            background: false,
            use_helper: ctx.settings.git_helper,
        };
        net.push_tag(&remote, &name)?;
        Ok(Outcome::TagPushed { name })
    }
}
