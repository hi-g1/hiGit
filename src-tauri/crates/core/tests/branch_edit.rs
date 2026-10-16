//! 브랜치 지우기·이름 바꾸기를 실제 git과 실행기로 시험한다(계획 008 E17, D-103).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, BranchNameInput, BranchRenameInput, ErrorCode, ErrorView, OpenHow,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
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

fn try_git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
}

/// main(처음) + 기능(한 버전 더). 지금은 main.
fn repo() -> (tempfile::TempDir, String) {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path().join("저장소");
    std::fs::create_dir(&dir).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
    ] {
        git(&dir, &["config", k, v]);
    }
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "처음"]);
    git(&dir, &["switch", "-q", "-c", "기능"]);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "기능"]);
    let tip = git(&dir, &["rev-parse", "HEAD"]);
    git(&dir, &["switch", "-q", "main"]);
    (repo, tip)
}

fn dir_of(t: &tempfile::TempDir) -> std::path::PathBuf {
    t.path().join("저장소")
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn delete(core: &Core, dir: &Path, name: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::BranchDelete(BranchNameInput { name: name.into() }),
        None,
    )
}

fn rename(core: &Core, dir: &Path, from: &str, to: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::BranchRename(BranchRenameInput {
            from: from.into(),
            to: to.into(),
        }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

/// 로컬 bare 원격에 올리고 upstream을 건다.
fn remote(t: &tempfile::TempDir, dir: &Path, name: &str) -> std::path::PathBuf {
    let bare = t.path().join(format!("{name}.git"));
    git(t.path(), &["init", "-q", "--bare", &bare.to_string_lossy()]);
    git(dir, &["remote", "add", name, &bare.to_string_lossy()]);
    bare
}

#[test]
fn delete_backs_up_the_tip_and_undo_reconnects_upstream() {
    let (t, tip) = repo();
    let dir = dir_of(&t);
    let bare = remote(&t, &dir, "origin");
    git(&dir, &["push", "-q", "-u", "origin", "기능"]);
    git(&dir, &["push", "-q", "origin", "main"]);
    // 기능에 맡긴 파일이 있다.
    git(&dir, &["update-ref", "refs/higit/parked/기능", &tip]);
    let core = core(&dir);
    assert_eq!(
        delete(&core, &dir, "기능").unwrap(),
        ActionResult::BranchDeleted {
            name: "기능".into()
        }
    );
    assert_eq!(
        try_git(&dir, &["rev-parse", "--verify", "-q", "refs/heads/기능"]),
        None
    );
    assert_eq!(try_git(&dir, &["config", "branch.기능.remote"]), None);
    assert_eq!(
        try_git(
            &dir,
            &["rev-parse", "--verify", "-q", "refs/higit/parked/기능"]
        ),
        None
    );
    let backups = git(
        &dir,
        &[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/higit/backup/",
        ],
    );
    assert!(
        backups.contains("branch-delete-") && backups.contains("parked-"),
        "{backups}"
    );
    // 원격 브랜치는 그대로.
    assert_eq!(git(&bare, &["rev-parse", "기능"]), tip);
    assert_eq!(git(&dir, &["rev-parse", "origin/기능"]), tip);
    let items = core.recovery(&dir).unwrap().items;
    assert!(items.iter().any(|i| i.label.contains("branch-delete")));

    core.undo(&dir).unwrap();
    assert_eq!(git(&dir, &["rev-parse", "refs/heads/기능"]), tip);
    assert_eq!(
        git(&dir, &["rev-parse", "--abbrev-ref", "기능@{upstream}"]),
        "origin/기능"
    );
}

#[test]
fn undo_does_not_guess_between_two_remotes() {
    let (t, tip) = repo();
    let dir = dir_of(&t);
    remote(&t, &dir, "origin");
    remote(&t, &dir, "up");
    git(&dir, &["push", "-q", "-u", "origin", "기능"]);
    git(&dir, &["push", "-q", "up", "기능"]);
    let core = core(&dir);
    delete(&core, &dir, "기능").unwrap();
    core.undo(&dir).unwrap();
    assert_eq!(git(&dir, &["rev-parse", "refs/heads/기능"]), tip);
    assert_eq!(
        try_git(&dir, &["rev-parse", "--abbrev-ref", "기능@{upstream}"]),
        None
    );
}

#[test]
fn protected_branches_and_operations_are_refused() {
    let (t, _) = repo();
    let dir = dir_of(&t);
    let core = core(&dir);
    // 지금 브랜치, 기본 브랜치(원격이 없으면 로컬 main).
    assert_eq!(
        code(&delete(&core, &dir, "main").unwrap_err()),
        ErrorCode::BranchProtected
    );
    git(&dir, &["switch", "-q", "기능"]);
    assert_eq!(
        code(&delete(&core, &dir, "기능").unwrap_err()),
        ErrorCode::BranchProtected
    );
    assert_eq!(
        code(&delete(&core, &dir, "main").unwrap_err()),
        ErrorCode::BranchProtected
    );
    git(&dir, &["switch", "-q", "main"]);
    // 다른 작업 폴더가 쓰는 브랜치.
    git(&dir, &["branch", "옆"]);
    let wt = t.path().join("옆 작업");
    git(
        &dir,
        &["worktree", "add", "-q", &wt.to_string_lossy(), "옆"],
    );
    assert_eq!(
        code(&delete(&core, &dir, "옆").unwrap_err()),
        ErrorCode::BranchProtected
    );
    // 없는 브랜치, 원격 브랜치 이름.
    assert_eq!(
        code(&delete(&core, &dir, "없음").unwrap_err()),
        ErrorCode::InvalidInput
    );
    // 합치는 중.
    std::fs::write(dir.join("b.txt"), "겹침").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "겹침"]);
    let _ = try_git(&dir, &["merge", "기능"]);
    assert_eq!(
        code(&delete(&core, &dir, "기능").unwrap_err()),
        ErrorCode::OperationInProgress
    );
    assert_eq!(
        code(&rename(&core, &dir, "기능", "새것").unwrap_err()),
        ErrorCode::OperationInProgress
    );
}

