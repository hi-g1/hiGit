//! 버전 저장 되돌리기·다시 하기를 실제 git으로 시험한다(D-042, D-065).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, CommitInput, ErrorCode, ErrorView, OpenHow, PathsInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "core.quotepath=off"])
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
    git(dir, &["config", "core.autocrlf", "false"]);
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

#[test]
fn failed_save_puts_back_what_it_staged() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "a\n").unwrap();
    let core = core(dir);
    // 연 뒤에 생긴 hook이 저장을 거부한다.
    let hook = dir.join(".git/hooks/pre-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&hook)
        .status();
    let err = core
        .execute(
            dir,
            &ActionInput::Commit(CommitInput {
                message: "저장".into(),
                stage_all: true,
            }),
            None,
        )
        .unwrap_err();
    assert_ne!(ErrorView::from(&err).code, ErrorCode::Internal);
    assert_eq!(head(dir), None);
    assert_eq!(git(dir, &["diff", "--cached", "--name-only"]), "");
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

/// 버전 둘을 앱으로 저장하고 원격에 올린 저장소. (원격, 저장소)
fn pushed_two() -> (tempfile::TempDir, tempfile::TempDir, Core) {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1\n").unwrap();
    commit(&core, dir, "하나", true);
    std::fs::write(dir.join("a.txt"), "2\n").unwrap();
    commit(&core, dir, "둘", true);
    let url = remote.path().to_string_lossy().into_owned();
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "-u", "origin", "main"]);
    (remote, repo, core)
}

#[test]
fn undoing_a_pushed_save_makes_a_reverting_version() {
    let (_remote, repo, core) = pushed_two();
    let dir = repo.path();
    let pushed = head(dir).unwrap();
    let result = core.undo(dir).unwrap();
    assert!(result.reverted);
    // 기록은 그대로이고 되돌리는 새 버전이 위에 생긴다(D-073).
    assert_ne!(head(dir).unwrap(), pushed);
    assert_eq!(git(dir, &["rev-parse", "HEAD~1"]).trim(), pushed);
    assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "1\n");
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    // 다시 하기도, 그 앞 줄 되돌리기도 없다.
    assert_eq!(code(core.redo(dir).unwrap_err()), ErrorCode::NothingToUndo);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::NothingToUndo);
}

#[test]
fn reverting_needs_nothing_staged_and_keeps_edits_safe() {
    let (_remote, repo, core) = pushed_two();
    let dir = repo.path();
    let pushed = head(dir);
    std::fs::write(dir.join("b.txt"), "b\n").unwrap();
    git(dir, &["add", "b.txt"]);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::StagedChanges);
    git(dir, &["reset", "-q"]);
    // 되돌릴 파일을 고쳐 두었으면 git이 거절하고 아무것도 바뀌지 않는다.
    std::fs::write(dir.join("a.txt"), "내 고침\n").unwrap();
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::WouldOverwrite);
    assert_eq!(head(dir), pushed);
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "내 고침\n"
    );
    assert!(!dir.join(".git/REVERT_HEAD").exists());
}

#[test]
fn a_hook_refusing_the_revert_leaves_nothing_behind() {
    let (_remote, repo, core) = pushed_two();
    let dir = repo.path();
    let pushed = head(dir);
    // 되돌리기(revert)는 prepare-commit-msg를 돈다(pre-commit·commit-msg는 돌지 않는다).
    let hook = dir.join(".git/hooks/prepare-commit-msg");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&hook)
        .status();
    assert!(core.undo(dir).is_err());
    assert_eq!(head(dir), pushed);
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    assert!(!dir.join(".git/REVERT_HEAD").exists());
}

fn staged(dir: &Path) -> String {
    git(dir, &["diff", "--cached", "--name-only"])
}

fn unstage(core: &Core, dir: &Path, file: &str) {
    core.execute(
        dir,
        &ActionInput::Unstage(PathsInput {
            paths: vec![file.into()],
        }),
        None,
    )
    .unwrap();
}

#[test]
fn staging_after_commit_undoes_only_the_staging_and_new_work_clears_redo() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    commit(&core, dir, "하나", true);
    let saved = head(dir);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    stage(&core, dir, "b.txt");
    // "저장 → 넣기 → Ctrl+Z"는 넣기만 되돌린다(D-072).
    core.undo(dir).unwrap();
    assert_eq!(staged(dir), "");
    assert_eq!(head(dir), saved);
    core.redo(dir).unwrap();
    assert_eq!(staged(dir), "b.txt\n");

    commit(&core, dir, "둘", false);
    core.undo(dir).unwrap();
    std::fs::write(dir.join("c.txt"), "c").unwrap();
    commit(&core, dir, "셋", true);
    assert_eq!(code(core.redo(dir).unwrap_err()), ErrorCode::NothingToUndo);
}

