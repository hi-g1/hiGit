//! 브랜치·원격 조회를 실제 git으로 시험한다.

use crate::testutil;
use crate::{BranchKind, CliBackend, GitBackend, RepoPath};

fn commit(dir: &std::path::Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", file]);
    testutil::git(dir, &["commit", "-q", "-m", text]);
}

fn local_backend() -> CliBackend {
    testutil::backend().allow_local_remotes_for_tests()
}

#[test]
fn empty_repo_has_no_branches() {
    let repo = testutil::init_repo();
    let b = testutil::backend()
        .branches(&RepoPath::new(repo.path()))
        .unwrap();
    assert!(b.is_empty());
}

#[test]
fn lists_local_branches_with_head_and_korean_names() {
    let repo = testutil::init_repo();
    commit(repo.path(), "a.txt", "첫 버전");
    testutil::git(repo.path(), &["branch", "기능/한글"]);
    testutil::git(repo.path(), &["branch", "fix-1"]);
    let backend = testutil::backend();
    let path = RepoPath::new(repo.path());
    let b = backend.branches(&path).unwrap();
    let names: Vec<_> = b.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(names, ["fix-1", "main", "기능/한글"]);
    assert!(b.iter().find(|x| x.name == "main").unwrap().is_head);
    assert!(b.iter().all(|x| x.kind == BranchKind::Local));
    assert!(
        b.iter()
            .all(|x| x.last_author == "테스트" && x.last_time > 0)
    );
    assert!(b.iter().all(|x| x.oid.len() >= 40));
}

#[test]
fn upstream_ahead_behind_gone_and_remote_branches() {
    let remote = tempfile::tempdir().unwrap();
    testutil::git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let repo = testutil::init_repo();
    commit(repo.path(), "a.txt", "하나");
    let url = remote.path().to_string_lossy().into_owned();
    testutil::git(repo.path(), &["remote", "add", "origin", &url]);
    testutil::git(repo.path(), &["push", "-q", "-u", "origin", "main"]);
    // 다른 사람이 원격에 하나 올린 것처럼 만든다.
    let other = tempfile::tempdir().unwrap();
    testutil::git(other.path(), &["clone", "-q", &url, "."]);
    commit(other.path(), "b.txt", "남의 것");
    testutil::git(other.path(), &["push", "-q", "origin", "main"]);
    // 내 쪽에도 하나 저장하고 원격을 받아 온다.
    commit(repo.path(), "c.txt", "내 것");
    testutil::git(repo.path(), &["fetch", "-q", "origin"]);
    // 사라진 upstream을 가진 브랜치.
    testutil::git(repo.path(), &["push", "-q", "origin", "main:gone-branch"]);
    testutil::git(
        repo.path(),
        &["branch", "gone-branch", "--track", "origin/gone-branch"],
    );
    testutil::git(
        repo.path(),
        &["push", "-q", "origin", "--delete", "gone-branch"],
    );

    let backend = local_backend();
    let path = RepoPath::new(repo.path());
    let b = backend.branches(&path).unwrap();
    let main = b.iter().find(|x| x.name == "main").unwrap();
    assert_eq!(main.upstream.as_deref(), Some("origin/main"));
    assert_eq!((main.ahead, main.behind, main.gone), (1, 1, false));
    let gone = b.iter().find(|x| x.name == "gone-branch").unwrap();
    assert!(gone.gone);
    let remote_main = b.iter().find(|x| x.name == "origin/main").unwrap();
    assert_eq!(
        remote_main.kind,
        BranchKind::Remote {
            remote: "origin".into(),
            branch: "main".into()
        }
    );

    let remotes = backend.remotes(&path).unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");
}

#[test]
fn remote_head_reads_symbolic_ref() {
    let remote = tempfile::tempdir().unwrap();
    testutil::git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let repo = testutil::init_repo();
    commit(repo.path(), "a.txt", "하나");
    let url = remote.path().to_string_lossy().into_owned();
    testutil::git(repo.path(), &["remote", "add", "origin", &url]);
    testutil::git(repo.path(), &["push", "-q", "origin", "main"]);
    let backend = local_backend();
    let path = RepoPath::new(repo.path());
    assert_eq!(backend.remote_head(&path, "origin").unwrap(), None);
    testutil::git(
        repo.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    assert_eq!(
        backend.remote_head(&path, "origin").unwrap().as_deref(),
        Some("main")
    );
}

#[test]
fn list_refs_under_prefix() {
    let repo = testutil::init_repo();
    commit(repo.path(), "a.txt", "하나");
    testutil::git(
        repo.path(),
        &["update-ref", "refs/higit/parked/main", "HEAD"],
    );
    testutil::git(repo.path(), &["update-ref", "refs/higit/backup/x", "HEAD"]);
    let backend = testutil::backend();
    let path = RepoPath::new(repo.path());
    let refs = backend.list_refs(&path, "refs/higit/parked/").unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].name, "refs/higit/parked/main");
    assert!(backend.list_refs(&path, "--upload-pack=x").is_err());
}

#[test]
fn branch_name_check() {
    let backend = testutil::backend();
    for good in ["main", "기능/한글", "fix-1", "a.b"] {
        assert!(backend.check_branch_name(good).unwrap(), "{good}");
    }
    for bad in [
        "a..b", "-x", "a b", "a~b", "a^b", "a:b", "@{-1}", "a.lock", "HEAD", "", "a/", ".",
    ] {
        assert!(!backend.check_branch_name(bad).unwrap(), "{bad}");
    }
}
