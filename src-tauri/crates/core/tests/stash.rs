//! 임시 보관과 꺼내기를 실제 git으로 시험한다(D-076).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ErrorCode, ErrorView, OpenHow, TakeOutInput, TakeOutSource,
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
    std::fs::write(dir.join("고칠 파일.txt"), "원래\n").unwrap();
    std::fs::write(dir.join("지울 파일.txt"), "지움\n").unwrap();
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

fn edit(dir: &Path) {
    std::fs::write(dir.join("고칠 파일.txt"), "고침\n").unwrap();
    std::fs::write(dir.join("새 파일.txt"), "새것\n").unwrap();
    std::fs::remove_file(dir.join("지울 파일.txt")).unwrap();
}

fn take_out(core: &Core, dir: &Path, oid: &str) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::TakeOut(TakeOutInput {
            source: TakeOutSource::Stash { oid: oid.into() },
        }),
        None,
    )
}

fn code(e: higit_core::CoreError) -> ErrorCode {
    ErrorView::from(&e).code
}

#[test]
fn stash_and_take_out_keep_the_users_own_stash() {
    let repo = repo();
    let dir = repo.path();
    // 사용자가 터미널에서 만든 stash가 이미 있다.
    std::fs::write(dir.join("고칠 파일.txt"), "사용자 것\n").unwrap();
    git(dir, &["stash", "push", "-q", "-m", "내 보관"]);
    let core = core(dir);
    edit(dir);
    assert_eq!(
        core.execute(dir, &ActionInput::Stash, None).unwrap(),
        ActionResult::Stashed
    );
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    let list = core.queries().stashes(dir).unwrap().items;
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].message, "higit stash main");
    assert_eq!(list[0].branch.as_deref(), Some("main"));
    assert_eq!(list[0].files, 3);
    assert_eq!(list[1].message, "내 보관");
    assert_eq!(list[1].branch, None);
    take_out(&core, dir, &list[0].oid).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("고칠 파일.txt")).unwrap(),
        "고침\n"
    );
    assert!(dir.join("새 파일.txt").exists());
    assert!(!dir.join("지울 파일.txt").exists());
    let list = core.queries().stashes(dir).unwrap().items;
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].message, "내 보관");
}

#[test]
fn take_out_needs_a_clean_folder_and_a_listed_item() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    edit(dir);
    core.execute(dir, &ActionInput::Stash, None).unwrap();
    let oid = core.queries().stashes(dir).unwrap().items[0].oid.clone();
    std::fs::write(dir.join("고칠 파일.txt"), "또 고침\n").unwrap();
    assert_eq!(
        code(take_out(&core, dir, &oid).unwrap_err()),
        ErrorCode::WouldOverwrite
    );
    assert_eq!(core.queries().stashes(dir).unwrap().items.len(), 1);
    let head = git(dir, &["rev-parse", "HEAD"]);
    assert_eq!(
        code(take_out(&core, dir, &head).unwrap_err()),
        ErrorCode::InvalidInput
    );
    assert_eq!(
        code(take_out(&core, dir, "-x").unwrap_err()),
        ErrorCode::InvalidInput
    );
}

#[test]
fn undoing_a_stash_takes_it_back_out() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    edit(dir);
    core.execute(dir, &ActionInput::Stash, None).unwrap();
    core.undo(dir).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("고칠 파일.txt")).unwrap(),
        "고침\n"
    );
    assert!(core.queries().stashes(dir).unwrap().items.is_empty());
    // 보관할 것이 없으면 하지 않는다.
    git(dir, &["checkout", "-q", "--", "."]);
    git(dir, &["clean", "-q", "-f"]);
    assert_eq!(
        code(core.execute(dir, &ActionInput::Stash, None).unwrap_err()),
        ErrorCode::InvalidInput
    );
}

#[test]
fn undo_is_stale_after_popping_in_a_terminal() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    edit(dir);
    core.execute(dir, &ActionInput::Stash, None).unwrap();
    git(dir, &["stash", "pop", "-q"]);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
}

#[test]
fn parked_entries_are_not_listed() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("고칠 파일.txt"), "맡김\n").unwrap();
    git(dir, &["stash", "push", "-q", "-m", "higit parked 기능"]);
    let core = core(dir);
    assert!(core.queries().stashes(dir).unwrap().items.is_empty());
}
