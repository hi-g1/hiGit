//! `GitBackend` 트레이트와 시스템 git CLI 구현(D-003).
//!
//! 다른 내부 크레이트에 의존하지 않는다. 출력은 기계용 형식만 파싱한다.

mod backend;
mod cli;
mod error;
mod types;

#[cfg(test)]
mod testutil;

pub use backend::GitBackend;
pub use cli::{CliBackend, ConfigureCommand};
pub use error::{GitError, Result};
pub use types::{GitVersion, RepoPath};
