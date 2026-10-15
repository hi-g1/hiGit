//! 홈 두 줄 노선도 조회를 실제 git으로 시험한다(D-046).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::OpenHow;
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@e.com",
            "-c",
            "commit.gpgsign=false",
        ])
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

fn commit(dir: &Path, file: &str) {
    std::fs::write(dir.join(file), file).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

fn core(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

#[test]
fn empty_repo_has_no_commits() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "main"]);
    let map = core(repo.path()).queries().home_map(repo.path()).unwrap();
    assert!(map.current_commits.is_empty());
    assert!(!map.two_lines);
}

#[test]
fn on_main_is_one_line_with_recent_commits() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for f in ["1", "2", "3", "4", "5"] {
        commit(dir, f);
    }
    std::fs::write(dir.join("x"), "x").unwrap();
    let map = core(dir).queries().home_map(dir).unwrap();
    assert!(!map.two_lines);
    assert_eq!(
        map.current_commits
            .iter()
            .map(|c| c.summary.as_str())
            .collect::<Vec<_>>(),
        ["3", "4", "5"]
    );
    assert_eq!(map.current_more, 1);
    assert_eq!(map.changed_files, 1);
    assert_eq!(map.default_branch.as_deref(), Some("main"));
}

#[test]
fn feature_branch_is_two_lines() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    commit(dir, "base");
    git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "f1");
    commit(dir, "f2");
    git(dir, &["switch", "-q", "main"]);
    commit(dir, "m1");
    git(dir, &["switch", "-q", "기능"]);
    let map = core(dir).queries().home_map(dir).unwrap();
    assert!(map.two_lines);
    assert_eq!(map.current.as_deref(), Some("기능"));
    assert_eq!(
        map.current_commits
            .iter()
            .map(|c| c.summary.as_str())
            .collect::<Vec<_>>(),
        ["f1", "f2"]
    );
    assert_eq!(map.main_since, 1);
    assert_eq!(map.main_last.unwrap().summary, "m1");
}
