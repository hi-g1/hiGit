//! 저장소 파일 변경 감시. 다른 내부 크레이트를 모른다.
//!
//! 작업 폴더 변경과 `.git` 안의 의미 있는 변경(HEAD, index, refs …)만 알리고,
//! `.gitignore` 대상과 `.git/objects` 같은 내부 파일은 거른다.

#![forbid(unsafe_code)]

mod filter;
mod watch;

pub use watch::{ChangeSet, RepoWatcher, WatchError, WatchEvent};
