//! 실행기가 journal을 남기는지 실제 git으로 시험한다(D-042, D-065).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ActionInput, CommitInput, OpenHow, PathsInput};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn lines(dir: &Path) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(dir.join(".git/higit/journal.jsonl")).unwrap_or_default();
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn stage_and_commit_are_recorded() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.name", "테스트"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
    std::fs::write(dir.join("a.txt"), "a").unwrap();

    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core.execute(
        dir,
        &ActionInput::Stage(PathsInput {
            paths: vec!["a.txt".into()],
        }),
        None,
    )
    .unwrap();
    core.execute(
        dir,
        &ActionInput::Commit(CommitInput {
            message: "첫 버전".into(),
            stage_all: false,
        }),
        None,
    )
    .unwrap();

    let l = lines(dir);
    assert_eq!(l.len(), 2, "{l:?}");
    assert_eq!(l[0]["type"], "action");
    assert_eq!(l[0]["action"], "stage");
    assert_eq!(l[0]["kind"], "restoreIndex");
    assert_ne!(l[0]["before"]["indexTree"], l[0]["after"]["indexTree"]);
    assert_eq!(l[1]["type"], "action");
    assert_eq!(l[1]["action"], "commit");
    assert_eq!(l[1]["kind"], "restoreRef");
    assert!(
        l[1]["before"]["head"].is_null(),
        "첫 저장 전에는 HEAD가 없다"
    );
    assert_eq!(l[1]["after"]["branch"], "main");
    assert!(l[1]["after"]["head"].as_str().unwrap().len() >= 40);
    assert!(l[1]["sig"].as_str().unwrap().len() == 64);
}
