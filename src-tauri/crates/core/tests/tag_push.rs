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

/// 원격 태그 지우기(`remote-tag-delete`, D-182). 확인 토큰이 필요하고 로컬 태그는 그대로다.
mod remote_tag_delete {
    use super::*;
    use higit_core::view::RemoteTagInput;

    fn setup() -> (tempfile::TempDir, tempfile::TempDir) {
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
        git(dir, &["tag", "-a", "-m", "주석", "v2"]);
        git(dir, &["push", "-q", "origin", "v1", "v2"]);
        (remote, mine)
    }

    fn input(dir: &Path, name: &str) -> ActionInput {
        ActionInput::RemoteTagDelete(RemoteTagInput {
            name: name.into(),
            expect: git(dir, &["rev-parse", &format!("refs/tags/{name}")]),
        })
    }

    fn confirmed(core: &Core, dir: &Path, input: &ActionInput) -> higit_core::Result<ActionResult> {
        let token = core.preview(dir, input).unwrap().token;
        core.execute(dir, input, Some(token))
    }

    #[test]
    fn deletes_only_the_remote_tag_after_confirmation() {
        let (remote, mine) = setup();
        let dir = mine.path();
        let core = core(dir);
        let err = core.execute(dir, &input(dir, "v1"), None).unwrap_err();
        assert_eq!(code(&err), ErrorCode::PreviewRequired);
        assert_eq!(
            confirmed(&core, dir, &input(dir, "v1")).unwrap(),
            ActionResult::RemoteTagDeleted { name: "v1".into() }
        );
        // 주석 태그도 태그 객체 값으로 지운다.
        confirmed(&core, dir, &input(dir, "v2")).unwrap();
        assert!(remote_tags(remote.path()).is_empty());
        // 내 컴퓨터의 태그는 그대로라 다시 올릴 수 있다.
        assert!(git(dir, &["tag", "--list"]).contains("v1"));
        push_tag(&core, dir, "v1").unwrap();
        assert!(remote_tags(remote.path()).contains("v1"));
    }

    #[test]
    fn refuses_when_the_remote_differs_or_is_gone() {
        let (remote, mine) = setup();
        let dir = mine.path();
        let core = core(dir);
        // 원격에 없음.
        git(remote.path(), &["tag", "-d", "v1"]);
        let err = confirmed(&core, dir, &input(dir, "v1")).unwrap_err();
        assert_eq!(code(&err), ErrorCode::RemoteRefMissing);
        // 원격의 같은 이름이 다른 버전.
        let other = tempfile::tempdir().unwrap();
        git(other.path(), &["init", "-q", "-b", "main"]);
        commit(other.path(), "x.txt");
        git(
            other.path(),
            &[
                "push",
                "-q",
                "-f",
                &remote.path().to_string_lossy(),
                "HEAD:refs/tags/v2",
            ],
        );
        let before = remote_tags(remote.path());
        let err = confirmed(&core, dir, &input(dir, "v2")).unwrap_err();
        assert_eq!(code(&err), ErrorCode::RemoteTagDiffers);
        assert_eq!(remote_tags(remote.path()), before);
        // 로컬 값과 다른 expect·이상한 이름은 원격에 닿기 전에 막힌다.
        let stale = ActionInput::RemoteTagDelete(RemoteTagInput {
            name: "v2".into(),
            expect: "0".repeat(40),
        });
        assert_eq!(
            code(&confirmed(&core, dir, &stale).unwrap_err()),
            ErrorCode::InvalidInput
        );
        for bad in ["-x", "a b"] {
            let odd = ActionInput::RemoteTagDelete(RemoteTagInput {
                name: bad.into(),
                expect: "0".repeat(40),
            });
            assert_eq!(
                code(&confirmed(&core, dir, &odd).unwrap_err()),
                ErrorCode::InvalidInput,
                "{bad}"
            );
        }
    }
}
