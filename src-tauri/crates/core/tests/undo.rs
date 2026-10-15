//! 버전 저장 되돌리기·다시 하기를 실제 git으로 시험한다(D-042, D-065).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ActionInput, CommitInput, ErrorCode, ErrorView, OpenHow, PathsInput};
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
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.name", "테스트"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
    repo
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn stage(core: &Core, dir: &Path, file: &str) {
    core.execute(
        dir,
        &ActionInput::Stage(PathsInput {
            paths: vec![file.into()],
        }),
        None,
    )
    .unwrap();
}

fn commit(core: &Core, dir: &Path, message: &str, stage_all: bool) {
    core.execute(
        dir,
        &ActionInput::Commit(CommitInput {
            message: message.into(),
            stage_all,
        }),
        None,
    )
    .unwrap();
}

fn head(dir: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "-q", "HEAD"])
        .current_dir(dir)
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
}

fn code(err: higit_core::CoreError) -> ErrorCode {
    ErrorView::from(&err).code
}

#[test]
fn undo_and_redo_the_first_commit() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    let core = core(dir);
    stage(&core, dir, "a.txt");
    // 넣기는 경계 줄을 남기지만, 그 뒤의 저장은 되돌릴 수 있다.
    commit(&core, dir, "첫 버전", false);
    let saved = head(dir).unwrap();

    let r = core.undo(dir).unwrap();
    assert_eq!(r.message.as_deref(), Some("첫 버전"));
    assert_eq!(head(dir), None, "첫 버전 전으로");
    // 고친 파일은 넣은 상태 그대로 남는다.
    assert_eq!(
        git(dir, &["diff", "--cached", "--name-only"]).trim(),
        "a.txt"
    );

    core.redo(dir).unwrap();
    assert_eq!(
        head(dir).as_deref(),
        Some(saved.as_str()),
        "같은 버전이 되살아난다"
    );
    assert_eq!(code(core.redo(dir).unwrap_err()), ErrorCode::NothingToUndo);
}

#[test]
fn two_undos_in_a_row() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core, dir, "하나", true);
    let first = head(dir).unwrap();
    std::fs::write(dir.join("a.txt"), "2").unwrap();
    commit(&core, dir, "둘", true);
    assert_eq!(core.undo(dir).unwrap().message.as_deref(), Some("둘"));
    assert_eq!(head(dir).as_deref(), Some(first.as_str()));
    assert_eq!(core.undo(dir).unwrap().message.as_deref(), Some("하나"));
    assert_eq!(head(dir), None);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::NothingToUndo);
}

#[test]
fn stage_all_commit_undo_restores_index() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core, dir, "하나", true);
    std::fs::write(dir.join("a.txt"), "2").unwrap();
    // 넣은 파일 없이 "모두 넣고 저장"(D-062).
    commit(&core, dir, "둘", true);
    assert_eq!(git(dir, &["log", "--format=%s"]).lines().count(), 2);
    core.undo(dir).unwrap();
    assert_eq!(
        git(dir, &["diff", "--cached", "--name-only"]).trim(),
        "",
        "index도 저장 전으로"
    );
    assert_eq!(git(dir, &["diff", "--name-only"]).trim(), "a.txt");
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "2");
}

#[test]
fn outside_change_makes_undo_stale() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core, dir, "앱에서", true);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(dir, &["add", "b.txt"]);
    git(dir, &["commit", "-q", "-m", "터미널에서"]);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
}

#[test]
fn pushed_commit_is_not_undone() {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core, dir, "올릴 것", true);
    let url = remote.path().to_string_lossy().into_owned();
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "-u", "origin", "main"]);
    let before = head(dir);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoPushed);
    assert_eq!(head(dir), before);
}

#[test]
fn staging_after_commit_is_a_barrier_and_new_work_clears_redo() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core, dir, "하나", true);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    stage(&core, dir, "b.txt");
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::NothingToUndo);

    commit(&core, dir, "둘", false);
    core.undo(dir).unwrap();
    std::fs::write(dir.join("c.txt"), "c").unwrap();
    commit(&core, dir, "셋", true);
    assert_eq!(code(core.redo(dir).unwrap_err()), ErrorCode::NothingToUndo);
}

#[test]
fn journal_from_another_key_is_ignored() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core(dir), dir, "다른 키로 남긴 줄", true);
    // 새 Core는 다른 키를 쓴다(다른 컴퓨터나 받은 저장소의 journal).
    assert_eq!(
        code(core(dir).undo(dir).unwrap_err()),
        ErrorCode::NothingToUndo
    );
}
