//! 남은 맡긴 항목 복구(`recover-parked`, D-082). 브랜치를 갈아타다 끊겨 stash 목록에 남은
//! "higit parked <브랜치>" 항목을 그 브랜치의 맡긴 ref로 옮긴다. HEAD와 작업 폴더는 그대로다.

use git::{GitOp, RefExpect};

use super::repo_of;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk, parked,
};

pub struct RecoverParkedAction {
    meta: ActionMeta,
}

impl RecoverParkedAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "recover-parked",
                glossary_id: "park",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for RecoverParkedAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for RecoverParkedAction {
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
        let ActionInput::RecoverParked = input else {
            return Err(CoreError::InputMismatch("recover-parked"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let leftovers = parked::leftovers(git, repo)?;
        // 오래된 것부터 옮긴다. 같은 브랜치 것이 둘이면 최근 것이 맡긴 ref에 남는다.
        for l in leftovers.iter().rev() {
            let name = parked::ref_name(&l.branch);
            match git.resolve_ref(repo, &name)? {
                None => git.update_ref(repo, &name, &l.oid, &RefExpect::Absent)?,
                Some(old) if old == l.oid => {}
                Some(old) => {
                    // 이미 맡긴 것이 있으면 백업 ref로 옮겨 두고 바꾼다(파일을 잃지 않는다).
                    crate::undo::backup::backup_oid(
                        git,
                        repo,
                        &old,
                        &format!("parked-leftover-{}", l.branch),
                    )?;
                    git.update_ref(repo, &name, &l.oid, &RefExpect::Is(old))?;
                }
            }
            git.run(repo, &GitOp::StashDropOid { oid: l.oid.clone() })?;
        }
        Ok(Outcome::ParkedRecovered {
            count: u32::try_from(leftovers.len()).unwrap_or(u32::MAX),
        })
    }
}
