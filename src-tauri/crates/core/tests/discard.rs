//! 고친 것 버리기를 실제 git으로 시험한다(D-043, D-074).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ActionInput, ActionResult, ErrorCode, ErrorView, OpenHow, PathsInput};
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
    for f in ["고칠 파일.txt", "지울 파일.txt", "그대로 둘 파일.txt"] {
        std::fs::write(dir.join(f), "원래\n").unwrap();
    }
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    repo
}

fn core(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn input(paths: &[&str]) -> ActionInput {
    ActionInput::Discard(PathsInput {
        paths: paths.iter().map(|p| p.to_string()).collect(),
    })
}

fn discard(core: &Core, dir: &Path, paths: &[&str]) -> Result<ActionResult, higit_core::CoreError> {
    let preview = core.preview(dir, &input(paths)).unwrap();
    core.execute(dir, &input(paths), Some(preview.token))
}

fn code(e: higit_core::CoreError) -> ErrorCode {
    ErrorView::from(&e).code
}

#[test]
fn discarding_chosen_files_keeps_a_backup() {
    let repo = repo();
    let dir = repo.path();
    // 사용자의 stash 목록에 이미 항목이 있다.
    std::fs::write(dir.join("그대로 둘 파일.txt"), "사용자 보관\n").unwrap();
    git(dir, &["stash", "push", "-q", "-m", "내 보관"]);
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    std::fs::remove_file(dir.join("지울 파일.txt")).unwrap();
    std::fs::write(dir.join("새 파일.txt"), "새것\n").unwrap();
    std::fs::write(dir.join("넣은 새 파일.txt"), "넣음\n").unwrap();
    git(dir, &["add", "넣은 새 파일.txt"]);
    std::fs::write(dir.join("그대로 둘 파일.txt"), "남김\n").unwrap();
    let r = discard(
        &core,
        dir,
        &[
            "고칠 파일.txt",
            "지울 파일.txt",
            "새 파일.txt",
            "넣은 새 파일.txt",
        ],
    )
    .unwrap();
    assert_eq!(r, ActionResult::Discarded { count: 4 });
    assert_eq!(
        std::fs::read_to_string(dir.join("고칠 파일.txt")).unwrap(),
        "원래\n"
    );
    assert!(dir.join("지울 파일.txt").exists());
    assert!(!dir.join("새 파일.txt").exists());
    assert!(!dir.join("넣은 새 파일.txt").exists());
    // 고르지 않은 파일은 그대로 고친 채 남는다.
    let status = git(dir, &["status", "--porcelain"]);
    assert_eq!(status.lines().count(), 1, "{status}");
    assert!(status.contains("그대로 둘 파일.txt"), "{status}");
    assert_eq!(
        git(dir, &["stash", "list", "--format=%gs"]),
        "On main: 내 보관"
    );
    // 버린 내용은 백업 ref에 stash 모양으로 남는다.
    let backup = git(
        dir,
        &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
    );
    assert!(backup.contains("discard-main"), "{backup}");
    assert_eq!(
        git(dir, &["show", &format!("{backup}:고칠 파일.txt")]),
        "고침"
    );
}

#[test]
fn discard_needs_a_confirmation_token_for_the_same_input() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    let err = core
        .execute(dir, &input(&["고칠 파일.txt"]), None)
        .unwrap_err();
    assert_eq!(code(err), ErrorCode::PreviewRequired);
    let preview = core.preview(dir, &input(&["고칠 파일.txt"])).unwrap();
    let err = core
        .execute(dir, &input(&["지울 파일.txt"]), Some(preview.token.clone()))
        .unwrap_err();
    assert_eq!(code(err), ErrorCode::PreviewMismatch);
    assert_eq!(
        std::fs::read_to_string(dir.join("고칠 파일.txt")).unwrap(),
        "고침\n"
    );
    // 토큰은 한 번만 쓴다.
    core.execute(dir, &input(&["고칠 파일.txt"]), Some(preview.token.clone()))
        .unwrap_err();
}

#[test]
fn backup_failure_leaves_the_changes_in_the_stash_list() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    // 백업 ref 자리에 파일이 있어 ref를 만들 수 없다.
    std::fs::create_dir_all(dir.join(".git/refs/higit")).unwrap();
    std::fs::write(dir.join(".git/refs/higit/backup"), "").unwrap();
    let err = discard(&core, dir, &["고칠 파일.txt"]).unwrap_err();
    assert_eq!(code(err), ErrorCode::BackupFailed);
    assert_eq!(
        git(dir, &["stash", "list", "--format=%gs"]),
        "On main: higit discard main"
    );
}

#[test]
fn nothing_to_discard_and_no_version_yet() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    assert_eq!(
        discard(&core, dir, &["고칠 파일.txt"]).unwrap(),
        ActionResult::Discarded { count: 0 }
    );
    let empty = tempfile::tempdir().unwrap();
    git(empty.path(), &["init", "-q"]);
    std::fs::write(empty.path().join("a.txt"), "a").unwrap();
    let core = self::core(empty.path());
    assert_eq!(
        code(discard(&core, empty.path(), &["a.txt"]).unwrap_err()),
        ErrorCode::InvalidInput
    );
}
