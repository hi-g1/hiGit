//! GitHub에 올리기(`publish`, D-132)를 가짜 전송과 실제 git으로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::github::{GitHubService, KEYCHAIN_SERVICE};
use higit_core::view::{
    ActionInput, ActionResult, ErrorCode, ErrorView, OpenHow, PublishInput, Visibility,
};
use higit_core::{Core, GitBackend};
use hosting::testing::FakeTransport;

const TOKEN: &str = "gho_TESTSECRET0123456789";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
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

struct Fixture {
    core: Core,
    fake: Arc<FakeTransport>,
    dir: tempfile::TempDir,
}

/// `commit`이면 버전 하나, `connected`면 octocat으로 연결된 상태.
fn fixture(commit: bool, connected: bool) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    if commit {
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        git(dir.path(), &["add", "-A"]);
        git(dir.path(), &["commit", "-q", "-m", "a"]);
    }
    let fake = Arc::new(FakeTransport::new());
    let config = hosting::GitHubConfig::new(Some("Ov23liTESTCLIENT01"));
    let provider = hosting::GitHub::new(fake.clone(), config.clone());
    let github = GitHubService::new(
        Arc::new(provider),
        &config,
        platform::Keychain::in_memory(KEYCHAIN_SERVICE),
        None,
    );
    if connected {
        github.connect_for_tests("octocat", TOKEN, true);
    }
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .with_github_for_tests(github);
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    Fixture { core, fake, dir }
}

