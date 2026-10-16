//! 검토 요청에 댓글 달기(pr-comment, D-178). GitHub HTTP 하나만 쓰고 확인 없이 실행한다(D-043, D-176).
//! 본문은 보내기 전에 core가 검사하고 journal에는 남기지 않는다.

use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

use super::{needs_repo, repo_of};

pub struct PrCommentAction {
    meta: ActionMeta,
}

impl PrCommentAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "pr-comment",
                glossary_id: "comment",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for PrCommentAction {
    fn default() -> Self {
        Self::new()
    }
}

/// 앞뒤 공백을 뗀 뒤 1바이트~64KB, NUL 없음.
pub(crate) fn check_comment(body: &str) -> Result<&str> {
    let body = body.trim();
    if body.is_empty() || body.len() > hosting::COMMENT_MAX_BYTES || body.contains('\0') {
        return Err(CoreError::InvalidInput("댓글"));
    }
    Ok(body)
}

impl Action for PrCommentAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::PrComment(i) = input else {
            return Err(CoreError::InputMismatch("pr-comment"));
        };
        if i.number == 0 {
            return Err(CoreError::InvalidInput("검토 요청 번호"));
        }
        let body = check_comment(&i.body)?;
        let repo = repo_of(ctx)?;
        let github_repo = crate::github::pr::github_repo(ctx.git, repo)?;
        ctx.github.call(|provider, token| {
            provider.create_comment(token, &github_repo, u64::from(i.number), body)
        })?;
        Ok(Outcome::PrCommented { number: i.number })
    }
}
