//! 원격 확인·가져오기·올리기·동기화를 로컬 bare 원격으로 시험한다(D-045, D-048).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, CommitInput, ErrorCode, ErrorView, FetchInput, OpenHow, SyncInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=민지", "-c", "user.email=m@example.com"])
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

struct Setup {
    _remote: tempfile::TempDir,
    url: String,
    mine: tempfile::TempDir,
    other: tempfile::TempDir,
}

fn setup() -> Setup {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let url = remote.path().to_string_lossy().into_owned();
    let mine = tempfile::tempdir().unwrap();
    git(mine.path(), &["init", "-q", "-b", "main"]);
    git(mine.path(), &["config", "core.autocrlf", "false"]);
    git(mine.path(), &["config", "user.name", "나"]);
    git(mine.path(), &["config", "user.email", "me@example.com"]);
    git(mine.path(), &["config", "commit.gpgsign", "false"]);
    commit_file(mine.path(), "a.txt", "줄 1\n줄 2\n줄 3\n줄 4\n줄 5\n");
    git(mine.path(), &["remote", "add", "origin", &url]);
    git(mine.path(), &["push", "-q", "-u", "origin", "main"]);
    let other = tempfile::tempdir().unwrap();
    git(other.path(), &["clone", "-q", &url, "."]);
    git(other.path(), &["config", "core.autocrlf", "false"]);
    Setup {
        _remote: remote,
        url,
        mine,
        other,
    }
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn sync(core: &Core, dir: &Path) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::Sync(SyncInput {
            locale: "ko".into(),
        }),
        None,
    )
}

fn others_push(s: &Setup, file: &str, text: &str) {
    commit_file(s.other.path(), file, text);
    git(s.other.path(), &["push", "-q", "origin", "main"]);
}

#[test]
fn receive_only_fast_forwards() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    let core = core(s.mine.path());
    let r = sync(&core, s.mine.path()).unwrap();
    assert_eq!(
        r,
        ActionResult::Synced {
            received: 1,
            sent: 0,
            merged: false,
            overlap: vec![]
        }
    );
    assert!(s.mine.path().join("b.txt").exists());
}

#[test]
fn send_only_pushes() {
    let s = setup();
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert!(
        matches!(
            r,
            ActionResult::Synced {
                sent: 1,
                received: 0,
                ..
            }
        ),
        "{r:?}"
    );
    git(s.other.path(), &["pull", "-q", "--ff-only"]);
    assert!(s.other.path().join("c.txt").exists());
}

#[test]
fn both_sides_different_files_merge_and_push() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert!(
        matches!(
            r,
            ActionResult::Synced {
                received: 1,
                merged: true,
                sent: 2,
                ..
            }
        ),
        "{r:?}"
    );
    let subject = git(s.mine.path(), &["log", "-1", "--format=%s"]);
    assert_eq!(subject.trim(), "민지의 버전 1개 가져와 합침");
    // 원격에도 올라갔다.
    git(s.other.path(), &["pull", "-q", "--ff-only"]);
    assert!(s.other.path().join("c.txt").exists());
}

#[test]
fn overlapping_change_does_not_start_merge() {
    let s = setup();
    others_push(&s, "a.txt", "줄 1\n남이 고침\n줄 3\n줄 4\n줄 5\n");
    commit_file(
        s.mine.path(),
        "a.txt",
        "줄 1\n내가 고침\n줄 3\n줄 4\n줄 5\n",
    );
    let before = git(s.mine.path(), &["rev-parse", "HEAD"]);
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert_eq!(
        r,
        ActionResult::Synced {
            received: 0,
            sent: 0,
            merged: false,
            overlap: vec!["a.txt".into()]
        }
    );
    assert_eq!(git(s.mine.path(), &["rev-parse", "HEAD"]), before);
    assert!(!s.mine.path().join(".git/MERGE_HEAD").exists());
    assert_eq!(git(s.mine.path(), &["status", "--porcelain"]), "");
}

