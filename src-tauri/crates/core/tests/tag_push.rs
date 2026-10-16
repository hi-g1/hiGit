//! 태그 올리기(`tag-push`, D-131)를 로컬 bare 원격과 실제 `Core`로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ActionInput, ActionResult, ErrorCode, ErrorView, OpenHow, TagPushInput};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
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

fn commit(dir: &Path, file: &str) {
    std::fs::write(dir.join(file), file).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn push_tag(core: &Core, dir: &Path, name: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::TagPush(TagPushInput { name: name.into() }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn remote_tags(remote: &Path) -> String {
    git(
        remote,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(objectname)",
            "refs/tags/",
        ],
    )
}

#[test]
fn pushes_one_tag_without_confirmation_and_never_moves_it() {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let mine = tempfile::tempdir().unwrap();
    let dir = mine.path();
    git(dir, &["init", "-q", "-b", "main"]);
    commit(dir, "a.txt");
    git(
        dir,
        &["remote", "add", "origin", &remote.path().to_string_lossy()],
    );
    git(dir, &["push", "-q", "-u", "origin", "main"]);
    git(dir, &["tag", "v1"]);
    git(dir, &["tag", "따로"]);
    let core = core(dir);
    // 확인 토큰 없이 된다.
    assert_eq!(
        push_tag(&core, dir, " v1 ").unwrap(),
        ActionResult::TagPushed { name: "v1".into() }
    );
    let first = git(dir, &["rev-parse", "v1"]);
    assert_eq!(remote_tags(remote.path()), format!("v1 {first}"));
    // 같은 버전이면 다시 올려도 된다.
    push_tag(&core, dir, "v1").unwrap();
    // 같은 이름을 다른 버전으로 옮겼으면 원격이 거절하고 원격은 그대로다.
    commit(dir, "b.txt");
    git(dir, &["tag", "-f", "v1"]);
    assert_eq!(
        code(&push_tag(&core, dir, "v1").unwrap_err()),
        ErrorCode::RemoteRefExists
    );
    assert_eq!(remote_tags(remote.path()), format!("v1 {first}"));
    // 없는 태그, 이름 규칙에 맞지 않는 이름.
    for bad in ["없음", "-x", "a b", ""] {
        assert_eq!(
            code(&push_tag(&core, dir, bad).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad}"
        );
    }
}

#[test]
fn needs_a_remote() {
    let mine = tempfile::tempdir().unwrap();
    let dir = mine.path();
    git(dir, &["init", "-q", "-b", "main"]);
    commit(dir, "a.txt");
    git(dir, &["tag", "v1"]);
    let core = core(dir);
    assert_eq!(
        code(&push_tag(&core, dir, "v1").unwrap_err()),
        ErrorCode::NoRemote
    );
}
