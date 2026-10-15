//! ref 읽기·쓰기와 버전 관계를 실제 git으로 시험한다.

use crate::testutil;
use crate::{GitBackend, Oid, RefExpect, RepoPath};

fn commit(dir: &std::path::Path, file: &str, text: &str) -> Oid {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", file]);
    testutil::git(dir, &["commit", "-q", "-m", text]);
    Oid::parse(&testutil::git(dir, &["rev-parse", "HEAD"])).unwrap()
}

#[test]
fn resolve_update_and_delete_with_expectations() {
    let repo = testutil::init_repo();
    let path = RepoPath::new(repo.path());
    let backend = testutil::backend();
    assert_eq!(backend.resolve_ref(&path, "HEAD").unwrap(), None);
    let first = commit(repo.path(), "a.txt", "하나");
    let second = commit(repo.path(), "a.txt", "둘");
    assert_eq!(
        backend.resolve_ref(&path, "HEAD").unwrap(),
        Some(second.clone())
    );
    assert_eq!(backend.resolve_ref(&path, "refs/higit/x").unwrap(), None);

    let name = "refs/higit/backup/1/main";
    backend
        .update_ref(&path, name, &first, &RefExpect::Absent)
        .unwrap();
    // 이미 있으니 "없어야 함"은 실패한다.
    assert!(
        backend
            .update_ref(&path, name, &second, &RefExpect::Absent)
            .is_err()
    );
    // 이전 값이 다르면 실패한다.
    assert!(
        backend
            .update_ref(&path, name, &second, &RefExpect::Is(second.clone()))
            .is_err()
    );
    backend
        .update_ref(&path, name, &second, &RefExpect::Is(first.clone()))
        .unwrap();
    assert_eq!(
        backend.resolve_ref(&path, name).unwrap(),
        Some(second.clone())
    );
    assert!(
        backend
            .delete_ref(&path, name, &RefExpect::Is(first.clone()))
            .is_err()
    );
    backend
        .delete_ref(&path, name, &RefExpect::Is(second.clone()))
        .unwrap();
    assert_eq!(backend.resolve_ref(&path, name).unwrap(), None);
    // core가 만든 이름이 아니면 받지 않는다.
    assert!(
        backend
            .update_ref(&path, "-x", &first, &RefExpect::Any)
            .is_err()
    );
    assert!(
        backend
            .update_ref(&path, "HEAD", &first, &RefExpect::Any)
            .is_err()
    );
}

#[test]
fn option_like_revision_is_not_an_option() {
    let repo = testutil::init_repo();
    let path = RepoPath::new(repo.path());
    commit(repo.path(), "a.txt", "하나");
    let backend = testutil::backend();
    if backend.version().unwrap() < crate::GitVersion::new(2, 34, 0) {
        return;
    }
    let pwned = repo.path().join("pwned");
    let rev = format!("--output={}", pwned.display());
    assert!(backend.resolve_ref(&path, &rev).unwrap().is_none());
    assert!(backend.is_ancestor(&path, &rev, "HEAD").is_err());
    assert!(!pwned.exists());
}

#[test]
fn ancestry_counts_and_merge_base() {
    let repo = testutil::init_repo();
    let path = RepoPath::new(repo.path());
    let base = commit(repo.path(), "a.txt", "하나");
    testutil::git(repo.path(), &["branch", "side"]);
    commit(repo.path(), "a.txt", "main 둘");
    commit(repo.path(), "a.txt", "main 셋");
    testutil::git(repo.path(), &["switch", "-q", "side"]);
    commit(repo.path(), "b.txt", "side 하나");
    let backend = testutil::backend();
    assert!(backend.is_ancestor(&path, base.as_str(), "main").unwrap());
    assert!(!backend.is_ancestor(&path, "main", "side").unwrap());
    assert_eq!(
        backend.count_between(&path, "main", "side").unwrap(),
        (2, 1)
    );
    assert_eq!(
        backend.merge_base(&path, "main", "side").unwrap(),
        Some(base)
    );
}

#[test]
fn unrelated_histories_have_no_merge_base() {
    let repo = testutil::init_repo();
    let path = RepoPath::new(repo.path());
    commit(repo.path(), "a.txt", "하나");
    testutil::git(repo.path(), &["switch", "-q", "--orphan", "other"]);
    commit(repo.path(), "b.txt", "따로");
    let backend = testutil::backend();
    assert_eq!(backend.merge_base(&path, "main", "other").unwrap(), None);
}

#[test]
fn git_dir_for_main_tree_and_worktree() {
    let repo = testutil::init_repo();
    commit(repo.path(), "a.txt", "하나");
    let backend = testutil::backend();
    let dir = backend.git_dir(&RepoPath::new(repo.path())).unwrap();
    assert!(dir.is_absolute(), "{dir:?}");
    assert_eq!(
        testutil::canonical(&dir),
        testutil::canonical(&repo.path().join(".git"))
    );

    let wt_parent = tempfile::tempdir().unwrap();
    let wt = wt_parent.path().join("작업 트리");
    let wt_str = wt.to_string_lossy().into_owned();
    testutil::git(repo.path(), &["worktree", "add", "-q", "-b", "wt", &wt_str]);
    let wt_dir = backend.git_dir(&RepoPath::new(&wt)).unwrap();
    assert!(
        testutil::canonical(&wt_dir)
            .starts_with(testutil::canonical(&repo.path().join(".git/worktrees"))),
        "{wt_dir:?}"
    );
}

#[test]
fn write_tree_and_current_branch() {
    let repo = testutil::init_repo();
    let path = RepoPath::new(repo.path());
    let backend = testutil::backend();
    assert_eq!(
        backend.current_branch(&path).unwrap().as_deref(),
        Some("main")
    );
    commit(repo.path(), "a.txt", "하나");
    let tree = backend.write_tree(&path).unwrap().unwrap();
    let head_tree = testutil::git(repo.path(), &["rev-parse", "HEAD^{tree}"]);
    assert_eq!(tree.as_str(), head_tree.trim());
    testutil::git(repo.path(), &["switch", "-q", "--detach"]);
    assert_eq!(backend.current_branch(&path).unwrap(), None);
}
