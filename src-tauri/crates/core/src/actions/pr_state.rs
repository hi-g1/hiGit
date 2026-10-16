//! 검토 요청 닫기·다시 열기(pr-close·pr-reopen, D-180). 다시 열 수 있어 확인 없이 실행한다(D-043).

use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, GithubFailure, Outcome, RepoState,
    Result, Risk,
};

use super::{needs_repo, repo_of};

pub struct PrStateAction {
    meta: ActionMeta,
    open: bool,
}

impl PrStateAction {
    pub fn close() -> Self {
        Self {
            meta: ActionMeta {
                id: "pr-close",
                glossary_id: "pr-close",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Nothing,
            },
            open: false,
        }
    }

    pub fn reopen() -> Self {
        Self {
            meta: ActionMeta {
                id: "pr-reopen",
                glossary_id: "pr",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Nothing,
            },
            open: true,
        }
    }
}

impl Action for PrStateAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let number = match (self.open, input) {
            (false, ActionInput::PrClose(i)) | (true, ActionInput::PrReopen(i)) => i.number,
            _ => return Err(CoreError::InputMismatch(self.meta.id)),
        };
        if number == 0 {
            return Err(CoreError::InvalidInput("검토 요청 번호"));
        }
        let repo = repo_of(ctx)?;
        let github_repo = crate::github::pr::github_repo(ctx.git, repo)?;
        let open = self.open;
        let result = ctx.github.call(|provider, token| {
            Ok(
                match provider.set_pr_state(token, &github_repo, u64::from(number), open) {
                    Ok(()) => Ok(()),
                    // 닫힌 것을 닫거나 열린 것을 열거나 다시 열 수 없음(422).
                    Err(e) if matches!(e.kind(), hosting::HostingErrorKind::Validation(_)) => {
                        Err(e.detail().map(str::to_owned))
                    }
                    Err(e) => return Err(e),
                },
            )
        })?;
        match result {
            Ok(()) if open => Ok(Outcome::PrReopened { number }),
            Ok(()) => Ok(Outcome::PrClosed { number }),
            Err(detail) => Err(CoreError::Github {
                kind: GithubFailure::PrNotOpen,
                detail,
            }),
        }
    }
}
