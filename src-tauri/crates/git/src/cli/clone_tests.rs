//! 복제를 로컬 bare 원격으로 시험한다(오프라인).

use std::path::Path;

use crate::testutil;
use crate::{AuthMode, CliBackend, FailureKind, GitBackend, GitOp, RepoPath};

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", "-A"]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

fn backend() -> CliBackend {
    testutil::backend().allow_local_remotes_for_tests()
}

/// 버전 하나와 태그 하나가 든 bare 원격(기본 브랜치 main).
fn remote() -> tempfile::TempDir {
    let remote = tempfile::tempdir().unwrap();
    testutil::git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let work = testutil::init_repo();
    commit(work.path(), "a.txt", "한 줄\n");
    testutil::git(work.path(), &["tag", "v1"]);
    let url = remote.path().to_string_lossy().into_owned();
    testutil::git(work.path(), &["push", "-q", &url, "main", "v1"]);
    remote
}

fn clone_op(url: &str, name: &str) -> GitOp {
    GitOp::Clone {
        url: url.into(),
        name: name.into(),
        auth: AuthMode::UserDefault,
    }
}

#[test]
fn clones_with_origin_branch_and_tags() {
    let remote = remote();
    let parent = tempfile::tempdir().unwrap();
    let url = remote.path().to_string_lossy().into_owned();
    backend()
        .run(
            &RepoPath::new(parent.path()),
            &clone_op(&url, "받은 저장소"),
        )
        .unwrap();
    let target = parent.path().join("받은 저장소");
    assert!(target.join("a.txt").is_file());
    let path = RepoPath::new(&target);
    let b = testutil::backend();
    let remotes = b.remotes(&path).unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");
    assert_eq!(b.current_branch(&path).unwrap().as_deref(), Some("main"));
    let tags = b.tags(&path).unwrap();
    assert!(tags.iter().any(|t| t.name == "v1"), "{tags:?}");
}

#[test]
fn existing_target_is_refused_by_git() {
    let remote = remote();
    let parent = tempfile::tempdir().unwrap();
    std::fs::create_dir(parent.path().join("x")).unwrap();
    std::fs::write(parent.path().join("x").join("keep.txt"), "mine").unwrap();
    let url = remote.path().to_string_lossy().into_owned();
    let err = backend()
        .run(&RepoPath::new(parent.path()), &clone_op(&url, "x"))
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::TargetExists);
    assert!(parent.path().join("x").join("keep.txt").is_file());
}

#[test]
fn disallowed_transports_are_refused() {
    let remote = remote();
    let parent = tempfile::tempdir().unwrap();
    let parent_path = RepoPath::new(parent.path());
    // 시험용 허용이 없는 기본 백엔드는 file:// 를 막는다.
    let path = remote.path().to_string_lossy().replace('\\', "/");
    let file_url = if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    };
    let err = testutil::backend()
        .run(&parent_path, &clone_op(&file_url, "a"))
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::ProtocolNotAllowed, "{err}");
    let err = backend()
        .run(&parent_path, &clone_op("ext::sh -c touch% pwned", "b"))
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::ProtocolNotAllowed, "{err}");
    assert!(!parent.path().join("pwned").exists());
}

#[test]
fn dash_url_is_not_an_option() {
    let parent = tempfile::tempdir().unwrap();
    let err = backend()
        .run(
            &RepoPath::new(parent.path()),
            &clone_op("--upload-pack=touch pwned", "c"),
        )
        .unwrap_err();
    assert_ne!(err.kind(), FailureKind::Cancelled);
    assert!(!parent.path().join("pwned").exists());
    assert!(!parent.path().join("c").join("pwned").exists());
}

#[test]
fn submodules_are_not_fetched() {
    let sub = remote();
    let main = testutil::init_repo();
    commit(main.path(), "m.txt", "main\n");
    let sub_url = sub.path().to_string_lossy().into_owned();
    testutil::git(
        main.path(),
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &sub_url,
            "sub",
        ],
    );
    testutil::git(main.path(), &["commit", "-q", "-m", "sub"]);
    let parent = tempfile::tempdir().unwrap();
    let url = main.path().to_string_lossy().into_owned();
    backend()
        .run(&RepoPath::new(parent.path()), &clone_op(&url, "d"))
        .unwrap();
    let sub_dir = parent.path().join("d").join("sub");
    let entries = std::fs::read_dir(&sub_dir).map(|d| d.count()).unwrap_or(0);
    assert_eq!(entries, 0, "서브모듈 폴더는 비어 있어야 한다");
}

#[test]
fn requested_cancel_on_target_ends_clone() {
    let remote = remote();
    let parent = tempfile::tempdir().unwrap();
    let url = remote.path().to_string_lossy().into_owned();
    let b = backend();
    // 취소 표시의 키는 대상 폴더다.
    b.request_cancel(&RepoPath::new(parent.path().join("e")));
    let err = b
        .run(&RepoPath::new(parent.path()), &clone_op(&url, "e"))
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::Cancelled);
    // 부모 폴더의 표시는 상관없다.
    b.clear_cancel(&RepoPath::new(parent.path().join("e")));
    b.request_cancel(&RepoPath::new(parent.path()));
    b.run(&RepoPath::new(parent.path()), &clone_op(&url, "f"))
        .unwrap();
}
