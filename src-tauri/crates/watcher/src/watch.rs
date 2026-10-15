use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use crate::filter::{IgnoreRules, Kind, classify};

/// 여러 변경을 한 번에 모으는 시간(docs/architecture.md "성능 기준").
const DEBOUNCE: Duration = Duration::from_millis(300);

/// 한 번에 알리는 변경 묶음.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChangeSet {
    /// 작업 폴더 파일이 바뀌었다(바뀐 내용·비교를 다시 읽어야 함).
    pub worktree: bool,
    /// 저장·갈래·index가 바뀌었다(바뀐 내용·저장 기록을 다시 읽어야 함).
    pub git: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    Changed(ChangeSet),
    /// 감시가 멈췄다(감시 오류, 저장소 폴더 삭제 등). 이후 이벤트는 오지 않는다.
    Stopped(String),
}

#[derive(Debug, thiserror::Error)]
#[error("파일 변경을 감시할 수 없습니다: {0}")]
pub struct WatchError(String);

/// 저장소 폴더가 아직 있는지 확인하는 간격. 감시 중인 폴더를 지워도 알림을 주지 않는 OS가 있다.
const ROOT_CHECK: Duration = Duration::from_millis(500);

const ROOT_GONE: &str = "저장소 폴더가 사라졌습니다";

/// 살아 있는 동안 감시한다. 버리면(drop) 멈춘다.
pub struct RepoWatcher {
    _debouncer: Debouncer<notify::RecommendedWatcher, RecommendedCache>,
    alive: Arc<AtomicBool>,
}

impl Drop for RepoWatcher {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }
}

type Sink = Arc<dyn Fn(WatchEvent) + Send + Sync>;

/// 처음 한 번만 `Stopped`를 알린다.
fn stop(stopped: &AtomicBool, on_event: &Sink, reason: String) {
    if !stopped.swap(true, Ordering::SeqCst) {
        on_event(WatchEvent::Stopped(reason));
    }
}

impl RepoWatcher {
    /// `root`(저장소 최상위)를 감시한다. `extra_excludes`는 전역 제외 파일 같은 추가 무시 규칙 파일.
    pub fn start(
        root: &Path,
        extra_excludes: Vec<PathBuf>,
        on_event: impl Fn(WatchEvent) + Send + Sync + 'static,
    ) -> Result<Self, WatchError> {
        let on_event: Sink = Arc::new(on_event);
        let root = root.to_path_buf();
        let rules = Mutex::new(IgnoreRules::load(&root, &extra_excludes));
        let stopped = Arc::new(AtomicBool::new(false));
        let handler_root = root.clone();
        let handler_stopped = stopped.clone();
        let handler_sink = on_event.clone();

        let handler = move |result: DebounceEventResult| {
            if handler_stopped.load(Ordering::SeqCst) {
                return;
            }
            let events = match result {
                Ok(events) => events,
                Err(errors) => {
                    let msg = errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; ");
                    stop(&handler_stopped, &handler_sink, msg);
                    return;
                }
            };
            if !handler_root.exists() {
                stop(&handler_stopped, &handler_sink, ROOT_GONE.into());
                return;
            }

            let mut changes = ChangeSet::default();
            let mut reload = false;
            {
                let rules = rules.lock().unwrap();
                for event in &events {
                    // 읽기만 한 것(앱이 status를 읽을 때 등)은 알리지 않는다. 되먹임 고리를 막는다.
                    if matches!(event.event.kind, EventKind::Access(_)) {
                        continue;
                    }
                    for path in &event.event.paths {
                        match classify(&handler_root, &rules, path, path.is_dir()) {
                            Kind::Worktree => changes.worktree = true,
                            Kind::GitignoreChanged => {
                                changes.worktree = true;
                                reload = true;
                            }
                            Kind::Git => changes.git = true,
                            Kind::Skip => {}
                        }
                    }
                }
            }
            if reload {
                *rules.lock().unwrap() = IgnoreRules::load(&handler_root, &extra_excludes);
            }
            if changes != ChangeSet::default() {
                handler_sink(WatchEvent::Changed(changes));
            }
        };

        let mut debouncer =
            new_debouncer(DEBOUNCE, None, handler).map_err(|e| WatchError(e.to_string()))?;
        debouncer
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|e| WatchError(e.to_string()))?;

