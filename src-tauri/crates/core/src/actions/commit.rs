use git::{Change, GitOp, LogQuery};

use super::repo_of;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 버전 메시지의 최대 길이(D-037 IPC 입력 상한).
pub(crate) const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// 저장하기: 넣은 내용을 메시지와 함께 저장한다.
pub struct CommitAction {
    meta: ActionMeta,
}

impl CommitAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "commit",
                glossary_id: "commit",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::RestoreRef,
            },
        }
    }
}

impl Default for CommitAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for CommitAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Commit(input) = input else {
            return Err(CoreError::InputMismatch("commit"));
        };
        if input.message.len() > MAX_MESSAGE_BYTES {
            return Err(CoreError::InvalidInput("버전 메시지가 너무 길어요"));
        }
        if input.message.trim().is_empty() {
            return Err(CoreError::InvalidInput("버전 메시지가 비어 있어요"));
        }
        let repo = repo_of(ctx)?;
        super::refuse_conflicts(ctx.git, repo)?;
        // 넣은 파일이 없으면 고친 파일을 모두 넣고 저장한다(D-062).
        let mut index_before = None;
        if input.stage_all {
            let status = ctx.git.status(repo)?;
            let staged = status
                .entries
                .iter()
                .any(|e| !matches!(e.index, Change::Unmodified | Change::Untracked));
            if !staged {
                index_before = ctx.git.write_tree(repo)?;
                ctx.git.run(repo, &GitOp::StageAll)?;
            }
        }
        if let Err(e) = ctx.git.run(
            repo,
            &GitOp::Commit {
                message: input.message.clone(),
            },
        ) {
            // 저장이 실패하면(hook 거부, 이름 없음) 모두 넣은 것을 되돌려 저장 전 모습으로 둔다.
            if let Some(tree) = index_before {
                let _ = ctx.git.run(repo, &GitOp::ReadTree { tree });
            }
            return Err(e.into());
        }
        // 방금 만든 저장을 화면에 알려 준다.
        let latest = ctx.git.log(
            repo,
            &LogQuery {
                skip: 0,
                limit: 1,
                rev: None,
            },
        )?;
        let commit = latest.into_iter().next().ok_or(CoreError::NotARepo)?;
        Ok(Outcome::Committed {
            short_oid: commit.short_oid,
            summary: commit.summary,
        })
    }
}
