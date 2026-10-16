#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use common::FakeGit;
use git::Remote;
use higit_core::view::{ErrorCode, ErrorView, HostingPage, OpenHow};
use higit_core::{Core, GitBackend};

fn core_with(remotes: &[(&str, &str)]) -> (Core, tempfile::TempDir) {
    let git = Arc::new(FakeGit::new());
    *git.remotes.lock().unwrap() = remotes
        .iter()
        .map(|(name, url)| Remote {
            name: (*name).into(),
            url: (*url).into(),
        })
        .collect();
    let core = Core::with_backend(git as Arc<dyn GitBackend>).unwrap();
    let dir = tempfile::tempdir().unwrap();
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    (core, dir)
}

#[test]
fn page_urls_come_from_constants_and_checked_remotes() {
    let (core, dir) = core_with(&[
        ("upstream", "https://github.com/other/hello.git"),
        ("origin", "git@github.com:octo/hello.git"),
    ]);
    assert_eq!(
        core.hosting_page_url(None, HostingPage::DeviceLogin)
            .unwrap(),
        "https://github.com/login/device"
    );
    assert_eq!(
        core.hosting_page_url(None, HostingPage::AppSettings)
            .unwrap(),
        "https://github.com/settings/applications"
    );
    assert_eq!(
        core.hosting_page_url(Some(dir.path()), HostingPage::PullRequest { number: 7 })
            .unwrap(),
        "https://github.com/octo/hello/pull/7"
    );
    let err = core
        .hosting_page_url(Some(dir.path()), HostingPage::PullRequest { number: 0 })
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
}

#[test]
fn pull_request_page_needs_an_opened_github_repo() {
    let (core, _dir) = core_with(&[("origin", "https://github.com/octo/hello")]);
    let other = tempfile::tempdir().unwrap();
    let err = core
        .hosting_page_url(Some(other.path()), HostingPage::PullRequest { number: 1 })
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    let err = core
        .hosting_page_url(None, HostingPage::PullRequest { number: 1 })
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);

    // GitHub가 아닌 원격(경로에 github.com이 있어도)은 쓰지 않는다.
    let (core, dir) = core_with(&[("origin", "https://evil.example/github.com/octo")]);
    let err = core
        .hosting_page_url(Some(dir.path()), HostingPage::PullRequest { number: 1 })
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NoRemote);
}

#[test]
fn default_core_is_not_configured() {
    let (core, _dir) = core_with(&[]);
    let status = core.github_status();
    assert!(!status.configured);
    assert!(!status.connected);
    let err = core.github_connect_start(None).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::GithubNotConfigured);
}
