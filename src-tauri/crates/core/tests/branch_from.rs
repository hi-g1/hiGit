//! 고른 버전에서 새 브랜치 만들기(`branch-create`의 `from`, 계획 008 E18). 실제 git으로.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, BranchCreateInput, ErrorCode, ErrorView, OpenHow,
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

fn repo() -> (tempfile::TempDir, String, String) {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
    ] {
        git(dir, &["config", k, v]);
    }
    std::fs::write(dir.join("a.txt"), "옛\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    let old = git(dir, &["rev-parse", "HEAD"]);
    std::fs::write(dir.join("a.txt"), "새\n").unwrap();
    git(dir, &["commit", "-q", "-am", "둘"]);
    let new = git(dir, &["rev-parse", "HEAD"]);
    (repo, old, new)
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn create(core: &Core, dir: &Path, name: &str, from: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::BranchCreate(BranchCreateInput {
            name: name.into(),
            from: Some(from.into()),
        }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

#[test]
fn from_an_old_version_then_undo_and_redo() {
    let (repo, old, new) = repo();
    let dir = repo.path();
    std::fs::write(dir.join("새 파일.txt"), "추적 안 함").unwrap();
    let core = core(dir);
    assert_eq!(
        create(&core, dir, "옛것", &old).unwrap(),
        ActionResult::BranchCreated {
            name: "옛것".into()
        }
    );
    assert_eq!(git(dir, &["symbolic-ref", "--short", "HEAD"]), "옛것");
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), old);
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "옛\n");
    assert!(dir.join("새 파일.txt").exists(), "새 파일은 따라간다");
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["symbolic-ref", "--short", "HEAD"]), "main");
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), new);
    assert!(git(dir, &["branch", "--list", "옛것"]).is_empty());
    core.redo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "refs/heads/옛것"]), old);
    assert_eq!(git(dir, &["symbolic-ref", "--short", "HEAD"]), "옛것");
}

#[test]
fn changed_files_and_unknown_hashes_are_refused() {
    let (repo, old, new) = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "고침\n").unwrap();
    assert_eq!(
        code(&create(&core, dir, "옛것", &old).unwrap_err()),
        ErrorCode::HasChanges
    );
    // 지금 버전에서 만들면 고친 파일이 따라간다(지금과 같다).
    create(&core, dir, "지금것", &new).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "고침\n"
    );
    git(dir, &["checkout", "-q", "--", "a.txt"]);
    // 어디에도 없는 버전(브랜치에서 안 닿고 작업 내역에도 없음).
    let tree = git(dir, &["rev-parse", "HEAD^{tree}"]);
    let dangling = git(dir, &["commit-tree", &tree, "-m", "떠돌이"]);
    assert_eq!(
        code(&create(&core, dir, "떠돌이", &dangling).unwrap_err()),
        ErrorCode::InvalidInput
    );
    assert_eq!(
        code(&create(&core, dir, "없음", &"0".repeat(40)).unwrap_err()),
        ErrorCode::InvalidInput
    );
}

#[test]
fn from_a_version_only_in_the_activity() {
    let (repo, _, new) = repo();
    let dir = repo.path();
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "버릴 것"]);
    let dropped = git(dir, &["rev-parse", "HEAD"]);
    git(dir, &["reset", "-q", "--hard", &new]);
    let core = core(dir);
    create(&core, dir, "살림", &dropped).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), dropped);
}

#[test]
fn old_input_without_from_still_reads() {
    let input: ActionInput = serde_json::from_value(
        serde_json::json!({"id": "branch-create", "input": {"name": "새것"}}),
    )
    .unwrap();
    assert_eq!(
        input,
        ActionInput::BranchCreate(BranchCreateInput {
            name: "새것".into(),
            from: None
        })
    );
}