#[test]
fn stage_and_unstage_undo_one_step_at_a_time() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    commit(&core, dir, "첫 버전", true);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    std::fs::write(dir.join("c.txt"), "c").unwrap();
    stage(&core, dir, "b.txt");
    stage(&core, dir, "c.txt");
    unstage(&core, dir, "b.txt");
    assert_eq!(staged(dir), "c.txt\n");
    core.undo(dir).unwrap();
    assert_eq!(staged(dir), "b.txt\nc.txt\n");
    core.undo(dir).unwrap();
    assert_eq!(staged(dir), "b.txt\n");
    core.undo(dir).unwrap();
    assert_eq!(staged(dir), "");
    // 작업 폴더 파일은 그대로다.
    assert!(dir.join("b.txt").exists() && dir.join("c.txt").exists());
    // 이미 넣은 파일을 다시 넣으면 바뀐 것이 없어 줄이 생기지 않는다.
    stage(&core, dir, "b.txt");
    stage(&core, dir, "b.txt");
    core.undo(dir).unwrap();
    assert_eq!(staged(dir), "");
}

#[test]
fn staging_outside_the_app_makes_index_undo_stale() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    commit(&core, dir, "첫 버전", true);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    std::fs::write(dir.join("c.txt"), "c").unwrap();
    stage(&core, dir, "b.txt");
    git(dir, &["add", "c.txt"]);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
    assert_eq!(staged(dir), "b.txt\nc.txt\n");
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

fn ignore(core: &Core, dir: &Path, files: &[&str]) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::Ignore(PathsInput {
            paths: files.iter().map(|f| f.to_string()).collect(),
        }),
        None,
    )
}

#[test]
fn ignoring_new_files_and_undoing_it() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join("a.txt"), "a\n").unwrap();
    commit(&core, dir, "첫 버전", true);
    std::fs::write(dir.join(".env"), "x\n").unwrap();
    // `*`는 Windows 파일 이름에 쓸 수 없어 `[`로 이스케이프를 확인한다.
    std::fs::write(dir.join("#메모 [1].txt"), "y\n").unwrap();
    std::fs::write(dir.join("넣은 새 파일.key"), "z\n").unwrap();
    stage(&core, dir, "넣은 새 파일.key");
    let r = ignore(&core, dir, &[".env", "#메모 [1].txt", "넣은 새 파일.key"]).unwrap();
    assert_eq!(r, ActionResult::Ignored { count: 3 });
    // 세 파일 모두 바뀐 내용 목록에서 사라지고, 넣었던 것은 빠진다.
    assert_eq!(git(dir, &["status", "--porcelain"]), "?? .gitignore\n");
    assert_eq!(
        std::fs::read_to_string(dir.join(".gitignore")).unwrap(),
        "/.env\n/\\#메모 \\[1].txt\n/넣은 새 파일.key\n"
    );
    core.undo(dir).unwrap();
    assert!(
        !dir.join(".gitignore").exists(),
        "앱이 만든 빈 파일은 남기지 않는다"
    );
    assert_eq!(staged(dir), "넣은 새 파일.key\n");
    // 이미 버전에 든 파일은 받지 않는다.
    assert_eq!(
        code(ignore(&core, dir, &["a.txt"]).unwrap_err()),
        ErrorCode::InvalidInput
    );
}

#[test]
fn ignore_keeps_crlf_and_undo_stops_if_the_file_changed() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    std::fs::write(dir.join(".gitignore"), "node_modules/\r\ndist/").unwrap();
    git(dir, &["add", ".gitignore"]);
    git(dir, &["commit", "-q", "-m", "규칙"]);
    std::fs::write(dir.join("비밀.pem"), "k\n").unwrap();
    ignore(&core, dir, &["비밀.pem"]).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join(".gitignore")).unwrap(),
        "node_modules/\r\ndist/\r\n/비밀.pem\r\n"
    );
    // 그사이 사람이 규칙을 더 적었으면 되돌리지 않는다.
    let mut text = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
    text.push_str("*.log\r\n");
    std::fs::write(dir.join(".gitignore"), &text).unwrap();
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
    assert_eq!(
        std::fs::read_to_string(dir.join(".gitignore")).unwrap(),
        text
    );
}
