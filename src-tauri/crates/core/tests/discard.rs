//! 고친 것 버리기와 그 되돌리기를 실제 git으로 시험한다(D-074, D-196).

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
    core.execute(dir, &input(paths), None)
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
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
fn ctrl_z_brings_every_kind_of_discarded_file_back() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    std::fs::remove_file(dir.join("지울 파일.txt")).unwrap();
    std::fs::write(dir.join("새 파일.txt"), "새것\n").unwrap();
    std::fs::write(dir.join("넣은 새 파일.txt"), "넣음\n").unwrap();
    git(dir, &["add", "넣은 새 파일.txt"]);
    // 고르지 않은 파일은 버리기·되돌리기 내내 그대로다.
    std::fs::write(dir.join("그대로 둘 파일.txt"), "남김\n").unwrap();
    let before = git(dir, &["status", "--porcelain", "--untracked-files=all"]);
    let picked = [
        "고칠 파일.txt",
        "지울 파일.txt",
        "새 파일.txt",
        "넣은 새 파일.txt",
    ];
    // 확인 토큰 없이 된다.
    discard(&core, dir, &picked).unwrap();
    // `git`은 앞뒤 공백을 뗀다.
    assert_eq!(
        git(dir, &["status", "--porcelain"]),
        "M \"그대로 둘 파일.txt\""
    );

    let undone = core.undo(dir).unwrap();
    assert_eq!(undone.action, "discard");
    assert_eq!(
        git(dir, &["status", "--porcelain", "--untracked-files=all"]),
        before
    );
    assert_eq!(read(dir, "고칠 파일.txt"), "고침\n");
    assert!(!dir.join("지울 파일.txt").exists());
    assert_eq!(read(dir, "새 파일.txt"), "새것\n");
    assert_eq!(read(dir, "넣은 새 파일.txt"), "넣음\n");
    assert_eq!(read(dir, "그대로 둘 파일.txt"), "남김\n");
    assert_eq!(git(dir, &["stash", "list"]), "", "목록에 남기지 않는다");
    // 다시 하기는 없다(버리기를 다시 누른다).
    assert_eq!(code(core.redo(dir).unwrap_err()), ErrorCode::NothingToUndo);
}

#[test]
fn ctrl_z_brings_a_discarded_rename_back() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    git(dir, &["mv", "고칠 파일.txt", "새 이름.txt"]);
    let before = git(dir, &["status", "--porcelain"]);
    discard(&core, dir, &["새 이름.txt", "고칠 파일.txt"]).unwrap();
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["status", "--porcelain"]), before);
}

#[test]
fn only_the_discarded_file_comes_back_and_only_while_untouched() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "버린 것\n").unwrap();
    std::fs::write(dir.join("지울 파일.txt"), "남긴 것\n").unwrap();
    discard(&core, dir, &["고칠 파일.txt"]).unwrap();
    // 버린 뒤 다른 파일을 더 고쳐도 되돌릴 수 있다.
    std::fs::write(dir.join("그대로 둘 파일.txt"), "나중에 고침\n").unwrap();
    core.undo(dir).unwrap();
    assert_eq!(read(dir, "고칠 파일.txt"), "버린 것\n");
    assert_eq!(read(dir, "지울 파일.txt"), "남긴 것\n");
    assert_eq!(read(dir, "그대로 둘 파일.txt"), "나중에 고침\n");

    // 버린 파일을 그사이 다시 고쳤으면 되돌리지 않는다.
    discard(&core, dir, &["고칠 파일.txt"]).unwrap();
    std::fs::write(dir.join("고칠 파일.txt"), "다시 고침\n").unwrap();
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
    assert_eq!(read(dir, "고칠 파일.txt"), "다시 고침\n");
}

#[test]
fn other_staged_files_do_not_block_undo() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    // 다음 버전에 넣어 둔 파일이 있는 채로 다른 파일을 버리고 되돌린다.
    std::fs::write(dir.join("그대로 둘 파일.txt"), "넣어 둠\n").unwrap();
    git(dir, &["add", "그대로 둘 파일.txt"]);
    std::fs::write(dir.join("고칠 파일.txt"), "버린 것\n").unwrap();
    git(dir, &["add", "고칠 파일.txt"]);
    std::fs::write(dir.join("고칠 파일.txt"), "버린 것 더 고침\n").unwrap();
    let before = git(dir, &["status", "--porcelain"]);
    discard(&core, dir, &["고칠 파일.txt"]).unwrap();
    assert_eq!(
        git(dir, &["status", "--porcelain"]),
        "M  \"그대로 둘 파일.txt\""
    );
    // 버린 스냅샷에는 고른 파일만 들어 있다.
    let backup = git(
        dir,
        &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
    );
    assert_eq!(
        git(
            dir,
            &["diff", "--name-only", &format!("{backup}^1"), &backup]
        ),
        "고칠 파일.txt"
    );
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["status", "--porcelain"]), before);
    assert_eq!(read(dir, "고칠 파일.txt"), "버린 것 더 고침\n");
    assert_eq!(
        git(dir, &["show", ":고칠 파일.txt"]),
        "버린 것",
        "넣어 둔 모습도 돌아온다"
    );
}

#[test]
fn a_staged_deletion_alone_can_be_discarded_and_undone() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    git(dir, &["rm", "-q", "지울 파일.txt"]);
    discard(&core, dir, &["지울 파일.txt"]).unwrap();
    assert_eq!(read(dir, "지울 파일.txt"), "원래\n");
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["status", "--porcelain"]), "D  \"지울 파일.txt\"");
}

#[test]
fn a_new_file_written_in_the_same_place_blocks_undo() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("새 파일.txt"), "버린 새 파일\n").unwrap();
    discard(&core, dir, &["새 파일.txt"]).unwrap();
    std::fs::write(dir.join("새 파일.txt"), "다른 새 파일\n").unwrap();
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
    assert_eq!(read(dir, "새 파일.txt"), "다른 새 파일\n");
}

#[test]
fn a_missing_backup_blocks_undo() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    discard(&core, dir, &["고칠 파일.txt"]).unwrap();
    let backup = git(
        dir,
        &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
    );
    git(dir, &["update-ref", "-d", &backup]);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
    assert_eq!(read(dir, "고칠 파일.txt"), "원래\n");
}

#[test]
fn backup_failure_discards_nothing() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    // 백업 ref 자리에 파일이 있어 ref를 만들 수 없다.
    std::fs::create_dir_all(dir.join(".git/refs/higit")).unwrap();
    std::fs::write(dir.join(".git/refs/higit/backup"), "").unwrap();
    let err = discard(&core, dir, &["고칠 파일.txt"]).unwrap_err();
    assert_eq!(code(err), ErrorCode::BackupFailed);
    // 백업을 남기지 못하면 버리지 않는다(D-196).
    assert_eq!(read(dir, "고칠 파일.txt"), "고침\n");
    assert_eq!(git(dir, &["stash", "list"]), "");
    // 작업 목록에 줄이 없어 Ctrl+Z가 되돌릴 것이 없다.
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::NothingToUndo);
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
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::NothingToUndo);
    let empty = tempfile::tempdir().unwrap();
    git(empty.path(), &["init", "-q"]);
    std::fs::write(empty.path().join("a.txt"), "a").unwrap();
    let core = self::core(empty.path());
    assert_eq!(
        code(discard(&core, empty.path(), &["a.txt"]).unwrap_err()),
        ErrorCode::InvalidInput
    );
}