#[test]
fn push_names_the_branch_and_ignores_repo_push_settings() {
    let s = setup();
    // 남이 올린 다른 브랜치. 내 쪽에는 같은 이름의 오래된 브랜치가 있다.
    git(s.mine.path(), &["branch", "old"]);
    git(s.other.path(), &["switch", "-q", "-c", "old"]);
    commit_file(s.other.path(), "o.txt", "남의 브랜치\n");
    git(s.other.path(), &["push", "-q", "origin", "old"]);
    let theirs = git(s.other.path(), &["rev-parse", "old"]);
    // 받은 저장소의 설정이 모든 브랜치를 강제로 올리게 해 둔다.
    git(
        s.mine.path(),
        &["config", "remote.origin.push", "+refs/heads/*:refs/heads/*"],
    );
    git(s.mine.path(), &["config", "push.default", "matching"]);
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    sync(&core(s.mine.path()), s.mine.path()).unwrap();
    let remote_old = git(Path::new(&s.url), &["rev-parse", "old"]);
    assert_eq!(remote_old, theirs, "다른 브랜치를 덮어쓰지 않는다");
    let remote_main = git(Path::new(&s.url), &["rev-parse", "main"]);
    assert_eq!(remote_main, git(s.mine.path(), &["rev-parse", "main"]));
}

#[test]
fn first_upload_sets_upstream() {
    let s = setup();
    git(s.mine.path(), &["switch", "-q", "-c", "기능"]);
    commit_file(s.mine.path(), "d.txt", "새 브랜치\n");
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert!(
        matches!(r, ActionResult::Synced { sent, .. } if sent >= 1),
        "{r:?}"
    );
    let up = git(s.mine.path(), &["rev-parse", "--abbrev-ref", "@{upstream}"]);
    assert_eq!(up.trim(), "origin/기능");
}

#[test]
fn no_remote_is_reported() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "main"]);
    commit_file(repo.path(), "a.txt", "a\n");
    let err = sync(&core(repo.path()), repo.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NoRemote);
}

#[test]
fn background_fetch_does_not_block_saving() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    let core = core(s.mine.path());
    core.execute(
        s.mine.path(),
        &ActionInput::Fetch(FetchInput { background: true }),
        None,
    )
    .unwrap();
    let st = git(s.mine.path(), &["status", "-sb"]);
    assert!(st.contains("behind 1"), "{st}");
    // FETCH_HEAD를 쓰지 않는다(감시 알림이 간격마다 나지 않게).
    assert!(!s.mine.path().join(".git/FETCH_HEAD").exists());
    std::fs::write(s.mine.path().join("c.txt"), "c").unwrap();
    core.execute(
        s.mine.path(),
        &ActionInput::Commit(CommitInput {
            message: "저장".into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
    let _ = s.url;
}

fn save_in_app(core: &Core, dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    core.execute(
        dir,
        &ActionInput::Commit(CommitInput {
            message: file.into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
}

#[test]
fn undoing_after_a_sync_says_it_was_uploaded() {
    let s = setup();
    let core = core(s.mine.path());
    save_in_app(&core, s.mine.path(), "c.txt", "내 것\n");
    sync(&core, s.mine.path()).unwrap();
    let err = core.undo(s.mine.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::UndoPushed);
}

#[test]
fn a_sync_that_stopped_does_not_block_undoing_the_save() {
    let s = setup();
    others_push(&s, "a.txt", "줄 1\n남이 고침\n줄 3\n줄 4\n줄 5\n");
    let core = core(s.mine.path());
    let before = git(s.mine.path(), &["rev-parse", "HEAD"]);
    save_in_app(
        &core,
        s.mine.path(),
        "a.txt",
        "줄 1\n내가 고침\n줄 3\n줄 4\n줄 5\n",
    );
    let r = sync(&core, s.mine.path()).unwrap();
    assert!(matches!(r, ActionResult::Synced { ref overlap, .. } if !overlap.is_empty()));
    // 겹쳐서 멈춘 동기화는 아무것도 바꾸지 않았으니, 되돌리기는 그 앞의 버전 저장을 되돌린다.
    core.undo(s.mine.path()).unwrap();
    assert_eq!(git(s.mine.path(), &["rev-parse", "HEAD"]), before);
}
