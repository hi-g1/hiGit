//! 원격 ref 쓰기와 `ls-remote`를 로컬 bare 원격으로 시험한다(오프라인, 계획 010 H7b).

use std::path::Path;

use crate::testutil;
use crate::{AuthMode, CliBackend, FailureKind, GitBackend, GitError, GitOp, Oid, RepoPath};

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", "-A"]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

fn backend() -> CliBackend {
    testutil::backend().allow_local_remotes_for_tests()
}

const AUTH: AuthMode = AuthMode::UserDefault;

/// 원격(main·`다른-브랜치`)과 내 저장소.
struct Setup {
    remote: tempfile::TempDir,
    mine: tempfile::TempDir,
}

impl Setup {
    fn repo(&self) -> RepoPath {
        RepoPath::new(self.mine.path())
    }

    /// 원격에서 지금 그 ref의 버전.
    fn remote_ref(&self, name: &str) -> Option<Oid> {
        testutil::backend()
            .resolve_ref(&RepoPath::new(self.remote.path()), name)
            .unwrap()
    }

    fn head(&self) -> Oid {
        testutil::backend()
            .resolve_ref(&self.repo(), "HEAD")
            .unwrap()
            .unwrap()
    }
}

fn setup() -> Setup {
    let remote = tempfile::tempdir().unwrap();
    testutil::git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let url = remote.path().to_string_lossy().into_owned();
    let mine = testutil::init_repo();
    commit(mine.path(), "a.txt", "한 줄\n");
    testutil::git(mine.path(), &["remote", "add", "origin", &url]);
    testutil::git(mine.path(), &["push", "-q", "-u", "origin", "main"]);
    testutil::git(
        mine.path(),
        &["push", "-q", "origin", "main:refs/heads/다른-브랜치"],
    );
    Setup { remote, mine }
}

fn kind(err: &GitError) -> FailureKind {
    err.kind()
}

