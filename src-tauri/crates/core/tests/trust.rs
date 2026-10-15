//! 저장소 신뢰 확인을 실제 git으로 시험한다(D-039, D-067).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::recent::RecentStore;
use higit_core::trust::{TrustDecision, TrustStore};
use higit_core::view::{
    ActionInput, BranchCreateInput, ErrorCode, ErrorView, OpenHow, ProjectOpenResult,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn core(data: &Path) -> Core {
    Core::with_stores(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>,
        RecentStore::in_memory(),
        TrustStore::open(data),
    )
    .unwrap()
}

fn repo_with_fsmonitor() -> (tempfile::TempDir, std::path::PathBuf) {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    let pwned = dir.join("pwned");
    git(
        dir,
        &[
            "config",
            "core.fsmonitor",
            &format!("touch '{}'", pwned.display()),
        ],
    );
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    (repo, pwned)
}

#[test]
fn executing_config_asks_before_opening() {
    let data = tempfile::tempdir().unwrap();
    let (repo, pwned) = repo_with_fsmonitor();
    let core = core(data.path());
    let r = core.open_project(repo.path(), OpenHow::Picked).unwrap();
    let ProjectOpenResult::NeedsTrust { keys, .. } = r else {
        panic!("{r:?}");
    };
    assert_eq!(keys, ["core.fsmonitor"]);
    // 아직 열리지 않았다.
    let err = core.queries().status(repo.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    assert!(!pwned.exists());
}

#[test]
fn disable_opens_without_running_config_and_is_remembered() {
    let data = tempfile::tempdir().unwrap();
    let (repo, pwned) = repo_with_fsmonitor();
    let first = core(data.path());
    let r = first
        .trust_project(repo.path(), TrustDecision::Disable)
        .unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
    first.queries().status(repo.path()).unwrap();
    assert!(!pwned.exists(), "끄고 열면 core.fsmonitor가 돌지 않는다");

    // 다음 실행(새 Core)에서도 같은 결정을 따른다.
    let second = core(data.path());
    let r = second.open_project(repo.path(), OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
    second.queries().status(repo.path()).unwrap();
    assert!(!pwned.exists());

    // 설정이 바뀌면 다시 묻는다.
    git(repo.path(), &["config", "core.sshCommand", "ssh -i x"]);
    let r = second.open_project(repo.path(), OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::NeedsTrust { .. }), "{r:?}");
}

#[test]
fn hooks_are_executing_too() {
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    std::fs::write(
        dir.join(".git/hooks/post-checkout"),
        "#!/bin/sh\ntouch pwned\n",
    )
    .unwrap();
    let core = core(data.path());
    let r = core.open_project(dir, OpenHow::Picked).unwrap();
    let ProjectOpenResult::NeedsTrust { keys, .. } = r else {
        panic!("{r:?}");
    };
    assert_eq!(keys, ["hooks/post-checkout"]);
}

/// 실행 비트를 준다. Windows git은 비트 없이도 hook을 돌리므로 실패해도 된다.
fn make_executable(path: &Path) {
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(path)
        .status();
}

#[test]
fn disable_really_turns_hooks_off() {
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "core.autocrlf", "false"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["commit", "-q", "--allow-empty", "-m", "첫 버전"]);
    let pwned = dir.join("pwned");
    let hook = dir.join(".git/hooks/post-checkout");
    std::fs::write(&hook, "#!/bin/sh\ntouch pwned\n").unwrap();
    make_executable(&hook);

    let core = core(data.path());
    let r = core.open_project(dir, OpenHow::Picked).unwrap();
    assert!(
        matches!(
            r,
            ProjectOpenResult::NeedsTrust {
                can_disable: true,
                ..
            }
        ),
        "{r:?}"
    );
    core.trust_project(dir, TrustDecision::Disable).unwrap();
    core.execute(
        dir,
        &ActionInput::BranchCreate(BranchCreateInput {
            name: "새-갈래".into(),
        }),
        None,
    )
    .unwrap();
    assert!(
        !pwned.exists(),
        "끄고 열면 post-checkout hook이 돌지 않는다"
    );
}

#[test]
fn keys_that_cannot_be_overridden_leave_only_trust() {
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    // `-c filter.a=b.clean=cat`은 키 `filter.a`로 읽혀 이 필터를 끄지 못한다.
    git(dir, &["config", "filter.a=b.clean", "touch pwned"]);
    let core = core(data.path());
    let r = core.open_project(dir, OpenHow::Picked).unwrap();
    let ProjectOpenResult::NeedsTrust { can_disable, .. } = r else {
        panic!("{r:?}");
    };
    assert!(!can_disable);
    let err = core.trust_project(dir, TrustDecision::Disable).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NeedsTrust);
    let err = core.queries().status(dir).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    // 믿고 열기는 된다.
    let r = core.trust_project(dir, TrustDecision::Trust).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
}

#[test]
fn hooks_of_the_main_folder_count_in_a_linked_worktree() {
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["commit", "-q", "--allow-empty", "-m", "첫 버전"]);
    let wt_parent = tempfile::tempdir().unwrap();
    let wt = wt_parent.path().join("wt");
    git(
        dir,
        &["worktree", "add", "-q", &wt.to_string_lossy(), "-b", "wt"],
    );
    std::fs::write(dir.join(".git/hooks/post-checkout"), "#!/bin/sh\n").unwrap();
    let r = core(data.path())
        .open_project(&wt, OpenHow::Picked)
        .unwrap();
    let ProjectOpenResult::NeedsTrust { keys, .. } = r else {
        panic!("{r:?}");
    };
    assert_eq!(keys, ["hooks/post-checkout"]);
}

#[test]
fn clean_repo_opens_without_asking() {
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "main"]);
    let r = core(data.path())
        .open_project(repo.path(), OpenHow::Picked)
        .unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
}

#[test]
fn start_screen_summary_waits_for_trust() {
    let data = tempfile::tempdir().unwrap();
    let (repo, pwned) = repo_with_fsmonitor();
    let recent = RecentStore::in_memory();
    recent.touch(&platform::normalize_repo_path(repo.path()).unwrap());
    let core = Core::with_stores(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>,
        recent,
        TrustStore::open(data.path()),
    )
    .unwrap();
    let err = core.queries().project_summary(repo.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NeedsTrust);
    assert!(!pwned.exists());
}
