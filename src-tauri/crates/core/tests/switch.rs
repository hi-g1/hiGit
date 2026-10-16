//! 맡겨 두는 갈아타기와 새 브랜치, 그 되돌리기를 실제 git으로 시험한다(D-047, D-054).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, BranchCreateInput, ErrorCode, ErrorView, OpenHow, SwitchInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=테스트", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn commit_file(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

/// main에 a.txt, b.txt가 있고 브랜치 `other`가 있는 저장소.
fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    // CI 러너의 전역 core.autocrlf가 앱 쪽 git에 섞이지 않게 저장소에 고정한다.
    git(dir, &["config", "core.autocrlf", "false"]);
    commit_file(dir, "a.txt", "a\n");
    commit_file(dir, "b.txt", "b\n");
    git(dir, &["branch", "other"]);
    repo
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn switch(core: &Core, dir: &Path, branch: &str) -> ActionResult {
    core.execute(
        dir,
        &ActionInput::Switch(SwitchInput {
            branch: branch.into(),
            remote: None,
            dirty: None,
        }),
        None,
    )
    .unwrap()
}

fn branch(dir: &Path) -> String {
    git(dir, &["branch", "--show-current"]).trim().to_string()
}

fn status(dir: &Path) -> String {
    git(dir, &["status", "--porcelain"])
}

fn parked_ref(dir: &Path, name: &str) -> bool {
    std::process::Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/higit/parked/{name}"),
        ])
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn edits_stay_on_their_branch_and_come_back() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    std::fs::write(dir.join("새 파일.txt"), "새\n").unwrap();
    let stash_before = git(dir, &["stash", "list"]);
    let core = core(dir);

    let r = switch(&core, dir, "other");
    assert!(
        matches!(&r, ActionResult::Switched { parked_on: Some(b), .. } if b == "main"),
        "{r:?}"
    );
    assert_eq!(branch(dir), "other");
    assert_eq!(status(dir), "", "맡긴 뒤 작업 폴더는 깨끗하다");
    assert!(parked_ref(dir, "main"));
    assert_eq!(git(dir, &["stash", "list"]), stash_before);

    let r = switch(&core, dir, "main");
    assert!(
        matches!(r, ActionResult::Switched { unparked: true, .. }),
        "{r:?}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );
    assert!(dir.join("새 파일.txt").exists());
    assert!(!parked_ref(dir, "main"));
}

#[test]
fn unpark_after_unrelated_change_but_not_after_overlap() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    let core = core(dir);
    switch(&core, dir, "other");
    // 떠나 있는 동안 main에 다른 파일을 저장했다(터미널).
    git(dir, &["switch", "-q", "main"]);
    commit_file(dir, "c.txt", "c\n");
    git(dir, &["switch", "-q", "other"]);
    let r = switch(&core, dir, "main");
    assert!(
        matches!(r, ActionResult::Switched { unparked: true, .. }),
        "{r:?}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );

    // 이번에는 맡긴 줄과 같은 줄을 main에 저장한다.
    switch(&core, dir, "other");
    git(dir, &["switch", "-q", "main"]);
    commit_file(dir, "a.txt", "다른 고침\n");
    git(dir, &["switch", "-q", "other"]);
    let r = switch(&core, dir, "main");
    assert!(
        matches!(
            r,
            ActionResult::Switched {
                unparked: false,
                unpark_blocked: true,
                ..
            }
        ),
        "{r:?}"
    );
    assert_eq!(status(dir), "", "충돌 표시를 남기지 않는다");
    assert!(parked_ref(dir, "main"), "맡긴 것은 남는다");
}

#[test]
fn failed_switch_restores_edits() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    let core = core(dir);
    let err = core
        .execute(
            dir,
            &ActionInput::Switch(SwitchInput {
                branch: "없는-브랜치".into(),
                remote: None,
                dirty: None,
            }),
            None,
        )
        .unwrap_err();
    assert_ne!(ErrorView::from(&err).code, ErrorCode::ParkBlocked);
    assert_eq!(branch(dir), "main");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );
    assert!(!parked_ref(dir, "main"));
}

