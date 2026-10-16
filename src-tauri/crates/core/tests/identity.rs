#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;
use std::sync::Arc;

use common::FakeGit;
use higit_core::github::{GitHubService, KEYCHAIN_SERVICE};
use higit_core::view::{ActionInput, ActionResult, ErrorCode, ErrorView, IdentityInput};
use higit_core::{Core, GitBackend};
use hosting::testing::FakeTransport;

fn core() -> (Arc<FakeGit>, Core) {
    let fake = Arc::new(FakeGit::new());
    let core = Core::with_backend(fake.clone() as Arc<dyn GitBackend>).unwrap();
    (fake, core)
}

fn set(name: &str, email: &str) -> ActionInput {
    ActionInput::IdentitySet(IdentityInput {
        name: name.into(),
        email: email.into(),
    })
}

#[test]
fn saves_trimmed_values_without_an_opened_project() {
    let (fake, core) = core();
    let result = core
        .execute(
            Path::new(""),
            &set("  김지원 ", " ji+won@example.com "),
            None,
        )
        .unwrap();
    assert_eq!(result, ActionResult::IdentitySaved);
    let ops = fake.global_ops.lock().unwrap().clone();
    assert_eq!(
        ops,
        vec![git::GlobalOp::SetIdentity {
            name: "김지원".into(),
            email: "ji+won@example.com".into()
        }]
    );
    let view = core.identity().unwrap();
    assert!(view.complete);
    assert_eq!(view.name.as_deref(), Some("김지원"));
}

#[test]
fn invalid_values_are_refused_before_git() {
    let (fake, core) = core();
    for (name, email) in [
        ("", "a@b.c"),
        ("-x", "a@b.c"),
        ("이름", "없음"),
        ("a\nb", "a@b.c"),
    ] {
        let err = core
            .execute(Path::new(""), &set(name, email), None)
            .unwrap_err();
        assert_eq!(
            ErrorView::from(&err).code,
            ErrorCode::IdentityInvalid,
            "{name:?}"
        );
    }
    assert!(fake.global_ops.lock().unwrap().is_empty());
}

#[test]
fn complete_needs_both_name_and_email() {
    let (fake, core) = core();
    assert!(!core.identity().unwrap().complete);
    *fake.identity.lock().unwrap() = git::GitIdentity {
        name: Some("이름".into()),
        email: None,
    };
    let view = core.identity().unwrap();
    assert!(!view.complete);
    assert_eq!(view.email, None);
    *fake.identity.lock().unwrap() = git::GitIdentity {
        name: Some("이름".into()),
        email: Some("a@b.c".into()),
    };
    assert!(core.identity().unwrap().complete);
}

/// `connected`: None이면 미연결, Some((로그인, 기억함)).
fn noreply_with(connected: Option<(&str, bool)>) -> Option<String> {
    let config = hosting::GitHubConfig::new(Some("Ov23liTESTCLIENT01"));
    let provider = hosting::GitHub::new(Arc::new(FakeTransport::new()), config.clone());
    let github = GitHubService::new(
        Arc::new(provider),
        &config,
        platform::Keychain::in_memory(KEYCHAIN_SERVICE),
        None,
    );
    if let Some((login, remembered)) = connected {
        github.connect_for_tests(login, "gho_TESTSECRET0123456789", remembered);
    }
    let core = Core::with_backend(Arc::new(FakeGit::new()) as Arc<dyn GitBackend>)
        .unwrap()
        .with_github_for_tests(github);
    core.identity().unwrap().noreply_email
}

#[test]
fn noreply_email_only_for_a_remembered_connection() {
    assert_eq!(
        noreply_with(Some(("octocat", true))).as_deref(),
        Some("1+octocat@users.noreply.github.com")
    );
    // 이번 실행뿐인 연결은 숫자 id를 저장하지 않는다.
    assert_eq!(noreply_with(Some(("octocat", false))), None);
    assert_eq!(noreply_with(None), None);
    // 로그인 모양이 아닌 값으로는 주소를 만들지 않는다.
    assert_eq!(noreply_with(Some(("bad login", true))), None);
    assert_eq!(noreply_with(Some(("-x", true))), None);
}
