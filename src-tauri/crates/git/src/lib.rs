//! `GitBackend` 트레이트와 시스템 git CLI 구현(D-003).
//!
//! 다른 내부 크레이트에 의존하지 않는다. 출력은 기계용 형식만 파싱한다.

#![forbid(unsafe_code)]

mod backend;
mod cli;
mod error;
mod failure;
mod types;

#[cfg(test)]
mod testutil;

pub use backend::GitBackend;

/// 진행 알림을 받는 함수(저장소, 경과, 마지막 출력 뒤 시간).
pub type ProgressObserver = std::sync::Arc<
    dyn Fn(&std::path::Path, std::time::Duration, std::time::Duration) + Send + Sync,
>;
pub use cli::{
    BACKGROUND_TIMEOUT, CliBackend, ConfigureCommand, NETWORK_TIMEOUT, NetMode, ProcessControl,
    RefExpect, safe_relative,
};
pub use error::{GitError, Result};
pub use failure::FailureKind;
pub use types::{
    BlobHead, Branch, BranchInfo, BranchKind, Change, Commit, CommitMessage, CommitStat, DiffKind,
    DiffLine, DiffTarget, EntryKind, FileDiff, GitOp, GitVersion, GraphLogQuery, Hunk, IndexEntry,
    LineKind, LogQuery, MergeCheck, MergeSegment, Oid, OpOutput, RefEntry, ReflogEntry, Remote,
    RepoPath, StashEntry, Status, StatusEntry, TagRef,
};