fn args(err: &GitError) -> Vec<String> {
    match err {
        GitError::Failed { args, .. } => args.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn force_push_overwrites_only_the_expected_value() {
    let s = setup();
    let pushed = s.head();
    // 올린 버전을 고쳐 갈라진 기록을 만든다.
    testutil::git(
        s.mine.path(),
        &["commit", "-q", "--amend", "-m", "고친 이름"],
    );
    let fixed = s.head();
    let other_before = s.remote_ref("refs/heads/다른-브랜치");

    // 기대 값이 틀리면 거절하고 원격은 그대로다.
    let wrong = Oid::parse(&"0".repeat(40)).unwrap();
    let err = backend()
        .run(
            &s.repo(),
            &GitOp::PushForce {
                remote: "origin".into(),
                branch: "main".into(),
                src: fixed.clone(),
                expect: wrong,
                auth: AUTH,
            },
        )
        .unwrap_err();
    assert_eq!(kind(&err), FailureKind::LeaseStale, "{}", err.detail());
    let a = args(&err);
    assert!(
        a.iter()
            .any(|x| x.starts_with("--force-with-lease=refs/heads/main:")),
        "{a:?}"
    );
    assert!(
        !a.iter()
            .any(|x| x == "--force" || x == "-f" || x.starts_with('+')),
        "{a:?}"
    );
    assert_eq!(s.remote_ref("refs/heads/main"), Some(pushed.clone()));

    // 맞으면 덮어쓴다. 다른-브랜치는 그대로.
    backend()
        .run(
            &s.repo(),
            &GitOp::PushForce {
                remote: "origin".into(),
                branch: "main".into(),
                src: fixed.clone(),
                expect: pushed,
                auth: AUTH,
            },
        )
        .unwrap();
    assert_eq!(s.remote_ref("refs/heads/main"), Some(fixed));
    assert_eq!(s.remote_ref("refs/heads/다른-브랜치"), other_before);
}

#[test]
fn force_push_sends_the_checked_version_not_head() {
    let s = setup();
    let pushed = s.head();
    testutil::git(
        s.mine.path(),
        &["commit", "-q", "--amend", "-m", "확인한 버전"],
    );
    let checked = s.head();
    // 확인한 뒤 HEAD가 더 움직여도 확인한 버전만 올라간다(계획 010 R4).
    commit(s.mine.path(), "b.txt", "그 뒤\n");
    backend()
        .run(
            &s.repo(),
            &GitOp::PushForce {
                remote: "origin".into(),
                branch: "main".into(),
                src: checked.clone(),
                expect: pushed,
                auth: AUTH,
            },
        )
        .unwrap();
    assert_eq!(s.remote_ref("refs/heads/main"), Some(checked));
}

#[test]
fn delete_needs_the_expected_value() {
    let s = setup();
    let there = s.remote_ref("refs/heads/다른-브랜치").unwrap();
    let delete = |expect: Oid| GitOp::PushDelete {
        remote: "origin".into(),
        branch: "다른-브랜치".into(),
        expect,
        auth: AUTH,
    };
    let wrong = Oid::parse(&"0".repeat(40)).unwrap();
    let err = backend().run(&s.repo(), &delete(wrong)).unwrap_err();
    assert_eq!(kind(&err), FailureKind::LeaseStale, "{}", err.detail());
    assert!(s.remote_ref("refs/heads/다른-브랜치").is_some());

    backend().run(&s.repo(), &delete(there.clone())).unwrap();
    assert_eq!(s.remote_ref("refs/heads/다른-브랜치"), None);

    // 이미 없으면 값 있는 lease는 "remote ref does not exist"가 아니라 (stale info)로 거절한다(R3).
    let err = backend().run(&s.repo(), &delete(there)).unwrap_err();
    assert_eq!(kind(&err), FailureKind::LeaseStale, "{}", err.detail());
}

#[test]
fn new_ref_is_made_only_when_the_name_is_free() {
    let s = setup();
    let head = s.head();
    let make = |branch: &str| GitOp::PushNewRef {
        remote: "origin".into(),
        branch: branch.into(),
        oid: head.clone(),
        auth: AUTH,
    };
    backend().run(&s.repo(), &make("새-이름")).unwrap();
    assert_eq!(s.remote_ref("refs/heads/새-이름"), Some(head.clone()));

    // 같은 이름이 다른 버전으로 있으면 거절(빈 기대 값).
    commit(s.mine.path(), "b.txt", "더\n");
    let newer = s.head();
    let err = backend()
        .run(
            &s.repo(),
            &GitOp::PushNewRef {
                remote: "origin".into(),
                branch: "새-이름".into(),
                oid: newer,
                auth: AUTH,
            },
        )
        .unwrap_err();
    assert_eq!(kind(&err), FailureKind::LeaseStale, "{}", err.detail());
    assert_eq!(s.remote_ref("refs/heads/새-이름"), Some(head));
}

#[test]
fn tag_push_never_moves_an_existing_tag() {
    let s = setup();
    testutil::git(s.mine.path(), &["tag", "v1"]);
    let tag = |name: &str| GitOp::PushTag {
        remote: "origin".into(),
        name: name.into(),
        auth: AUTH,
    };
    backend().run(&s.repo(), &tag("v1")).unwrap();
    let first = s.remote_ref("refs/tags/v1");
    assert_eq!(first, Some(s.head()));
    // 같은 버전이면 다시 올려도 된다.
    backend().run(&s.repo(), &tag("v1")).unwrap();

    // 같은 이름을 다른 버전으로 옮기면 거절.
    commit(s.mine.path(), "b.txt", "더\n");
    testutil::git(s.mine.path(), &["tag", "-f", "v1"]);
    let err = backend().run(&s.repo(), &tag("v1")).unwrap_err();
    assert_eq!(kind(&err), FailureKind::RemoteRefExists, "{}", err.detail());
    assert_eq!(s.remote_ref("refs/tags/v1"), first);
}

#[test]
fn tag_delete_needs_the_expected_value() {
    let s = setup();
    testutil::git(s.mine.path(), &["tag", "v1"]);
    testutil::git(s.mine.path(), &["tag", "-a", "-m", "주석", "v2"]);
    testutil::git(s.mine.path(), &["push", "-q", "origin", "v1", "v2"]);
    // ref 값 그대로(주석 태그는 태그 객체).
    let local = |name: &str| {
        Oid::parse(&testutil::git(
            s.mine.path(),
            &["rev-parse", &format!("refs/tags/{name}")],
        ))
        .unwrap()
    };
    let delete = |name: &str, expect: Oid| GitOp::PushDeleteTag {
        remote: "origin".into(),
        name: name.into(),
        expect,
        auth: AUTH,
    };
    // 다른 값이면 지우지 않는다(남이 바꿔 둔 태그).
    let other = Oid::parse(&"0".repeat(40)).unwrap();
    let err = backend().run(&s.repo(), &delete("v1", other)).unwrap_err();
    assert_eq!(kind(&err), FailureKind::LeaseStale, "{}", err.detail());
    assert!(
        !args(&err)
            .iter()
            .any(|a| a == "--force" || a.starts_with('+'))
    );
    assert!(s.remote_ref("refs/tags/v1").is_some());
    // 같은 값이면 그 태그만 지운다. 주석 태그는 태그 객체 값으로.
    backend()
        .run(&s.repo(), &delete("v1", local("v1")))
        .unwrap();
    backend()
        .run(&s.repo(), &delete("v2", local("v2")))
        .unwrap();
    assert_eq!(s.remote_ref("refs/tags/v1"), None);
    assert_eq!(s.remote_ref("refs/tags/v2"), None);
    assert!(s.remote_ref("refs/heads/main").is_some());
    // 없는 태그는 lease가 어긋난다.
    let err = backend()
        .run(&s.repo(), &delete("v1", local("v1")))
        .unwrap_err();
    assert_eq!(kind(&err), FailureKind::LeaseStale, "{}", err.detail());
}

#[test]
fn ls_remote_reads_one_exact_ref() {
    let s = setup();
    let b = backend();
    assert_eq!(
        b.ls_remote(&s.repo(), "origin", "refs/heads/main", AUTH)
            .unwrap(),
        Some(s.head())
    );
    assert_eq!(
        b.ls_remote(&s.repo(), "origin", "refs/heads/없음", AUTH)
            .unwrap(),
        None
    );
    // 짧은 이름은 꼬리 일치로 다른 ref가 걸리므로 전체 이름만 본다.
    assert_eq!(
        b.ls_remote(&s.repo(), "origin", "main", AUTH).unwrap(),
        None
    );
    assert_eq!(
        b.ls_remote_head(&s.repo(), "origin", AUTH)
            .unwrap()
            .as_deref(),
        Some("main")
    );
}

#[test]
fn option_like_values_are_not_options() {
    let s = setup();
    let marker = s.mine.path().join("만들면 안 됨");
    let pattern = format!("--upload-pack=touch {}", marker.display());
    // core가 이런 이름을 막지만, git도 옵션으로 읽지 않는다(--end-of-options).
    let _ = backend().ls_remote(&s.repo(), "origin", &pattern, AUTH);
    assert!(!marker.exists());
}
