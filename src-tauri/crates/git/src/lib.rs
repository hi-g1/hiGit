//! `GitBackend` 트레이트와 시스템 git CLI 구현(D-003).
//!
//! 다른 내부 크레이트에 의존하지 않는다. 출력은 기계용 형식만 파싱한다.

#![forbid(unsafe_code)]

mod backend;
mod cli;
mod error;
mod failure;
mod progress;
mod types;

#[cfg(test)]
mod testutil;

pub use backend::{GitBackend, RepoPolicy};

/// 진행 알림을 받는 함수(저장소, 경과, 마지막 출력 뒤 시간, Git이 준 마지막 개수).
pub type ProgressObserver = std::sync::Arc<
    dyn Fn(&std::path::Path, std::time::Duration, std::time::Duration, Option<ProgressCount>)
        + Send
        + Sync,
>;
pub use cli::{
    BACKGROUND_TIMEOUT, CLONE_TIMEOUT, CliBackend, ConfigureCommand, HELPER_ENV, NETWORK_TIMEOUT,
    NetMode, ProcessControl, RefExpect, safe_relative,
};
pub use error::{GitError, Result};
pub use failure::FailureKind;
pub use progress::{ProgressCount, ProgressPhase};
pub use types::{
    AuthMode, BlobHead, Branch, BranchInfo, BranchKind, Change, Commit, CommitFile, CommitFiles,
    CommitMessage, CommitStat, DiffKind, DiffLine, DiffTarget, EntryKind, FileDiff, GitIdentity,
    GitOp, GitVersion, GlobalOp, GraphLogQuery, Hunk, IndexEntry, LineKind, LogQuery, MergeCheck,
    MergeSegment, Oid, OpOutput, OperationKind, RebaseState, RefEntry, ReflogEntry, Remote,
    RemoteUrls, RepoPath, RewriteKind, RewriteStep, StashEntry, Status, StatusEntry, TagRef,
};