#[test]
fn rename_moves_parked_files_and_cuts_upstream_then_undo() {
    let (t, tip) = repo();
    let dir = dir_of(&t);
    let bare = remote(&t, &dir, "origin");
    git(&dir, &["push", "-q", "origin", "main"]);
    git(&dir, &["switch", "-q", "기능"]);
    git(&dir, &["push", "-q", "-u", "origin", "기능"]);
    git(&dir, &["update-ref", "refs/higit/parked/기능", &tip]);
    let core = core(&dir);
    assert_eq!(
        rename(&core, &dir, "기능", "새것/하나").unwrap(),
        ActionResult::BranchRenamed {
            from: "기능".into(),
            to: "새것/하나".into()
        }
    );
    assert_eq!(git(&dir, &["symbolic-ref", "--short", "HEAD"]), "새것/하나");
    assert_eq!(
        try_git(&dir, &["rev-parse", "--abbrev-ref", "새것/하나@{upstream}"]),
        None
    );
    assert_eq!(
        git(&dir, &["rev-parse", "refs/higit/parked/새것%2F하나"]),
        tip
    );
    assert_eq!(
        try_git(
            &dir,
            &["rev-parse", "--verify", "-q", "refs/higit/parked/기능"]
        ),
        None
    );
    // 올리면 새 이름으로 올라간다.
    core.execute(&dir, &ActionInput::Push, None).unwrap();
    assert_eq!(git(&bare, &["rev-parse", "새것/하나"]), tip);
    // Ctrl+Z로 원래 이름(upstream은 끊긴 채), 다시 하기는 없다.
    git(&dir, &["branch", "-q", "--unset-upstream"]);
    core.undo(&dir).unwrap();
    assert_eq!(git(&dir, &["symbolic-ref", "--short", "HEAD"]), "기능");
    assert_eq!(git(&dir, &["rev-parse", "refs/higit/parked/기능"]), tip);
    assert_eq!(
        code(&core.redo(&dir).unwrap_err()),
        ErrorCode::NothingToUndo
    );
}

#[test]
fn rename_checks_names_and_allows_case_only_changes() {
    let (t, _) = repo();
    let dir = dir_of(&t);
    git(&dir, &["branch", "다른"]);
    let core = core(&dir);
    assert_eq!(
        code(&rename(&core, &dir, "기능", "다른").unwrap_err()),
        ErrorCode::BranchExists
    );
    assert_eq!(
        code(&rename(&core, &dir, "기능", "다른".to_uppercase().as_str()).unwrap_err()),
        ErrorCode::BranchExists
    );
    assert_eq!(
        code(&rename(&core, &dir, "기능", "a..b").unwrap_err()),
        ErrorCode::InvalidInput
    );
    assert_eq!(
        code(&rename(&core, &dir, "기능", "기능").unwrap_err()),
        ErrorCode::NoChange
    );
    git(&dir, &["branch", "feature"]);
    rename(&core, &dir, "feature", "Feature").unwrap();
    assert!(
        git(&dir, &["branch", "--format=%(refname:short)"])
            .lines()
            .any(|l| l == "Feature")
    );
}
