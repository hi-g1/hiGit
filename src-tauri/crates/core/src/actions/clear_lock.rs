//! 멈춘 작업 정리(`clear-lock`, D-081). 중간에 끊긴 git 작업이 남긴 잠금 파일을 지운다.
//! 되돌릴 수 없어 확인을 받지만, HEAD를 바꾸지 않아 작업 목록에 경계를 남기지 않는다.

use std::time::SystemTime;

use super::repo_of;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct ClearLockAction {
    meta: ActionMeta,
}

impl ClearLockAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "clear-lock",
                glossary_id: "lock",
                risk: Risk::Caution,
                undo: crate::undo::UndoKind::Irreversible,
            },
        }
    }
}

impl Default for ClearLockAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for ClearLockAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    /// core가 이 저장소의 작업 폴더·원격 잠금을 모두 잡은 채 부른다(앱의 다른 작업이 없다).
    /// 확인한 뒤 다시 조사해 그때도 버려진 것만 지운다.
    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::ClearLock = input else {
            return Err(CoreError::InputMismatch("clear-lock"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let git_dir = git.git_dir(repo)?;
        let common_dir = git.common_dir(repo)?;
        if crate::locks::find(&git_dir, &common_dir).is_empty() {
            return Ok(Outcome::LocksCleared { count: 0 });
        }
        let stale = crate::locks::stale(&git_dir, &common_dir, SystemTime::now(), || {
            git.other_git_running()
        });
        if stale.is_empty() {
            return Err(CoreError::Busy);
        }
        for lock in &stale {
            crate::locks::remove(lock, &[&git_dir, &common_dir])?;
        }
        Ok(Outcome::LocksCleared {
            count: u32::try_from(stale.len()).unwrap_or(u32::MAX),
        })
    }
}
