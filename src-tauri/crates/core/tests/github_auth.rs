//! 원격 작업의 인증 방식(D-111): 조건이 모두 맞을 때만 hiGit helper를 붙인다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use common::{FakeGit, fake_failure};
use git::Remote;
use higit_core::github::{GitHubService, KEYCHAIN_ACCOUNT, KEYCHAIN_SERVICE};
use higit_core::view::{ActionInput, ErrorCode, ErrorView, FetchInput, OpenHow};
use higit_core::{Core, GitBackend};
use hosting::testing::FakeTransport;

const TOKEN: &str = "gho_TESTSECRET0123456789";

struct Fixture {
    core: Core,
    git: Arc<FakeGit>,
    fake: Arc<FakeTransport>,
    dir: tempfile::TempDir,
}

fn service(
    fake: &Arc<FakeTransport>,
    keychain: platform::Keychain,
    data: Option<&std::path::Path>,
) -> GitHubService {
    let config = hosting::GitHubConfig::new(Some("Ov23liTESTCLIENT01"));
    let provider = hosting::GitHub::new(fake.clone(), config.clone());
    GitHubService::new(Arc::new(provider), &config, keychain, data)
}

/// `url`을 origin으로 둔 저장소. `connected`: None이면 미연결, Some(remembered).
fn fixture(url: &str, connected: Option<bool>) -> Fixture {
    let git = Arc::new(FakeGit::new());
    *git.remotes.lock().unwrap() = vec![Remote {
        name: "origin".into(),
        url: url.into(),
    }];
    let fake = Arc::new(FakeTransport::new());
    let github = service(&fake, platform::Keychain::in_memory(KEYCHAIN_SERVICE), None);
    if let Some(remembered) = connected {
        github.connect_for_tests("octocat", TOKEN, remembered);
    }
    let core = Core::with_backend(git.clone() as Arc<dyn GitBackend>)
        .unwrap()
        .with_github_for_tests(github);
    let dir = tempfile::tempdir().unwrap();
    *git.repo.lock().unwrap() = Some(higit_core::RepoPath::new(dir.path()));
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    Fixture {
        core,
        git,
        fake,
        dir,
    }
}

fn fetch(f: &Fixture, background: bool) -> Result<(), higit_core::CoreError> {
    f.core
        .execute(
            f.dir.path(),
            &ActionInput::Fetch(FetchInput { background }),
            None,
        )
        .map(|_| ())
}

/// 마지막 Fetch 호출이 helper를 썼는지.
fn used_helper(git: &FakeGit) -> bool {
    let calls = git.calls.lock().unwrap();
    let last = calls
        .iter()
        .rev()
        .find(|c| c.starts_with("Fetch") || c.starts_with("Push"))
        .expect("원격 호출");
    assert!(!last.contains(TOKEN), "{last}");
    last.contains("auth: Helper")
}

#[test]
fn helper_only_when_every_condition_holds() {
    let f = fixture("https://github.com/octo/hello.git", Some(true));
    fetch(&f, false).unwrap();
    assert!(used_helper(&f.git));

    let cases: &[(&str, Option<bool>)] = &[
        ("git@github.com:octo/hello.git", Some(true)),
        ("ssh://git@github.com/octo/hello.git", Some(true)),
        ("https://gitlab.com/octo/hello.git", Some(true)),
        ("https://evil.example/github.com/hello", Some(true)),
        ("https://github.com/octo/hello.git", None),
        // 키체인을 못 써 이번 실행에만 둔 토큰은 helper 프로세스가 읽을 수 없다.
        ("https://github.com/octo/hello.git", Some(false)),
    ];
    for (url, connected) in cases {
        let f = fixture(url, *connected);
        fetch(&f, false).unwrap();
        assert!(!used_helper(&f.git), "{url} {connected:?}");
    }
}

#[test]
fn setting_off_never_uses_the_helper() {
    let f = fixture("https://github.com/octo/hello.git", Some(true));
    f.core.set_settings(&higit_core::view::AppSettingsPatch {
        git_helper: Some(false),
        ..Default::default()
    });
    fetch(&f, false).unwrap();
    assert!(!used_helper(&f.git));
    fetch(&f, true).unwrap();
    assert!(!used_helper(&f.git));
    f.core.set_settings(&higit_core::view::AppSettingsPatch {
        git_helper: Some(true),
        ..Default::default()
    });
    fetch(&f, false).unwrap();
    assert!(used_helper(&f.git));
}

