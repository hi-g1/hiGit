//! 등록된 액션들. 새 액션 추가 절차는 docs/architecture.md "새 액션 추가 절차".

mod commit;
mod init;
pub(crate) mod paths;
mod stage;

pub use commit::CommitAction;
pub use init::InitAction;
pub use stage::PathsAction;

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

/// 앱이 쓰는 액션을 모두 등록한다.
pub fn register_all(registry: &mut Registry) -> Result<()> {
    registry.register(Box::new(InitAction::new()))?;
    registry.register(Box::new(PathsAction::stage()))?;
    registry.register(Box::new(PathsAction::unstage()))?;
    registry.register(Box::new(CommitAction::new()))?;
    Ok(())
}

/// 등록되는 액션의 메타 목록. git 없이도 만들 수 있다(바인딩 생성용).
pub fn all_metas() -> Vec<ActionMeta> {
    let mut registry = Registry::new();
    register_all(&mut registry).expect("액션 id가 겹치지 않아야 한다");
    registry.metas().cloned().collect()
}
