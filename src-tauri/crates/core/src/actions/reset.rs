//! 이 버전으로 돌아가기(`reset`, D-097, D-099). 지금 브랜치를 노선도에서 고른 옛 버전(지금 위치의
//! 조상)으로 옮긴다. 고친 파일은 둔다(`reset --keep`). 빠지는 버전이 하나라도 올라갔으면 하지 않는다.
//! 실행기가 옮기기 전 끝을 `reset` 백업 ref로 남기고, 되돌리기는 그 끝으로 다시 옮긴다.

use git::GitOp;

use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::guard::{self, Scope};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct ResetAction {
    meta: ActionMeta,
}

impl ResetAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "reset",
                glossary_id: "reset",
                // 용어표 reset의 위험도(danger). 되돌릴 수 있어 확인은 받지 않는다(D-043).
                risk: Risk::Danger,
                undo: crate::undo::UndoKind::RestoreTree,
            },
        }
    }
}

impl Default for ResetAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for ResetAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Reset(input) = input else {
            return Err(CoreError::InputMismatch("reset"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        guard::require_branch(git, repo)?;
        refuse_during_operation(git, repo)?;
        let oid = guard::verified_commit(git, repo, &input.oid, Scope::HeadAncestor)?;
        let head = git
            .resolve_ref(repo, "HEAD")?
            .ok_or(CoreError::InvalidInput("없는 버전이에요"))?;
        if head == oid {
            return Err(CoreError::InvalidInput("지금 버전과 같아요"));
        }
        let rule = guard::published_rule(git, repo, ctx.settings.allow_published_rewrite)?;
        if guard::touches_published(git, repo, &head, &oid)? {
            if rule == guard::PublishedRule::Refuse {
                return Err(CoreError::PublishedHistory);
            }
            // 바꾸기 전에 남긴다: 못 남기면 바꾸지 않는다(D-129).
            crate::rewrote::record(git, repo, ctx.key)?;
        }
        let (_, dropped) = git.count_between(repo, oid.as_str(), head.as_str())?;
        git.run(repo, &GitOp::ResetKeep { oid })?;
        Ok(Outcome::Reset { dropped })
    }
}
