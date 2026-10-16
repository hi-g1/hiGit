//! 복제(clone, D-115). 주소·대상 폴더 검사, 잠금, 실패 정리는 `Core::execute`가 먼저 한다.
//! 여기는 검사가 끝난 입력으로 git을 부른다.

use git::{AuthMode, GitOp, RepoPath};

use super::names;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct CloneAction {
    meta: ActionMeta,
}

impl CloneAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "clone",
                glossary_id: "clone",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for CloneAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for CloneAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    /// 새 폴더를 만드는 작업이라 열린 저장소가 필요 없다.
    fn available(&self, _state: &RepoState) -> Availability {
        Availability::Available
    }

    /// `ctx.state.folder`는 부모 폴더. 주소는 `Core`가 검사한 값이다.
    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Clone(i) = input else {
            return Err(CoreError::InputMismatch("clone"));
        };
        names::validate_clone_name(&i.name)?;
        let parent = &ctx.state.folder;
        // 저장소가 아직 없으니 주소 자체로 판정한다(D-111).
        let auth = if crate::github::github_https_only(std::slice::from_ref(&i.url))
            && ctx.github.token().is_ok()
            && ctx.github.remembered()
        {
            AuthMode::Helper
        } else {
            AuthMode::UserDefault
        };
        ctx.git.run(
            &RepoPath::new(parent),
            &GitOp::Clone {
                url: i.url.clone(),
                name: i.name.clone(),
                auth,
            },
        )?;
        let target = parent.join(&i.name);
        let repo = platform::normalize_repo_path(&target)
            .map(RepoPath::new)
            .unwrap_or_else(|_| RepoPath::new(target));
        Ok(Outcome::RepoCreated(repo))
    }
}
