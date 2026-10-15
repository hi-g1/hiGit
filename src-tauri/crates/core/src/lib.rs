//! Action 레지스트리, 실행기, Query, view 타입.
//!
//! 화면(app)은 이 크레이트만 부른다. `git`, `platform`, `hosting`, `watcher`를
//! 조합하는 곳은 여기 하나다(docs/architecture.md "계층").

mod action;
pub mod actions;
mod error;
mod executor;
mod registry;
mod state;

pub use action::{
    Action, ActionMeta, Availability, ExecCtx, InputField, InputKind, Outcome, Preview, Risk,
    UnavailableReason,
};
pub use error::{CoreError, Result};
pub use executor::{Executor, PreviewToken};
pub use registry::Registry;
pub use state::RepoState;

/// core 밖(app, 테스트)에서 필요한 git 타입. app은 `git` 크레이트를 직접 의존하지 않는다.
pub use git::{GitBackend, RepoPath};
