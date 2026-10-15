//! 등록된 액션들. 새 액션 추가 절차는 docs/architecture.md "새 액션 추가 절차".

mod init;

pub use init::InitAction;

use crate::{ActionMeta, Registry, Result};

/// 앱이 쓰는 액션을 모두 등록한다.
pub fn register_all(registry: &mut Registry) -> Result<()> {
    registry.register(Box::new(InitAction::new()))?;
    Ok(())
}

/// 등록되는 액션의 메타 목록. git 없이도 만들 수 있다(바인딩 생성용).
pub fn all_metas() -> Vec<ActionMeta> {
    let mut registry = Registry::new();
    register_all(&mut registry).expect("액션 id가 겹치지 않아야 한다");
    registry.metas().cloned().collect()
}
