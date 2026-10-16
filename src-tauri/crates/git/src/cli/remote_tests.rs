//! 원격 GitOp을 로컬 bare 원격으로 시험한다(오프라인).

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
            auth: AuthMode::UserDefault,
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
            auth: AuthMode::UserDefault,
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
            auth: AuthMode::UserDefault,
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
            auth: AuthMode::UserDefault,
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
                auth: AuthMode::UserDefault,
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::PushRejected, "{}", err.detail());
}

/// 서버 hook이 거절하면(보호 브랜치 흉내) 일반 올리기 거절과 따로 `RemoteRefused`다(계획 010 R11).
/// hook 실행 비트가 필요해 Unix에서만 시험한다.
#[cfg(unix)]
#[test]
fn server_hook_refusal_is_remote_refused() {
    use std::os::unix::fs::PermissionsExt;
    let s = setup();
    let hook = s._remote.path().join("hooks").join("pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho '보호된 브랜치' >&2\nexit 1\n").unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    commit(s.mine.path(), "c.txt", "내 것\n");
    let err = backend()
        .run(
            &RepoPath::new(s.mine.path()),
            &GitOp::Push {
                target: None,
                set_upstream: false,
                auth: AuthMode::UserDefault,
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::RemoteRefused, "{}", err.detail());
}

/// GitHub의 큰 파일 거절(GH001) 문구를 hook으로 흉내 내면 `FileTooLarge`다(D-194).
#[cfg(unix)]
#[test]
fn large_file_refusal_is_its_own_kind() {
    use std::os::unix::fs::PermissionsExt;
    let s = setup();
    let hook = s._remote.path().join("hooks").join("pre-receive");
    std::fs::write(
        &hook,
        "#!/bin/sh\necho 'error: GH001: Large files detected. You may want to try Git Large File Storage.' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    commit(s.mine.path(), "c.txt", "내 것\n");
    let err = backend()
        .run(
            &RepoPath::new(s.mine.path()),
            &GitOp::Push {
                target: None,
                set_upstream: false,
                auth: AuthMode::UserDefault,
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::FileTooLarge, "{}", err.detail());
}

/// GitHub의 비공개 이메일 거절(GH007) 문구를 hook으로 흉내 내면 `PrivateEmailBlocked`다(D-186).
#[cfg(unix)]
#[test]
fn private_email_refusal_is_its_own_kind() {
    use std::os::unix::fs::PermissionsExt;
    let s = setup();
    let hook = s._remote.path().join("hooks").join("pre-receive");
    std::fs::write(
        &hook,
        "#!/bin/sh\necho 'error: GH007: Your push would publish a private email address.' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    commit(s.mine.path(), "c.txt", "내 것\n");
    let err = backend()
        .run(
            &RepoPath::new(s.mine.path()),
            &GitOp::Push {
                target: None,
                set_upstream: false,
                auth: AuthMode::UserDefault,
            },
        )
        .unwrap_err();
    assert_eq!(
        err.kind(),
        FailureKind::PrivateEmailBlocked,
        "{}",
        err.detail()
    );
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
                auth: AuthMode::UserDefault,
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

#[test]
fn remote_urls_apply_instead_of_and_pushurl() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let backend = testutil::backend();
    let path = RepoPath::new(p);
    testutil::git(p, &["remote", "add", "origin", "gh:octo/hello.git"]);
    testutil::git(p, &["config", "url.https://github.com/.insteadOf", "gh:"]);
    // pushInsteadOf는 바꾸기 전 원래 주소에 맞춘다.
    testutil::git(
        p,
        &["config", "url.ssh://git@example.com/.pushInsteadOf", "gh:"],
    );
    let urls = backend.remote_urls(&path, "origin").unwrap();
    assert_eq!(urls.fetch, ["https://github.com/octo/hello.git"]);
    assert_eq!(urls.push, ["ssh://git@example.com/octo/hello.git"]);

    // pushurl이 있으면 올리기 주소는 그것이고 여러 개를 모두 준다.
    testutil::git(
        p,
        &[
            "config",
            "--add",
            "remote.origin.pushurl",
            "https://a.example/x.git",
        ],
    );
    testutil::git(
        p,
        &[
            "config",
            "--add",
            "remote.origin.pushurl",
            "https://b.example/y.git",
        ],
    );
    let urls = backend.remote_urls(&path, "origin").unwrap();
    assert_eq!(urls.fetch, ["https://github.com/octo/hello.git"]);
    assert_eq!(
        urls.push,
        ["https://a.example/x.git", "https://b.example/y.git"]
    );

    assert!(backend.remote_urls(&path, "없는원격").is_err());
}

#[test]
fn remote_add_reads_names_and_urls_as_values() {
    let repo = testutil::init_repo();
    let path = RepoPath::new(repo.path());
    let b = backend();
    b.run(
        &path,
        &GitOp::RemoteAdd {
            name: "origin".into(),
            url: "https://github.com/octo/hello.git".into(),
        },
    )
    .unwrap();
    let remotes = b.remotes(&path).unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].url, "https://github.com/octo/hello.git");
    // 옵션 같은 이름도 옵션으로 읽지 않는다(--end-of-options). core는 이런 이름을 보내지 않는다.
    b.run(
        &path,
        &GitOp::RemoteAdd {
            name: "--mirror=fetch".into(),
            url: "https://github.com/octo/x.git".into(),
        },
    )
    .unwrap();
    assert!(
        b.remotes(&path)
            .unwrap()
            .iter()
            .any(|r| r.name == "--mirror=fetch")
    );
}
