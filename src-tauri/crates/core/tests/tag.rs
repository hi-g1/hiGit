//! 태그 만들기·지우기를 실제 git과 실행기로 시험한다(계획 008 E16, D-104).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ErrorCode, ErrorView, OpenHow, TagDeleteInput, TagInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args([
            "-c",
            "core.autocrlf=false",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "tag.gpgSign=false",
        ])
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
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    let first = git(dir, &["rev-parse", "HEAD"]);
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "둘"]);
    let second = git(dir, &["rev-parse", "HEAD"]);
    (repo, first, second)
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn tag(core: &Core, dir: &Path, oid: &str, name: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::Tag(TagInput {
            oid: oid.into(),
            name: name.into(),
        }),
        None,
    )
}

fn untag(core: &Core, dir: &Path, name: &str, oid: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::TagDelete(TagDeleteInput {
            name: name.into(),
            oid: oid.into(),
        }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn tags(dir: &Path) -> String {
    git(
        dir,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(objectname)",
            "refs/tags/",
        ],
    )
}

#[test]
fn create_on_an_older_version_undo_and_redo() {
    let (repo, first, second) = repo();
    let dir = repo.path();
    let core = core(dir);
    assert_eq!(
        tag(&core, dir, &first, " 출시/1.0 ").unwrap(),
        ActionResult::Tagged {
            name: "출시/1.0".into()
        }
    );
    assert_eq!(tags(dir), format!("출시/1.0 {first}"));
    assert_eq!(
        git(dir, &["rev-parse", "HEAD"]),
        second,
        "지금 위치는 그대로"
    );
    // 노선도 이름표로 보인다.
    let page = core
        .queries()
        .log(
            dir,
            higit_core::view::LogQuery {
                skip: 0,
                limit: 10,
                scope: Default::default(),
                tips: None,
            },
        )
        .unwrap();
    assert!(page.labels.iter().any(|l| l.name == "출시/1.0"
        && l.oid == first
        && l.kind == higit_core::view::RefLabelKind::Tag));
    let items = core.recovery(dir).unwrap().items;
    assert!(items.iter().any(|i| i.action.as_deref() == Some("tag")));
    core.undo(dir).unwrap();
    assert_eq!(tags(dir), "");
    core.redo(dir).unwrap();
    assert_eq!(tags(dir), format!("출시/1.0 {first}"));
}

#[test]
fn names_and_duplicates_are_checked() {
    let (repo, first, second) = repo();
    let dir = repo.path();
    let core = core(dir);
    tag(&core, dir, &first, "v1").unwrap();
    assert_eq!(
        code(&tag(&core, dir, &second, "v1").unwrap_err()),
        ErrorCode::TagExists
    );
    assert_eq!(
        code(&tag(&core, dir, &second, "V1").unwrap_err()),
        ErrorCode::TagExists
    );
    for bad in ["a..b", "두 단어", "-x", "", "a~1", &"가".repeat(90)] {
        assert_eq!(
            code(&tag(&core, dir, &second, bad).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad:?}"
        );
    }
    // 화면이 아무 해시나 줄 수 없다.
    assert_eq!(
        code(&tag(&core, dir, &"0".repeat(40), "v2").unwrap_err()),
        ErrorCode::InvalidInput
    );
}

#[test]
fn delete_checks_the_version_and_refuses_annotated_tags() {
    let (repo, first, second) = repo();
    let dir = repo.path();
    let core = core(dir);
    tag(&core, dir, &first, "v1").unwrap();
    // 화면이 본 뒤 다른 버전으로 옮겨졌다.
    git(dir, &["tag", "-f", "v1", &second]);
    assert_eq!(
        code(&untag(&core, dir, "v1", &first).unwrap_err()),
        ErrorCode::InvalidInput
    );
    assert_eq!(
        untag(&core, dir, "v1", &second).unwrap(),
        ActionResult::TagDeleted { name: "v1".into() }
    );
    assert_eq!(tags(dir), "");
    core.undo(dir).unwrap();
    assert_eq!(tags(dir), format!("v1 {second}"));
    // 주석 태그.
    git(dir, &["tag", "-a", "-m", "설명", "v2", &first]);
    assert_eq!(
        code(&untag(&core, dir, "v2", &first).unwrap_err()),
        ErrorCode::AnnotatedTag
    );
    assert!(tags(dir).contains("v2"));
}

#[test]
fn undo_fails_when_the_tag_moved_in_between() {
    let (repo, first, second) = repo();
    let dir = repo.path();
    let core = core(dir);
    tag(&core, dir, &first, "v1").unwrap();
    git(dir, &["tag", "-f", "v1", &second]);
    assert!(core.undo(dir).is_err());
    assert_eq!(tags(dir), format!("v1 {second}"));
}
