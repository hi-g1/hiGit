//! 원격 GitOp을 로컬 bare 원격으로 시험한다(오프라인).

use std::path::Path;

use crate::testutil;
use crate::{CliBackend, FailureKind, GitBackend, GitOp, RepoPath};

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", "-A"]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

fn backend() -> CliBackend {
    testutil::backend().allow_local_remotes_for_tests()
}

/// 원격, 내 저장소(upstream 연결됨), 다른 사람 저장소.
struct Setup {
    _remote: tempfile::TempDir,
    mine: tempfile::TempDir,
    other: tempfile::TempDir,
}

fn setup() -> Setup {
    let remote = tempfile::tempdir().unwrap();
    testutil::git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let url = remote.path().to_string_lossy().into_owned();
    let mine = testutil::init_repo();
    commit(mine.path(), "a.txt", "한 줄\n");
    testutil::git(mine.path(), &["remote", "add", "origin", &url]);
    testutil::git(mine.path(), &["push", "-q", "-u", "origin", "main"]);
    let other = tempfile::tempdir().unwrap();
    testutil::git(other.path(), &["clone", "-q", &url, "."]);
    Setup {
        _remote: remote,
        mine,
        other,
    }
}

fn status(path: &RepoPath) -> crate::BranchInfo {
    testutil::backend().status(path).unwrap().branch
}

#[test]
fn fetch_then_fast_forward() {
    let s = setup();
    commit(s.other.path(), "b.txt", "남의 것\n");
    testutil::git(s.other.path(), &["push", "-q", "origin", "main"]);
    let path = RepoPath::new(s.mine.path());
    let b = backend();
    b.run(
        &path,
        &GitOp::Fetch {
            remote: Some("origin".into()),
            background: true,
        },
    )
    .unwrap();
    assert_eq!(status(&path).behind, 1);
    b.run(
        &path,
        &GitOp::MergeFf {
            rev: "origin/main".into(),
        },
    )
    .unwrap();
    assert_eq!(status(&path).behind, 0);
    assert!(s.mine.path().join("b.txt").exists());
}

#[test]
fn diverged_merge_makes_a_merge_commit_with_message() {
    let s = setup();
    commit(s.other.path(), "b.txt", "남의 것\n");
    testutil::git(s.other.path(), &["push", "-q", "origin", "main"]);
    commit(s.mine.path(), "c.txt", "내 것\n");
    let path = RepoPath::new(s.mine.path());
    let b = backend();
    b.run(
        &path,
        &GitOp::Fetch {
            remote: None,
            background: false,
        },
    )
    .unwrap();
    let err = b
        .run(
            &path,
            &GitOp::MergeFf {
                rev: "origin/main".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::NotFastForward, "{}", err.detail());
    // 사용자가 merge.ff=only를 켰어도 --no-ff로 합친다.
    testutil::git(s.mine.path(), &["config", "merge.ff", "only"]);
    b.run(
        &path,
        &GitOp::Merge {
            rev: "origin/main".into(),
            message: "테스트의 버전 1개 가져와 합침".into(),
        },
    )
    .unwrap();
    let log = b.log(&path, &crate::LogQuery::default()).unwrap();
    assert_eq!(log[0].summary, "테스트의 버전 1개 가져와 합침");
    assert_eq!(log[0].parents.len(), 2);
}

#[test]
fn conflicting_merge_is_classified_and_aborted() {
    let s = setup();
    commit(s.other.path(), "a.txt", "남이 고침\n");
    testutil::git(s.other.path(), &["push", "-q", "origin", "main"]);
    commit(s.mine.path(), "a.txt", "내가 고침\n");
    let path = RepoPath::new(s.mine.path());
    let b = backend();
    b.run(
        &path,
        &GitOp::Fetch {
            remote: None,
            background: false,
        },
    )
    .unwrap();
    let err = b
        .run(
            &path,
            &GitOp::Merge {
                rev: "origin/main".into(),
                message: "합침".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::MergeConflict, "{}", err.detail());
    b.run(&path, &GitOp::MergeAbort).unwrap();
    assert!(b.status(&path).unwrap().entries.is_empty());
    // 합치기 중이 아니면 아무것도 하지 않는다.
    b.run(&path, &GitOp::MergeAbort).unwrap();
}

#[test]
fn push_upstream_and_rejection() {
    let s = setup();
    let path = RepoPath::new(s.mine.path());
    let b = backend();
    testutil::git(s.mine.path(), &["switch", "-q", "-c", "기능"]);
    commit(s.mine.path(), "d.txt", "새 브랜치\n");
    b.run(
        &path,
        &GitOp::Push {
            target: Some(("origin".into(), "기능".into())),
            set_upstream: true,
        },
    )
    .unwrap();
    assert_eq!(status(&path).upstream.as_deref(), Some("origin/기능"));

    // 다른 사람이 main에 먼저 올리면 내 main 올리기는 거부된다.
    testutil::git(s.mine.path(), &["switch", "-q", "main"]);
    commit(s.other.path(), "b.txt", "남의 것\n");
    testutil::git(s.other.path(), &["push", "-q", "origin", "main"]);
    commit(s.mine.path(), "c.txt", "내 것\n");
    let err = b
        .run(
            &path,
            &GitOp::Push {
                target: None,
                set_upstream: false,
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::PushRejected, "{}", err.detail());
}

#[test]
fn unreachable_remote_is_classified() {
    let repo = testutil::init_repo();
    commit(repo.path(), "a.txt", "a\n");
    let missing = repo.path().join("없는 원격");
    let url = missing.to_string_lossy().into_owned();
    testutil::git(repo.path(), &["remote", "add", "origin", &url]);
    let err = backend()
        .run(
            &RepoPath::new(repo.path()),
            &GitOp::Fetch {
                remote: Some("origin".into()),
                background: true,
            },
        )
        .unwrap_err();
    assert_eq!(
        err.kind(),
        FailureKind::RemoteUnreachable,
        "{}",
        err.detail()
    );
}
