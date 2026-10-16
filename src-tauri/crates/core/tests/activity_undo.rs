//! 작업 내역의 Ctrl+Z 표시와 "이 작업 전으로 되돌리기"를 실제 git으로 시험한다(D-204).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, CommitInput, ErrorCode, ErrorView, OpenHow, RecoveryItem, RecoverySource,
    RestoreInput, SwitchInput, UndoHint,
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
    std::fs::write(dir.join("a.txt"), "1\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    git(dir, &["branch", "other"]);
    repo
}

fn open(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn save(core: &Core, dir: &Path, file: &str) {
    std::fs::write(dir.join(file), file).unwrap();
    core.execute(
        dir,
        &ActionInput::Commit(CommitInput {
            message: format!("{file} 저장"),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
}

/// 앱 작업 줄을 최신 먼저.
fn journal(core: &Core, dir: &Path) -> Vec<RecoveryItem> {
    core.recovery(dir)
        .unwrap()
        .items
        .into_iter()
        .filter(|i| i.source == RecoverySource::Journal)
        .collect()
}

fn hints(core: &Core, dir: &Path) -> Vec<UndoHint> {
    journal(core, dir).iter().map(|i| i.undo).collect()
}

fn code(e: higit_core::CoreError) -> ErrorCode {
    ErrorView::from(&e).code
}

#[test]
fn hints_follow_the_undo_stack() {
    let repo = repo();
    let dir = repo.path();
    let core = open(dir);
    save(&core, dir, "b.txt");
    save(&core, dir, "c.txt");
    assert_eq!(hints(&core, dir), [UndoHint::Next, UndoHint::Earlier]);

    core.undo(dir).unwrap();
    assert_eq!(hints(&core, dir), [UndoHint::Done, UndoHint::Next]);

    // 새 작업이 생겨도 되돌린 줄은 되돌린 채다. Ctrl+Z가 되돌릴 줄은 늘 하나다.
    save(&core, dir, "d.txt");
    let now = hints(&core, dir);
    assert_eq!(
        now,
        [UndoHint::Next, UndoHint::Done, UndoHint::Earlier],
        "{now:?}"
    );
    assert_eq!(now.iter().filter(|h| **h == UndoHint::Next).count(), 1);

    // 다른 줄은 앱 작업이 아니다.
    assert!(
        core.recovery(dir)
            .unwrap()
            .items
            .iter()
            .filter(|i| i.source != RecoverySource::Journal)
            .all(|i| i.undo == UndoHint::None && i.before_oid.is_none())
    );
}

#[test]
fn an_outside_change_makes_every_row_unreachable() {
    let repo = repo();
    let dir = repo.path();
    let core = open(dir);
    save(&core, dir, "b.txt");
    save(&core, dir, "c.txt");
    std::fs::write(dir.join("밖.txt"), "밖").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "터미널 저장"]);
    assert_eq!(hints(&core, dir), [UndoHint::Never, UndoHint::Never]);
    assert_eq!(code(core.undo(dir).unwrap_err()), ErrorCode::UndoStale);
}

#[test]
fn a_pushed_version_is_undone_with_a_new_version() {
    let repo = repo();
    let dir = repo.path();
    let bare = tempfile::tempdir().unwrap();
    git(bare.path(), &["init", "-q", "--bare", "-b", "main"]);
    git(
        dir,
        &["remote", "add", "origin", &bare.path().to_string_lossy()],
    );
    let core = open(dir);
    save(&core, dir, "b.txt");
    save(&core, dir, "c.txt");
    git(dir, &["push", "-q", "-u", "origin", "main"]);
    // 맨 위는 되돌리는 새 버전을 만들고, 그 앞 줄은 더 되돌리지 않는다.
    assert_eq!(hints(&core, dir), [UndoHint::RevertCommit, UndoHint::Never]);

    // "이 작업 전으로 되돌리기"도 올린 버전을 기록에서 빼지 않는다.
    let rows = journal(&core, dir);
    let before = rows[0].before_oid.clone().unwrap();
    let err = core
        .execute(
            dir,
            &ActionInput::Restore(RestoreInput { oid: before }),
            None,
        )
        .unwrap_err();
    assert_eq!(code(err), ErrorCode::PublishedHistory);
}

#[test]
fn restoring_before_a_row_goes_back_to_its_starting_version() {
    let repo = repo();
    let dir = repo.path();
    let first = git(dir, &["rev-parse", "HEAD"]);
    let core = open(dir);
    save(&core, dir, "b.txt");
    let second = git(dir, &["rev-parse", "HEAD"]);
    save(&core, dir, "c.txt");
    let rows = journal(&core, dir);
    assert_eq!(rows[0].before_oid.as_deref(), Some(second.as_str()));
    assert_eq!(rows[1].before_oid.as_deref(), Some(first.as_str()));

    core.execute(
        dir,
        &ActionInput::Restore(RestoreInput { oid: first.clone() }),
        None,
    )
    .unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), first);

    // 브랜치를 바꾼 줄과 HEAD를 옮기지 않은 줄에는 없다.
    core.execute(
        dir,
        &ActionInput::Switch(SwitchInput {
            branch: "other".into(),
            remote: None,
            dirty: None,
        }),
        None,
    )
    .unwrap();
    let rows = journal(&core, dir);
    assert_eq!(rows[0].action.as_deref(), Some("switch"));
    assert_eq!(rows[0].before_oid, None);
    // 다른 브랜치에서 한 일의 줄은 지금 브랜치가 아니라 보이지 않는다.
    assert!(rows.iter().all(|r| r.before_oid.is_none()), "{rows:?}");
}