fn publish(
    f: &Fixture,
    name: &str,
    visibility: Visibility,
    ack: bool,
) -> higit_core::Result<ActionResult> {
    f.core.execute(
        f.dir.path(),
        &ActionInput::Publish(PublishInput {
            name: name.into(),
            description: Some("설명 \"따옴표\"".into()),
            visibility,
            acknowledge_public: ack,
            owner: None,
        }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn origin(dir: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(dir)
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
}

#[test]
fn creates_a_private_repository_and_links_origin() {
    let f = fixture(true, true);
    f.fake
        .push_json(201, r#"{"name":"hello","owner":{"login":"OctoCat"},"private":true,"clone_url":"https://evil.example/x.git"}"#);
    let result = publish(&f, " hello ", Visibility::Private, false).unwrap();
    assert_eq!(
        result,
        ActionResult::Published {
            owner: "OctoCat".into(),
            name: "hello".into()
        }
    );
    // 주소는 서버가 준 clone_url이 아니라 검사한 소유자·이름으로 만든다.
    assert_eq!(
        origin(f.dir.path()).as_deref(),
        Some("https://github.com/OctoCat/hello.git")
    );
    let reqs = f.fake.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].url, "https://api.github.com/user/repos");
    let body: serde_json::Value = serde_json::from_slice(reqs[0].body.as_deref().unwrap()).unwrap();
    assert_eq!(body["private"], true);
    assert_eq!(body["auto_init"], false);
    assert_eq!(body["description"], "설명 \"따옴표\"");
    assert!(!format!("{result:?}").contains(TOKEN));
}

#[test]
fn checks_before_sending_anything() {
    // 원격이 이미 있으면.
    let f = fixture(true, true);
    git(
        f.dir.path(),
        &["remote", "add", "origin", "https://github.com/a/b.git"],
    );
    assert_eq!(
        code(&publish(&f, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::PublishHasRemote
    );
    // 버전이 없으면.
    let f2 = fixture(false, true);
    assert_eq!(
        code(&publish(&f2, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::PublishNoVersions
    );
    // 이름 규칙·공개 재확인.
    let f3 = fixture(true, true);
    for bad in ["..", "a b", &"x".repeat(101), ""] {
        assert_eq!(
            code(&publish(&f3, bad, Visibility::Private, false).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad}"
        );
    }
    assert_eq!(
        code(&publish(&f3, "hello", Visibility::Public, false).unwrap_err()),
        ErrorCode::InvalidInput
    );
    // 연결 안 됨.
    let f4 = fixture(true, false);
    assert_eq!(
        code(&publish(&f4, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::GithubNotConnected
    );
    for f in [f, f2, f3, f4] {
        assert!(f.fake.requests().is_empty());
    }
}

#[test]
fn public_needs_the_extra_confirmation() {
    let f = fixture(true, true);
    f.fake.push_json(
        201,
        r#"{"name":"open","owner":{"login":"octocat"},"private":false}"#,
    );
    publish(&f, "open", Visibility::Public, true).unwrap();
    let body: serde_json::Value =
        serde_json::from_slice(f.fake.requests()[0].body.as_deref().unwrap()).unwrap();
    assert_eq!(body["private"], false);
}

#[test]
fn odd_or_refused_answers_leave_no_origin() {
    let f = fixture(true, true);
    // 연결된 계정이 아닌 소유자.
    f.fake
        .push_json(201, r#"{"name":"hello","owner":{"login":"someone-else"}}"#);
    assert_eq!(
        code(&publish(&f, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::GithubBadResponse
    );
    assert_eq!(origin(f.dir.path()), None);
    // 같은 이름이 있음, 그 밖의 422.
    f.fake.push_json(422, r#"{"message":"Repository creation failed.","errors":[{"message":"name already exists on this account"}]}"#);
    assert_eq!(
        code(&publish(&f, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::GithubRepoExists
    );
    f.fake.push_json(422, r#"{"message":"Repository creation failed.","errors":[{"message":"description control characters"}]}"#);
    assert_eq!(
        code(&publish(&f, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::GithubRepoInvalid
    );
    // 401이면 연결이 끊긴다.
    f.fake.push_json(401, "{}");
    assert_eq!(
        code(&publish(&f, "hello", Visibility::Private, false).unwrap_err()),
        ErrorCode::GithubExpired
    );
    assert_eq!(origin(f.dir.path()), None);
}

#[test]
fn draft_suggests_a_name_and_counts() {
    let f = fixture(true, true);
    let draft = f.core.queries().publish_draft(f.dir.path()).unwrap();
    assert_eq!(draft.versions, 1);
    assert_eq!(draft.remotes, 0);
    assert!(draft.connected && draft.configured);
    assert_eq!(draft.login.as_deref(), Some("octocat"));
    assert_eq!(draft.branch.as_deref(), Some("main"));
}

fn publish_to(f: &Fixture, owner: &str) -> higit_core::Result<ActionResult> {
    f.core.execute(
        f.dir.path(),
        &ActionInput::Publish(PublishInput {
            name: "tool".into(),
            description: None,
            visibility: Visibility::Private,
            acknowledge_public: false,
            owner: Some(owner.into()),
        }),
        None,
    )
}

#[test]
fn publishes_to_an_organization() {
    let f = fixture(true, true);
    f.fake.push_json(
        201,
        r#"{"name":"tool","owner":{"login":"My-Org","type":"Organization"},"private":true}"#,
    );
    assert_eq!(
        publish_to(&f, "my-org").unwrap(),
        ActionResult::Published {
            owner: "My-Org".into(),
            name: "tool".into()
        }
    );
    assert_eq!(
        f.fake.requests()[0].url,
        "https://api.github.com/orgs/my-org/repos"
    );
    assert_eq!(
        origin(f.dir.path()).as_deref(),
        Some("https://github.com/My-Org/tool.git")
    );
}

#[test]
fn organization_refusals() {
    // 조직이 앱을 막았거나 권한이 없음.
    let f = fixture(true, true);
    f.fake.push_json(
        403,
        r#"{"message":"Must have admin rights to Repository."}"#,
    );
    assert_eq!(
        code(&publish_to(&f, "my-org").unwrap_err()),
        ErrorCode::GithubOrgForbidden
    );
    // 다른 곳에 만들어졌다는 응답은 받지 않는다.
    f.fake
        .push_json(201, r#"{"name":"tool","owner":{"login":"someone"}}"#);
    assert_eq!(
        code(&publish_to(&f, "my-org").unwrap_err()),
        ErrorCode::GithubBadResponse
    );
    assert_eq!(origin(f.dir.path()), None);
    // 이름 중복은 그대로.
    f.fake.push_json(422, r#"{"message":"Repository creation failed.","errors":[{"message":"name already exists on this account"}]}"#);
    assert_eq!(
        code(&publish_to(&f, "my-org").unwrap_err()),
        ErrorCode::GithubRepoExists
    );
    // 조직 이름 규칙에 맞지 않으면 보내지 않는다.
    let g = fixture(true, true);
    for bad in ["-a", "a/b", &"a".repeat(40)] {
        assert_eq!(
            code(&publish_to(&g, bad).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad}"
        );
    }
    assert!(g.fake.requests().is_empty());
}
