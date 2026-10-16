//! 이 버전으로 돌아가기(`reset`)를 실제 git과 실행기로 시험한다(계획 008 E13, D-097).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ActionInput, ActionResult, ErrorCode, ErrorView, OidInput, OpenHow};
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

fn repo() -> tempfile::TempDir {
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

fn reset(core: &Core, dir: &Path, oid: &str) -> higit_core::Result<ActionResult> {
    core.execute(dir, &ActionInput::Reset(OidInput { oid: oid.into() }), None)
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

/// 처음 → 둘 → 셋 → 넷(파일 a.txt를 고치고 b·c·d를 더함).
fn four(dir: &Path) -> [String; 4] {
    [
        commit(dir, "a.txt", "a\n", "처음"),
        commit(dir, "b.txt", "b\n", "둘"),
        commit(dir, "c.txt", "c\n", "셋"),
        commit(dir, "d.txt", "d\n", "넷"),
    ]
}

#[test]
fn moves_back_keeps_edits_and_undo_comes_forward() {
    let repo = repo();
    let dir = repo.path();
    let [_, two, _, four] = four(dir);
    std::fs::write(dir.join("a.txt"), "고친 내용\n").unwrap();
    let core = core(dir);
    assert_eq!(
        reset(&core, dir, &two).unwrap(),
        ActionResult::Reset { dropped: 2 }
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), two);
    assert!(!dir.join("c.txt").exists() && !dir.join("d.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "고친 내용\n"
    );
    let backups = git(
        dir,
        &[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/higit/backup/",
        ],
    );
    assert!(
        backups.contains(&four) && backups.contains("/reset"),
        "{backups}"
    );
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), four);
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "고친 내용\n"
    );
    core.redo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), two);
}

#[test]
fn published_versions_are_not_dropped() {
    let repo = repo();
    let dir = repo.path();
    let [one, two, three, four] = four(dir);
    git(dir, &["update-ref", "refs/remotes/origin/main", &three]);
    let core = core(dir);
    assert_eq!(
        code(&reset(&core, dir, &two).unwrap_err()),
        ErrorCode::PublishedHistory
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), four);
    // 올린 버전 위의 안 올린 버전만 빠지면 된다.
    assert_eq!(
        reset(&core, dir, &three).unwrap(),
        ActionResult::Reset { dropped: 1 }
    );
    // 다시 하기도 같은 검사를 한다: 되돌린 뒤 넷을 올렸다.
    core.undo(dir).unwrap();
    git(dir, &["update-ref", "refs/remotes/origin/main", &four]);
    assert_eq!(
        code(&core.redo(dir).unwrap_err()),
        ErrorCode::PublishedHistory
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), four);
    let _ = one;
}

#[test]
fn refuses_other_hashes_and_overlapping_edits() {
    let repo = repo();
    let dir = repo.path();
    let [one, _, _, four] = four(dir);
    git(dir, &["switch", "-q", "-c", "옆", &one]);
    let side = commit(dir, "s.txt", "s\n", "옆");
    git(dir, &["switch", "-q", "main"]);
    let core = core(dir);
    for bad in [side.as_str(), four.as_str(), "HEAD~1", &"0".repeat(40)] {
        assert_eq!(
            code(&reset(&core, dir, bad).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad}"
        );
    }
    // 빠지는 버전이 만든 파일을 고쳤으면 git이 아무것도 바꾸지 않는다.
    std::fs::write(dir.join("d.txt"), "내 고침\n").unwrap();
    assert_eq!(
        code(&reset(&core, dir, &one).unwrap_err()),
        ErrorCode::WouldOverwrite
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), four);
    assert_eq!(
        std::fs::read_to_string(dir.join("d.txt")).unwrap(),
        "내 고침\n"
    );
}

#[test]
fn a_merged_range_can_be_dropped() {
    let repo = repo();
    let dir = repo.path();
    let one = commit(dir, "a.txt", "a\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "b.txt", "b\n", "기능");
    git(dir, &["switch", "-q", "main"]);
    commit(dir, "c.txt", "c\n", "내 것");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    let core = core(dir);
    // 합친 버전, 내 것, 합쳐 들어온 기능 버전이 빠진다.
    assert_eq!(
        reset(&core, dir, &one).unwrap(),
        ActionResult::Reset { dropped: 3 }
    );
    assert!(!dir.join("b.txt").exists() && !dir.join("c.txt").exists());
    // 합쳐 들어온 버전이 올라가 있으면(다른 브랜치로) 거절한다.
    core.undo(dir).unwrap();
    let side = git(dir, &["rev-parse", "기능"]);
    git(dir, &["update-ref", "refs/remotes/origin/기능", &side]);
    assert_eq!(
        code(&reset(&core, dir, &one).unwrap_err()),
        ErrorCode::PublishedHistory
    );
}
