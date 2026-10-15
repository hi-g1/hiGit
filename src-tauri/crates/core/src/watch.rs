//! 저장소 파일 변경 감시를 화면 이벤트로 바꾼다.

use std::path::Path;
use std::sync::Arc;

use git::RepoPath;
use watcher::{RepoWatcher, WatchEvent};

use crate::view::{RepoChanged, WatchStatus};
use crate::{Core, CoreError, Result};

/// 화면으로 보낼 감시 신호. app이 Tauri 이벤트로 내보낸다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchSignal {
    Changed(RepoChanged),
    Status(WatchStatus),
}

pub type WatchSink = Arc<dyn Fn(WatchSignal) + Send + Sync>;

impl Core {
    /// `repo`를 감시한다. 이전 감시는 멈춘다(한 번에 하나).
    /// 시작하지 못하면 `WatchUnavailable`. 화면은 창 포커스 때 새로 고침으로 대신한다.
    pub fn watch(&self, repo: &Path, sink: WatchSink) -> Result<()> {
        self.unwatch();
        let root = platform::normalize_repo_path(repo).map_err(CoreError::FolderNotFound)?;
        let excludes = self
            .git
            .excludes_file(&RepoPath::new(&root))?
            .into_iter()
            .collect();
        let name = root.to_string_lossy().into_owned();

        let event_sink = sink.clone();
        let event_repo = name.clone();
        let watcher = RepoWatcher::start(&root, excludes, move |event| match event {
            WatchEvent::Changed(c) => event_sink(WatchSignal::Changed(RepoChanged {
                repo: event_repo.clone(),
                worktree: c.worktree,
                git: c.git,
            })),
            WatchEvent::Stopped(reason) => event_sink(WatchSignal::Status(WatchStatus {
                repo: event_repo.clone(),
                active: false,
                reason: Some(reason),
            })),
        })
        .map_err(|e| CoreError::WatchUnavailable(e.to_string()))?;

        *self.watcher.lock().unwrap() = Some(watcher);
        sink(WatchSignal::Status(WatchStatus {
            repo: name,
            active: true,
            reason: None,
        }));
        Ok(())
    }

    pub fn unwatch(&self) {
        // 버리면 감시가 멈춘다(Windows에서 폴더 잠금도 풀린다).
        self.watcher.lock().unwrap().take();
    }
}