#[test]
fn failing_to_keep_the_parked_files_puts_them_back() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    std::fs::write(dir.join("새 파일.txt"), "새것\n").unwrap();
    // 맡긴 ref 자리에 잠금 파일이 있어 ref 쓰기가 실패한다.
    let lock = dir.join(".git/refs/higit/parked/main.lock");
    std::fs::create_dir_all(lock.parent().unwrap()).unwrap();
    std::fs::write(&lock, "").unwrap();
    let core = core(dir);
    let err = core
        .execute(
            dir,
            &ActionInput::Switch(SwitchInput {
                branch: "other".into(),
                remote: None,
                dirty: None,
            }),
            None,
        )
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::ParkBlocked);
    assert_eq!(branch(dir), "main");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );
    assert!(dir.join("새 파일.txt").exists());
    assert_eq!(
        git(dir, &["stash", "list"]),
        "",
        "사용자 stash 목록에 남기지 않는다"
    );
    assert!(!parked_ref(dir, "main"));
}

#[test]
fn arriving_with_edits_leaves_parked_files_without_an_error() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    let core = core(dir);
    switch(&core, dir, "other");
    assert!(parked_ref(dir, "main"));
    // 특정 버전을 보는 중에 다른 파일을 고친 채로 main으로 돌아온다(맡기지 않고 따라온다).
    git(dir, &["switch", "-q", "--detach"]);
    std::fs::write(dir.join("b.txt"), "b 고침\n").unwrap();
    let r = switch(&core, dir, "main");
    assert_eq!(
        r,
        ActionResult::Switched {
            branch: "main".into(),
            parked_on: None,
            unparked: false,
            unpark_blocked: true,
            files: 0,
            carried: false,
        }
    );
    assert!(parked_ref(dir, "main"), "맡긴 파일은 그대로 남는다");
    assert_eq!(
        std::fs::read_to_string(dir.join("b.txt")).unwrap(),
        "b 고침\n"
    );
}

#[test]
fn switching_to_a_name_git_rejects_is_refused() {
    let repo = repo();
    let dir = repo.path();
    let err = core(dir)
        .execute(
            dir,
            &ActionInput::Switch(SwitchInput {
                branch: "a..b".into(),
                remote: None,
                dirty: None,
            }),
            None,
        )
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
}

#[test]
fn switch_undo_and_redo() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    let core = core(dir);
    switch(&core, dir, "other");
    core.undo(dir).unwrap();
    assert_eq!(branch(dir), "main");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );
    core.redo(dir).unwrap();
    assert_eq!(branch(dir), "other");
    assert_eq!(status(dir), "");
}

#[test]
fn new_branch_carries_edits_and_undo_deletes_it() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    let core = core(dir);
    let create = |name: &str| {
        core.execute(
            dir,
            &ActionInput::BranchCreate(BranchCreateInput {
                name: name.into(),
                from: None,
            }),
            None,
        )
    };
    let r = create("기능/새것").unwrap();
    assert_eq!(
        r,
        ActionResult::BranchCreated {
            name: "기능/새것".into()
        }
    );
    assert_eq!(branch(dir), "기능/새것");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );

    core.undo(dir).unwrap();
    assert_eq!(branch(dir), "main");
    assert!(!git(dir, &["branch", "--list"]).contains("새것"));
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "a 고침\n"
    );
    core.redo(dir).unwrap();
    assert_eq!(branch(dir), "기능/새것");

    let err = create("OTHER").unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::BranchExists);
    let err = create("-x").unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
}

#[test]
fn no_switch_during_merge() {
    let repo = repo();
    let dir = repo.path();
    git(dir, &["switch", "-q", "other"]);
    commit_file(dir, "a.txt", "other 쪽\n");
    git(dir, &["switch", "-q", "main"]);
    commit_file(dir, "a.txt", "main 쪽\n");
    let _ = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@e.com",
            "merge",
            "other",
        ])
        .current_dir(dir)
        .output()
        .unwrap();
    let core = core(dir);
    let err = core
        .execute(
            dir,
            &ActionInput::Switch(SwitchInput {
                branch: "other".into(),
                remote: None,
                dirty: None,
            }),
            None,
        )
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::OperationInProgress);
}

#[test]
fn remote_branch_switch_and_undo_removes_created_local() {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let repo = repo();
    let dir = repo.path();
    let url = remote.path().to_string_lossy().into_owned();
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "origin", "main:원격기능"]);
    git(dir, &["fetch", "-q", "origin"]);
    let core = core(dir);
    core.execute(
        dir,
        &ActionInput::Switch(SwitchInput {
            branch: "원격기능".into(),
            remote: Some("origin".into()),
            dirty: None,
        }),
        None,
    )
    .unwrap();
    assert_eq!(branch(dir), "원격기능");
    core.undo(dir).unwrap();
    assert_eq!(branch(dir), "main");
    assert!(!git(dir, &["branch", "--list"]).contains("원격기능"));
}