#[test]
fn local_transport_settings_turn_helper_off() {
    for key in [
        "http.sslverify",
        "http.proxy",
        "http.https://github.com/.extraheader",
        "remote.origin.proxy",
    ] {
        let f = fixture("https://github.com/octo/hello.git", Some(true));
        *f.git.local_config.lock().unwrap() = vec![(key.into(), "x".into())];
        fetch(&f, false).unwrap();
        assert!(!used_helper(&f.git), "{key}");
    }
    // 연결과 상관없는 설정은 괜찮다.
    let f = fixture("https://github.com/octo/hello.git", Some(true));
    *f.git.local_config.lock().unwrap() = vec![("core.autocrlf".into(), "false".into())];
    fetch(&f, false).unwrap();
    assert!(used_helper(&f.git));
}

#[test]
fn push_url_to_another_host_turns_helper_off() {
    let f = fixture("https://github.com/octo/hello.git", Some(true));
    *f.git.push_urls.lock().unwrap() = vec![(
        "origin".into(),
        vec![
            "https://github.com/octo/hello.git".into(),
            "https://evil.example/x.git".into(),
        ],
    )];
    fetch(&f, false).unwrap();
    assert!(!used_helper(&f.git));
}

#[test]
fn background_check_does_not_open_the_keychain() {
    let data = tempfile::tempdir().unwrap();
    let keychain = platform::Keychain::in_memory(KEYCHAIN_SERVICE);
    let fake = Arc::new(FakeTransport::new());
    // 지난 실행에 연결했다: 키체인과 상태 파일에 있다.
    service(&fake, keychain.clone(), Some(data.path())).connect_for_tests("octocat", TOKEN, true);

    let git = Arc::new(FakeGit::new());
    *git.remotes.lock().unwrap() = vec![Remote {
        name: "origin".into(),
        url: "https://github.com/octo/hello.git".into(),
    }];
    let core = Core::with_backend(git.clone() as Arc<dyn GitBackend>)
        .unwrap()
        .with_github_for_tests(service(&fake, keychain, Some(data.path())));
    let dir = tempfile::tempdir().unwrap();
    *git.repo.lock().unwrap() = Some(higit_core::RepoPath::new(dir.path()));
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    let f = Fixture {
        core,
        git,
        fake,
        dir,
    };
    assert!(f.core.github_status().connected);
    // 아직 키체인을 읽지 않았으니 저절로 하는 확인은 사용자 기본.
    fetch(&f, true).unwrap();
    assert!(!used_helper(&f.git));
    // 사람이 누른 작업은 키체인을 읽고 helper를 쓴다. 그 뒤에는 저절로 하는 확인도.
    fetch(&f, false).unwrap();
    assert!(used_helper(&f.git));
    fetch(&f, true).unwrap();
    assert!(used_helper(&f.git));
}

#[test]
fn auth_failure_with_revoked_token_disconnects() {
    let f = fixture("https://github.com/octo/hello.git", Some(true));
    *f.git.run_error.lock().unwrap() = Some(fake_failure(
        "remote: Invalid username or token.\nfatal: Authentication failed for 'https://github.com/octo/hello.git/'\n",
    ));
    f.fake.push_json(401, r#"{"message":"Bad credentials"}"#);
    let err = fetch(&f, false).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::GithubExpired);
    assert!(!f.core.github_status().connected);
}

#[test]
fn auth_failure_with_working_token_keeps_git_error() {
    let f = fixture("https://github.com/octo/hello.git", Some(true));
    *f.git.run_error.lock().unwrap() = Some(fake_failure(
        "fatal: unable to access 'https://github.com/octo/hello.git/': The requested URL returned error: 403\n",
    ));
    f.fake.push_json(200, r#"{"login":"octocat","id":1}"#);
    let err = fetch(&f, false).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::HttpForbidden);
    assert!(f.core.github_status().connected);

    // 네트워크가 막혀 확인하지 못해도 원래 오류 그대로.
    *f.git.run_error.lock().unwrap() = Some(fake_failure(
        "fatal: Authentication failed for 'https://github.com/octo/hello.git/'\n",
    ));
    let err = fetch(&f, false).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::HttpAuthFailed);
    assert!(f.core.github_status().connected);
}

#[test]
fn keychain_account_constants() {
    assert_eq!(KEYCHAIN_SERVICE, "dev.higit.desktop");
    assert_eq!(KEYCHAIN_ACCOUNT, "github.com");
}
