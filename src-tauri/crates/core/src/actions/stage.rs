use git::GitOp;

use super::{paths, repo_of};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 저장에 넣기 / 저장에서 빼기. 둘은 입력과 git 작업만 다르다.
pub struct PathsAction {
    meta: ActionMeta,
}

impl PathsAction {
    pub fn stage() -> Self {
        Self {
            meta: ActionMeta {
                id: "stage",
                glossary_id: "stage",
                risk: Risk::Safe,
            },
        }
    }

    pub fn unstage() -> Self {
        Self {
            meta: ActionMeta {
                id: "unstage",
                glossary_id: "unstage",
                risk: Risk::Safe,
            },
        }
    }
}

impl Action for PathsAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let (op, count) = match (self.meta.id, input) {
            ("stage", ActionInput::Stage(i)) => {
                paths::validate(&i.paths)?;
                (
                    GitOp::Stage {
                        paths: i.paths.clone(),
                    },
                    i.paths.len(),
                )
            }
            ("unstage", ActionInput::Unstage(i)) => {
                paths::validate(&i.paths)?;
                (
                    GitOp::Unstage {
                        paths: i.paths.clone(),
                    },
                    i.paths.len(),
                )
            }
            _ => return Err(CoreError::InputMismatch(self.meta.id)),
        };
        ctx.git.run(repo, &op)?;
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        Ok(match op {
            GitOp::Stage { .. } => Outcome::Staged { count },
            _ => Outcome::Unstaged { count },
        })
    }
}
