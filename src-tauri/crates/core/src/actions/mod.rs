//! 등록된 액션들. 새 액션 추가 절차는 docs/architecture.md "새 액션 추가 절차".

mod amend;
mod branch_create;
mod branch_edit;
mod clear_lock;
mod commit;
pub(crate) mod conflict;
mod discard;
pub(crate) mod ignore;
mod init;
pub(crate) mod merge;
pub(crate) mod names;
pub(crate) mod paths;
mod pick;
mod recover_parked;
mod remote;
mod reset;
mod restore;
mod rewrite;
mod stage;
pub(crate) mod stash;
pub(crate) mod switch;
mod tag;

pub use amend::AmendAction;
pub use branch_create::BranchCreateAction;
pub use branch_edit::BranchEditAction;
pub use clear_lock::ClearLockAction;
pub use commit::CommitAction;
pub use conflict::ConflictAction;
pub use discard::DiscardAction;
pub use ignore::IgnoreAction;
pub use init::InitAction;
pub use merge::{MergeAction, MergeIntoAction};
pub use pick::PickAction;
pub use recover_parked::RecoverParkedAction;
pub use remote::{RemoteAction, SyncReport};
pub use reset::ResetAction;
pub use restore::RestoreAction;
pub use rewrite::RewriteAction;
pub use stage::PathsAction;
pub use stash::StashAction;
pub use switch::{SwitchAction, SwitchReport};
pub use tag::TagAction;

use git::RepoPath;

use crate::{
    ActionMeta, Availability, CoreError, ExecCtx, Registry, RepoState, Result, UnavailableReason,
};

/// 저장소 안에서만 쓰는 액션의 `available`.
fn needs_repo(state: &RepoState) -> Availability {
    match state.repo {
        Some(_) => Availability::Available,
        None => Availability::Unavailable(UnavailableReason::NotARepo),
    }
}

fn repo_of<'a>(ctx: &'a ExecCtx<'_>) -> Result<&'a RepoPath> {
    ctx.state.repo.as_ref().ok_or(CoreError::NotARepo)
}

/// 충돌이 남은 파일이 있으면 거절한다. 그대로 넣거나 저장하면 git의 충돌 표시가 든 파일이
/// 버전에 들어간다(다른 도구로 시작한 리베이스·cherry-pick 중 홈에서 누른 경우).
fn refuse_conflicts(git: &dyn git::GitBackend, repo: &RepoPath) -> Result<()> {
    let unmerged = git
        .status(repo)?
        .entries
        .iter()
        .any(|e| e.kind == git::EntryKind::Unmerged);
    if unmerged {
        return Err(CoreError::OperationInProgress);
    }
    Ok(())
}

/// 앱이 쓰는 액션을 모두 등록한다.
pub fn register_all(registry: &mut Registry) -> Result<()> {
    registry.register(Box::new(InitAction::new()))?;
    registry.register(Box::new(PathsAction::stage()))?;
    registry.register(Box::new(PathsAction::unstage()))?;
    registry.register(Box::new(CommitAction::new()))?;
    registry.register(Box::new(IgnoreAction::new()))?;
    registry.register(Box::new(MergeAction::merge()))?;
    registry.register(Box::new(MergeIntoAction::new()))?;
    registry.register(Box::new(ConflictAction::finish()))?;
    registry.register(Box::new(ConflictAction::abort()))?;
    registry.register(Box::new(StashAction::stash()))?;
    registry.register(Box::new(StashAction::take_out()))?;
    registry.register(Box::new(DiscardAction::new()))?;
    registry.register(Box::new(RestoreAction::new()))?;
    registry.register(Box::new(ClearLockAction::new()))?;
    registry.register(Box::new(RecoverParkedAction::new()))?;
    registry.register(Box::new(SwitchAction::new()))?;
    registry.register(Box::new(BranchCreateAction::new()))?;
    registry.register(Box::new(RemoteAction::fetch()))?;
    registry.register(Box::new(RemoteAction::pull()))?;
    registry.register(Box::new(RemoteAction::push()))?;
    registry.register(Box::new(RemoteAction::sync()))?;
    registry.register(Box::new(AmendAction::new()))?;
    registry.register(Box::new(PickAction::revert()))?;
    registry.register(Box::new(PickAction::cherry_pick()))?;
    registry.register(Box::new(ResetAction::new()))?;
    registry.register(Box::new(RewriteAction::new()))?;
    registry.register(Box::new(TagAction::create()))?;
    registry.register(Box::new(TagAction::delete()))?;
    registry.register(Box::new(BranchEditAction::delete()))?;
    registry.register(Box::new(BranchEditAction::rename()))?;
    Ok(())
}

/// 등록되는 액션의 메타 목록. git 없이도 만들 수 있다(바인딩 생성용).
// 등록 목록은 코드에 고정돼 있고 겹치면 테스트가 먼저 실패한다. 바인딩 생성과 테스트만 부른다.
#[allow(clippy::expect_used)]
pub fn all_metas() -> Vec<ActionMeta> {
    let mut registry = Registry::new();
    register_all(&mut registry).expect("액션 id가 겹치지 않아야 한다");
    registry.metas().cloned().collect()
}
