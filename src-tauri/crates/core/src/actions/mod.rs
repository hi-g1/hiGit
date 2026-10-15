//! 등록된 액션들. 새 액션 추가 절차는 docs/architecture.md "새 액션 추가 절차".

mod init;

pub use init::InitAction;

use crate::{Registry, Result};

/// 앱이 쓰는 액션을 모두 등록한다.
pub fn register_all(registry: &mut Registry) -> Result<()> {
    registry.register(Box::new(InitAction::new()))?;
    Ok(())
}
