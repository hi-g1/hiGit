//! 브랜치 목록·이름 검사 조회를 실제 git으로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{NameProblem, OpenHow};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .args(["-c", "user.name=테스트", "-c", "user.email=t@example.com"])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        ok.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&ok.stderr)
    );
}

fn commit(dir: &Path, file: &str) {
    std::fs::write(dir.join(file), file).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

fn core(repo: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(repo, OpenHow::Picked).unwrap();
    core
}

#[test]
fn lists_branches_with_default_parked_and_orphans() {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "trunk"]);
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "trunk"]);
    commit(dir, "a.txt");
    git(dir, &["branch", "기능/한글"]);
    let url = remote.path().to_string_lossy().into_owned();
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "-u", "origin", "trunk"]);
    git(dir, &["remote", "set-head", "origin", "trunk"]);
    // 기능/한글에 맡겨 둔 파일 하나, 지워진 브랜치 old에 맡겨 둔 파일 하나.
    std::fs::write(dir.join("a.txt"), "고침").unwrap();
    git(dir, &["stash", "push", "-q"]);
    git(
        dir,
        &["update-ref", "refs/higit/parked/기능%2F한글", "stash@{0}"],
    );
    git(dir, &["update-ref", "refs/higit/parked/old", "stash@{0}"]);

    let core = core(dir);
    let list = core.queries().branches(dir).unwrap();
    assert_eq!(list.default_branch.as_deref(), Some("trunk"));
    let trunk = list.local.iter().find(|b| b.name == "trunk").unwrap();
    assert!(trunk.is_head && trunk.is_default);
    assert_eq!(trunk.parked_count, 0);
    let feature = list.local.iter().find(|b| b.name == "기능/한글").unwrap();
    assert_eq!(feature.parked_count, 1);
    assert!(!feature.park_blocked);
    assert!(list.remote.iter().any(|b| b.name == "origin/trunk"));
    assert!(list.orphan_parked.contains(&"old".to_string()));
}

#[test]
fn default_branch_falls_back_to_main_without_remote() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    commit(dir, "a.txt");
    git(dir, &["switch", "-q", "-c", "other"]);
    let list = core(dir).queries().branches(dir).unwrap();
    assert_eq!(list.default_branch.as_deref(), Some("main"));
}

#[test]
fn branch_name_check() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    commit(dir, "a.txt");
    git(dir, &["branch", "Feature"]);
    let core = core(dir);
    let q = core.queries();
    assert!(q.check_branch_name(dir, "새 기능").unwrap().problem == Some(NameProblem::Invalid));
    assert!(q.check_branch_name(dir, "새기능").unwrap().ok);
    assert_eq!(
        q.check_branch_name(dir, "  ").unwrap().problem,
        Some(NameProblem::Empty)
    );
    assert_eq!(
        q.check_branch_name(dir, "-x").unwrap().problem,
        Some(NameProblem::Invalid)
    );
    assert_eq!(
        q.check_branch_name(dir, "feature").unwrap().problem,
        Some(NameProblem::Exists)
    );
    assert_eq!(
        q.check_branch_name(dir, "main").unwrap().problem,
        Some(NameProblem::Exists)
    );
}
