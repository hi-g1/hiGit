//! Action 레지스트리, 실행기, Query, view 타입.
//!
//! 화면(app)은 이 크레이트만 부른다. `git`, `platform`, `hosting`, `watcher`를
//! 조합하는 곳은 여기 하나다(docs/architecture.md "계층").

#![forbid(unsafe_code)]

mod access;
mod action;
pub mod actions;
mod bootstrap;
mod conflict;
mod error;
mod executor;
pub mod git_security;
pub mod log;
pub(crate) mod parked;
mod query;
pub mod recent;
mod recovery;
mod redact;
mod registry;
mod secrets;
mod state;
pub mod trust;
pub mod undo;
pub mod view;
mod watch;

pub use action::{
    Action, ActionMeta, Availability, ExecCtx, JournalMode, Outcome, Preview, Risk,
    UnavailableReason,
};
pub use bootstrap::{APP_ID, Core, MIN_GIT_VERSION, init};
pub use error::{CoreError, Result};
pub use executor::{Executor, PreviewToken};
pub use query::{MAX_LOG_PAGE, Queries};
pub use registry::Registry;
pub use state::RepoState;
pub use watch::{WatchSignal, WatchSink};

/// core 밖(app, 테스트)에서 필요한 git 타입. app은 `git` 크레이트를 직접 의존하지 않는다.
pub use git::{GitBackend, RepoPath};