        // 저장소 폴더가 지워졌는지 따로 확인한다.
        let alive = Arc::new(AtomicBool::new(true));
        {
            let alive = alive.clone();
            std::thread::spawn(move || {
                while alive.load(Ordering::SeqCst) && !stopped.load(Ordering::SeqCst) {
                    std::thread::sleep(ROOT_CHECK);
                    if alive.load(Ordering::SeqCst) && !root.exists() {
                        stop(&stopped, &on_event, ROOT_GONE.into());
                    }
                }
            });
        }
        Ok(Self {
            _debouncer: debouncer,
            alive,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;
    use std::time::Instant;

    const WAIT: Duration = Duration::from_secs(5);

    fn start(root: &Path) -> (RepoWatcher, mpsc::Receiver<WatchEvent>) {
        let (tx, rx) = mpsc::channel();
        let watcher = RepoWatcher::start(root, Vec::new(), move |e| {
            let _ = tx.send(e);
        })
        .unwrap();
        // 감시가 자리 잡을 시간
        std::thread::sleep(Duration::from_millis(200));
        (watcher, rx)
    }

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/objects/ab")).unwrap();
        fs::create_dir_all(dir.path().join(".git/refs/heads")).unwrap();
        fs::write(dir.path().join(".gitignore"), "node_modules/\n").unwrap();
        fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        dir
    }

    /// 한동안 이벤트를 모아 합친다.
    fn collect(rx: &mpsc::Receiver<WatchEvent>, for_: Duration) -> Vec<WatchEvent> {
        let end = Instant::now() + for_;
        let mut out = Vec::new();
        while let Some(left) = end.checked_duration_since(Instant::now()) {
            match rx.recv_timeout(left) {
                Ok(e) => out.push(e),
                Err(_) => break,
            }
        }
        out
    }

    #[test]
    fn worktree_change_is_reported() {
        let dir = repo();
        let (_w, rx) = start(dir.path());
        fs::write(dir.path().join("한글 파일.txt"), "내용").unwrap();
        let event = rx.recv_timeout(WAIT).expect("이벤트가 와야 한다");
        assert_eq!(
            event,
            WatchEvent::Changed(ChangeSet {
                worktree: true,
                git: false
            })
        );
    }

    #[test]
    fn git_index_change_is_reported_but_objects_are_not() {
        let dir = repo();
        let (_w, rx) = start(dir.path());
        fs::write(dir.path().join(".git/objects/ab/cdef"), "x").unwrap();
        assert!(collect(&rx, Duration::from_millis(900)).is_empty());

        fs::write(dir.path().join(".git/index"), "x").unwrap();
        let event = rx.recv_timeout(WAIT).expect("이벤트가 와야 한다");
        assert_eq!(
            event,
            WatchEvent::Changed(ChangeSet {
                worktree: false,
                git: true
            })
        );
    }

    #[test]
    fn ignored_files_are_not_reported() {
        let dir = repo();
        let (_w, rx) = start(dir.path());
        fs::write(dir.path().join("node_modules/x.js"), "x").unwrap();
        assert!(collect(&rx, Duration::from_millis(900)).is_empty());
    }

    #[test]
    fn gitignore_change_reloads_rules() {
        let dir = repo();
        let (_w, rx) = start(dir.path());
        fs::write(dir.path().join(".gitignore"), "node_modules/\nsecret.txt\n").unwrap();
        assert!(matches!(rx.recv_timeout(WAIT), Ok(WatchEvent::Changed(_))));
        let _ = collect(&rx, Duration::from_millis(500));

        fs::write(dir.path().join("secret.txt"), "x").unwrap();
        assert!(collect(&rx, Duration::from_millis(900)).is_empty());
    }

    #[test]
    fn burst_of_changes_is_grouped() {
        let dir = repo();
        let (_w, rx) = start(dir.path());
        for i in 0..20 {
            fs::write(dir.path().join(format!("파일{i}.txt")), "x").unwrap();
        }
        let events = collect(&rx, Duration::from_millis(1500));
        assert!(!events.is_empty());
        assert!(events.len() <= 2, "{events:?}");
    }

    #[test]
    fn removing_repo_stops_watching() {
        let dir = repo();
        let root = dir.path().join("저장소");
        fs::create_dir(&root).unwrap();
        let (_w, rx) = start(&root);
        // Windows는 감시 중인 폴더를 지우지 못할 수 있다. 그때는 이 경우를 시험할 수 없다.
        if let Err(e) = fs::remove_dir_all(&root) {
            eprintln!("감시 중인 폴더를 지우지 못했습니다(이 OS에서는 시험 생략): {e}");
            return;
        }
        let events = collect(&rx, WAIT);
        assert!(
            events.iter().any(|e| matches!(e, WatchEvent::Stopped(_))),
            "{events:?}"
        );
    }
}
