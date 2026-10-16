//! 마지막 버전 고치기(`amend`, D-097, D-099). 지금 위치의 버전을 메시지·넣은 파일로 다시 만든다.
//! 올린 버전은 고치지 않는다. 실행기가 고치기 전 버전을 `amend` 백업 ref로 남기고, 되돌리기는
//! 버전 저장 되돌리기처럼 이전 버전으로 브랜치를 옮긴다(넣은 파일은 그대로 넣은 채로 남는다).

use git::{Change, GitOp, LogQuery, Oid};

use super::commit::MAX_MESSAGE_BYTES;
use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::guard;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct AmendAction {
    meta: ActionMeta,
}

impl AmendAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "amend",
                glossary_id: "amend",
                // 용어표 amend의 위험도(caution).
                risk: Risk::Caution,
                undo: crate::undo::UndoKind::RestoreRef,
            },
        }
    }
}

impl Default for AmendAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for AmendAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Amend(input) = input else {
            return Err(CoreError::InputMismatch("amend"));
        };
        if let Some(message) = &input.message {
            if message.len() > MAX_MESSAGE_BYTES {
                return Err(CoreError::InvalidInput("버전 메시지가 너무 길어요"));
            }
            if message.trim().is_empty() {
                return Err(CoreError::InvalidInput("버전 메시지가 비어 있어요"));
            }
        }
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        guard::require_branch(git, repo)?;
        refuse_during_operation(git, repo)?;
        // 화면이 본 버전이 지금 위치여야 한다(그사이 바뀌었으면 다른 버전을 고치게 된다).
        let head = git
            .resolve_ref(repo, "HEAD")?
            .ok_or(CoreError::InvalidInput("고칠 버전이 없어요"))?;
        if Oid::parse(&input.expect_head).as_ref() != Some(&head) {
            return Err(CoreError::InvalidInput("지금 버전이 바뀌었어요"));
        }
        let rule = guard::published_rule(git, repo, ctx.settings.allow_published_rewrite)?;
        let published = guard::is_published(git, repo, &head)?;
        if published && rule == guard::PublishedRule::Refuse {
            return Err(CoreError::PublishedHistory);
        }
        super::refuse_conflicts(git, repo)?;
        let staged = git
            .status(repo)?
            .entries
            .iter()
            .any(|e| !matches!(e.index, Change::Unmodified | Change::Untracked));
        let same_message = match &input.message {
            None => true,
            Some(m) => git
                .commit_messages(repo, std::slice::from_ref(&head))?
                .first()
                .is_some_and(|c| c.message.trim_end() == m.trim_end()),
        };
        if same_message && !staged {
            return Err(CoreError::NoChange);
        }
        if published {
            // 바꾸기 전에 남긴다: 못 남기면 바꾸지 않는다(D-129).
            crate::rewrote::record(git, repo, ctx.key)?;
        }
        git.run(
            repo,
            &GitOp::Amend {
                message: input.message.clone(),
            },
        )?;
        let commit = git
            .log(
                repo,
                &LogQuery {
                    skip: 0,
                    limit: 1,
                    rev: None,
                },
            )?
            .into_iter()
            .next()
            .ok_or(CoreError::NotARepo)?;
        Ok(Outcome::Amended {
            short_oid: commit.short_oid,
            summary: commit.summary,
        })
    }
}
