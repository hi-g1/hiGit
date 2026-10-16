//! 마지막 버전 고치기(`amend`)를 실제 git과 실행기로 시험한다(계획 008 E10, D-097).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, AmendInput, ErrorCode, ErrorView, OpenHow, PathsInput,
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
    repo
}

fn commit(dir: &Path, file: &str, text: &str, message: &str) -> String {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn amend(core: &Core, dir: &Path, message: Option<&str>) -> higit_core::Result<ActionResult> {
    let expect_head = git(dir, &["rev-parse", "HEAD"]);
    core.execute(
        dir,
        &ActionInput::Amend(AmendInput {
            expect_head,
            message: message.map(str::to_string),
        }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn stage(core: &Core, dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    core.execute(
        dir,
        &ActionInput::Stage(PathsInput {
            paths: vec![file.into()],
        }),
        None,
    )
    .unwrap();
}

#[test]
fn message_only_keeps_parent_author_and_tree() {
    let repo = repo();
    let dir = repo.path();
    let parent = commit(dir, "a.txt", "a\n", "처음");
    let before = commit(dir, "b.txt", "b\n", "오타 있는 이름");
    let tree = git(dir, &["rev-parse", "HEAD^{tree}"]);
    let core = core(dir);
    let result = amend(&core, dir, Some("고친 이름\n\n본문 줄")).unwrap();
    let ActionResult::Amended { summary, .. } = result else {
        panic!("{result:?}");
    };
    assert_eq!(summary, "고친 이름");
    assert_ne!(git(dir, &["rev-parse", "HEAD"]), before);
    assert_eq!(git(dir, &["rev-parse", "HEAD^"]), parent);
    assert_eq!(git(dir, &["rev-parse", "HEAD^{tree}"]), tree);
    assert_eq!(
        git(dir, &["log", "-1", "--format=%an <%ae>"]),
        "테스트 <t@example.com>"
    );
    assert_eq!(
        git(dir, &["log", "-1", "--format=%B"]),
        "고친 이름\n\n본문 줄"
    );
    // 고치기 전 버전은 백업 ref로 남는다(실행기, D-043).
    let backups = git(
        dir,
        &[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/higit/backup/",
        ],
    );
    assert!(
        backups.contains(&before) && backups.contains("/amend"),
        "{backups}"
    );
}

#[test]
fn staged_files_go_into_the_version() {
    let repo = repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n", "처음");
    let core = core(dir);
    stage(&core, dir, "c.txt", "넣은 파일\n");
    amend(&core, dir, None).unwrap();
    assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "처음");
    assert!(git(dir, &["show", "--name-only", "--format=", "HEAD"]).contains("c.txt"));
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    // 둘 다.
    stage(&core, dir, "d.txt", "또\n");
    amend(&core, dir, Some("둘 다 고침")).unwrap();
    assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "둘 다 고침");
    assert!(git(dir, &["show", "--name-only", "--format=", "HEAD"]).contains("d.txt"));
    assert_eq!(
        git(dir, &["rev-list", "--count", "HEAD"]),
        "1",
        "첫 버전도 고친다"
    );
}

#[test]
fn merge_version_keeps_both_parents() {
    let repo = repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    let side = commit(dir, "b.txt", "b\n", "기능");
    git(dir, &["switch", "-q", "main"]);
    let mine = commit(dir, "c.txt", "c\n", "내 것");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    let core = core(dir);
    amend(&core, dir, Some("기능 합침")).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD^1"]), mine);
    assert_eq!(git(dir, &["rev-parse", "HEAD^2"]), side);
    assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "기능 합침");
}

#[test]
fn refuses_published_stale_unchanged_and_merging() {
    let repo = repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n", "처음");
    let head = commit(dir, "f.txt", "1\n", "둘");
    let core = core(dir);
    // 바뀐 것 없음.
    assert_eq!(
        code(&amend(&core, dir, None).unwrap_err()),
        ErrorCode::NoChange
    );
    assert_eq!(
        code(&amend(&core, dir, Some("둘\n")).unwrap_err()),
        ErrorCode::NoChange
    );
    assert_eq!(
        code(&amend(&core, dir, Some("  ")).unwrap_err()),
        ErrorCode::InvalidInput
    );
    // 화면이 본 버전이 아니다.
    let err = core
        .execute(
            dir,
            &ActionInput::Amend(AmendInput {
                expect_head: "a".repeat(40),
                message: Some("새 이름".into()),
            }),
            None,
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);
    // 어떤 원격 추적 브랜치에라도 닿으면 올린 것이다.
    git(dir, &["update-ref", "refs/remotes/up/다른", &head]);
    assert_eq!(
        code(&amend(&core, dir, Some("새 이름")).unwrap_err()),
        ErrorCode::PublishedHistory
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    git(dir, &["update-ref", "-d", "refs/remotes/up/다른"]);
    // 합치는 중.
    git(dir, &["switch", "-q", "-c", "기능", "HEAD~1"]);
    commit(dir, "f.txt", "기능\n", "기능");
    git(dir, &["switch", "-q", "main"]);
    let _ = std::process::Command::new("git")
        .args(["merge", "기능"])
        .current_dir(dir)
        .output();
    assert_eq!(
        code(&amend(&core, dir, Some("새 이름")).unwrap_err()),
        ErrorCode::OperationInProgress
    );
}

#[test]
fn hook_refusal_leaves_everything_as_it_was() {
    let repo = repo();
    let dir = repo.path();
    let head = commit(dir, "a.txt", "a\n", "처음");
    let core = core(dir);
    let hook = dir.join(".git/hooks/commit-msg");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&hook)
        .status();
    stage(&core, dir, "b.txt", "b\n");
    assert!(amend(&core, dir, Some("새 이름")).is_err());
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(dir, &["status", "--porcelain"]), "A  b.txt");
}

#[test]
fn undo_brings_the_old_version_back_and_redo_checks_publishing() {
    let repo = repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n", "처음");
    let old = commit(dir, "b.txt", "b\n", "옛 이름");
    let core = core(dir);
    stage(&core, dir, "c.txt", "c\n");
    amend(&core, dir, Some("새 이름")).unwrap();
    let new = git(dir, &["rev-parse", "HEAD"]);
    // 작업 내역에 고치기 줄이 있다.
    let items = core.recovery(dir).unwrap().items;
    assert!(items.iter().any(|i| i.action.as_deref() == Some("amend")));

    let undone = core.undo(dir).unwrap();
    assert_eq!(undone.action, "amend");
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), old);
    // 고치기에 넣은 파일은 다시 넣은 파일로 남는다.
    assert_eq!(git(dir, &["status", "--porcelain"]), "A  c.txt");
    // 고친 버전도 백업 ref로 남는다.
    let backups = git(
        dir,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/higit/backup/",
        ],
    );
    assert!(
        backups.contains(&new) && backups.contains(&old),
        "{backups}"
    );

    // 그사이 옛 버전을 올렸으면 다시 하기가 그것을 기록에서 빼지 않는다.
    git(dir, &["update-ref", "refs/remotes/origin/main", &old]);
    assert_eq!(
        code(&core.redo(dir).unwrap_err()),
        ErrorCode::PublishedHistory
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), old);
    git(dir, &["update-ref", "-d", "refs/remotes/origin/main"]);
    core.redo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), new);
}
