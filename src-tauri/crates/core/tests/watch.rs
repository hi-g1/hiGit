//! 파일 변경 감시(실제 git과 실제 파일 시스템).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use git::CliBackend;
use higit_core::view::RepoChanged;
use higit_core::{Core, WatchSignal};

const WAIT: Duration = Duration::from_secs(5);

fn init_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let status = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(status.success());
    dir
}

fn core() -> Core {
    Core::with_backend(Arc::new(CliBackend::new("git", Arc::new(|_| {})))).unwrap()
}

fn watch(core: &Core, repo: &Path) -> mpsc::Receiver<WatchSignal> {
    let (tx, rx) = mpsc::channel();
    let tx = Mutex::new(tx);
    core.watch(
        repo,
        Arc::new(move |s| {
            let _ = tx.lock().unwrap().send(s);
        }),
    )
    .unwrap();
    // 첫 신호는 "감시 켜짐"
    assert!(matches!(
        rx.recv_timeout(WAIT),
        Ok(WatchSignal::Status(s)) if s.active
    ));
    std::thread::sleep(Duration::from_millis(200));
    rx
}

fn changes(rx: &mpsc::Receiver<WatchSignal>, for_: Duration) -> Vec<RepoChanged> {
    let end = Instant::now() + for_;
    let mut out = Vec::new();
    while let Some(left) = end.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(WatchSignal::Changed(c)) => out.push(c),
            Ok(_) => {}
            Err(_) => break,
        }
    }
    out
}

#[test]
fn file_change_reaches_sink() {
    let repo = init_repo();
    let core = core();
    let rx = watch(&core, repo.path());
    std::fs::write(repo.path().join("한글 파일.txt"), "내용").unwrap();
    let got = changes(&rx, Duration::from_secs(2));
    assert!(got.iter().any(|c| c.worktree), "{got:?}");
}

#[test]
fn reading_status_does_not_trigger_events() {
    let repo = init_repo();
    std::fs::write(repo.path().join("a.txt"), "a").unwrap();
    let core = core();
    let rx = watch(&core, repo.path());
    // 앱이 status를 읽어도 index를 쓰지 않아야(GIT_OPTIONAL_LOCKS=0) 되먹임 고리가 없다.
    for _ in 0..3 {
        core.queries().status(repo.path()).unwrap();
    }
    let got = changes(&rx, Duration::from_millis(1200));
    assert!(got.is_empty(), "{got:?}");
}

#[test]
fn watching_another_repo_stops_the_first() {
    let first = init_repo();
    let second = init_repo();
    let core = core();
    let rx_first = watch(&core, first.path());
    let _rx_second = watch(&core, second.path());
    std::fs::write(first.path().join("a.txt"), "a").unwrap();
    assert!(changes(&rx_first, Duration::from_millis(1200)).is_empty());
}
