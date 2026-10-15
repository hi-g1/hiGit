//! 끝나지 않은 작업 알아내기(D-081, D-082).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod common;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

use common::FakeGit;
use higit_core::view::{
    ActionInput, ErrorCode, ErrorView, OpenHow, OtherOperation, PathsInput, PendingWorkView,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
    std::fs::write(dir.join("a.txt"), "원래\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    repo
}

/// 기본 프로세스 조회는 "도는 git 없음"이다(테스트가 동시에 띄우는 git에 흔들리지 않게).
fn core(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn make_old(path: &Path) {
    std::fs::write(path, "").unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(120))
        .unwrap();
}

#[test]
fn clean_repo_has_nothing_pending() {
    let repo = repo();
    let core = core(repo.path());
    assert_eq!(
        core.pending_work(repo.path()).unwrap(),
        PendingWorkView::default()
    );
}

#[test]
fn old_lock_files_are_stale_and_fresh_ones_are_not() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    make_old(&dir.join(".git/index.lock"));
    make_old(&dir.join(".git/refs/heads/main.lock"));
    let mut locks = core.pending_work(dir).unwrap().stale_locks;
    locks.sort();
    assert_eq!(locks, vec!["index.lock", "refs/heads/main.lock"]);
    // 방금 생긴 잠금이 하나라도 있으면 무언가 작업 중이다.
    std::fs::write(dir.join(".git/HEAD.lock"), "").unwrap();
    assert!(core.pending_work(dir).unwrap().stale_locks.is_empty());
}

#[test]
fn leftover_parked_entries_are_counted() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "맡김\n").unwrap();
    git(dir, &["stash", "push", "-q", "-m", "higit parked main"]);
    std::fs::write(dir.join("a.txt"), "내 보관\n").unwrap();
    git(dir, &["stash", "push", "-q", "-m", "내 보관"]);
    std::fs::write(dir.join("a.txt"), "앱 보관\n").unwrap();
    git(dir, &["stash", "push", "-q", "-m", "higit parkedx"]);
    let core = core(dir);
    assert_eq!(core.pending_work(dir).unwrap().leftover_parked, 1);
}

#[test]
fn other_tools_operations_are_named() {
    let repo = repo();
    let dir = repo.path();
    let head = git(dir, &["rev-parse", "HEAD"]);
    std::fs::write(dir.join(".git/CHERRY_PICK_HEAD"), format!("{head}\n")).unwrap();
    let core = core(dir);
    assert_eq!(
        core.pending_work(dir).unwrap().other_operation,
        Some(OtherOperation::CherryPick)
    );
    std::fs::remove_file(dir.join(".git/CHERRY_PICK_HEAD")).unwrap();
    std::fs::create_dir_all(dir.join(".git/rebase-merge")).unwrap();
    assert_eq!(
        core.pending_work(dir).unwrap().other_operation,
        Some(OtherOperation::Rebase)
    );
}

#[test]
fn unopened_repo_is_refused() {
    let repo = repo();
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    let err = core.pending_work(repo.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
}

#[test]
fn locks_are_not_stale_while_git_or_the_app_runs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    make_old(&dir.path().join(".git/index.lock"));
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = Some(higit_core::RepoPath::new(dir.path()));
    let core = Arc::new(Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap());
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    assert_eq!(
        core.pending_work(dir.path()).unwrap().stale_locks,
        vec!["index.lock"]
    );

    // 시스템에 git이 돈다.
    git.git_running.store(true, Ordering::SeqCst);
    assert!(
        core.pending_work(dir.path())
            .unwrap()
            .stale_locks
            .is_empty()
    );
    git.git_running.store(false, Ordering::SeqCst);

    // 앱의 작업이 이 저장소에서 돈다.
    git.hold.store(true, Ordering::SeqCst);
    let first = {
        let core = core.clone();
        let path = dir.path().to_path_buf();
        std::thread::spawn(move || {
            core.execute(
                &path,
                &ActionInput::Stage(PathsInput {
                    paths: vec!["a.txt".into()],
                }),
                None,
            )
        })
    };
    while !git.entered.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        core.pending_work(dir.path())
            .unwrap()
            .stale_locks
            .is_empty()
    );
    git.hold.store(false, Ordering::SeqCst);
    first.join().unwrap().unwrap();
}
