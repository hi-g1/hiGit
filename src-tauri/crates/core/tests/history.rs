//! 버전 기록 노선도 조회(D-088)를 실제 git으로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ErrorCode, ErrorView, LogPage, LogQuery, LogScope, OpenHow, RefLabelKind};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
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

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", text]);
}

/// main: 처음 → 메인 하나 → 메인 둘, 기능(지금): 처음 → 기능 하나 → 기능 둘. 태그 v1은 처음.
fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
    commit(dir, "a.txt", "처음");
    git(dir, &["tag", "v1"]);
    git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "b.txt", "기능 하나");
    commit(dir, "b.txt", "기능 둘");
    git(dir, &["switch", "-q", "main"]);
    commit(dir, "c.txt", "메인 하나");
    commit(dir, "c.txt", "메인 둘");
    git(dir, &["switch", "-q", "기능"]);
    repo
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn page(core: &Core, dir: &Path, query: LogQuery) -> LogPage {
    core.queries().log(dir, query).unwrap()
}

fn summaries(page: &LogPage) -> Vec<&str> {
    page.commits.iter().map(|c| c.summary.as_str()).collect()
}

#[test]
fn all_branches_or_only_the_current_one() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    let all = page(
        &core,
        dir,
        LogQuery {
            limit: 50,
            ..Default::default()
        },
    );
    assert_eq!(all.commits.len(), 5);
    assert!(summaries(&all).contains(&"메인 둘"));
    assert_eq!(all.current_branch.as_deref(), Some("기능"));
    assert_eq!(all.default_branch.as_deref(), Some("main"));
    assert_eq!(
        all.default_tip.as_deref(),
        Some(git(dir, &["rev-parse", "main"]).as_str())
    );
    assert_eq!(
        all.head.as_deref(),
        Some(git(dir, &["rev-parse", "HEAD"]).as_str())
    );
    assert_eq!(all.tips.len(), 2);

    let current = page(
        &core,
        dir,
        LogQuery {
            limit: 50,
            scope: LogScope::Current,
            ..Default::default()
        },
    );
    assert_eq!(summaries(&current), ["기능 둘", "기능 하나", "처음"]);
    assert_eq!(current.tips, vec![git(dir, &["rev-parse", "HEAD"])]);
}

#[test]
fn later_pages_follow_the_first_page_tips() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    let whole = page(
        &core,
        dir,
        LogQuery {
            limit: 50,
            ..Default::default()
        },
    );
    let first = page(
        &core,
        dir,
        LogQuery {
            limit: 2,
            ..Default::default()
        },
    );
    assert!(first.has_more);
    // 그사이 새 버전이 생겨도 둘째 쪽은 첫 쪽의 끝점으로 읽어 겹치거나 빠지지 않는다.
    commit(dir, "d.txt", "그사이 새 버전");
    let mut read = summaries(&first)
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut skip = 2;
    loop {
        let next = page(
            &core,
            dir,
            LogQuery {
                skip,
                limit: 2,
                tips: Some(first.tips.clone()),
                ..Default::default()
            },
        );
        // 다음 쪽에는 첫 쪽에만 주는 값이 없다.
        assert!(next.head.is_none() && next.unpushed.is_empty());
        read.extend(next.commits.iter().map(|c| c.summary.clone()));
        if !next.has_more {
            break;
        }
        skip += 2;
    }
    assert_eq!(read, summaries(&whole));
}

#[test]
fn labels_are_only_for_versions_on_the_page() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    let first = page(
        &core,
        dir,
        LogQuery {
            limit: 2,
            ..Default::default()
        },
    );
    let names: Vec<(&str, RefLabelKind)> = first
        .labels
        .iter()
        .map(|l| (l.name.as_str(), l.kind))
        .collect();
    // 첫 쪽(위상 순서 위 둘)에 v1 태그가 붙은 처음 버전은 없다.
    assert!(!names.iter().any(|(n, _)| *n == "v1"));
    let whole = page(
        &core,
        dir,
        LogQuery {
            limit: 50,
            ..Default::default()
        },
    );
    let first_oid = git(dir, &["rev-parse", "v1"]);
    assert!(
        whole
            .labels
            .iter()
            .any(|l| l.name == "v1" && l.kind == RefLabelKind::Tag && l.oid == first_oid)
    );
    assert!(
        whole
            .labels
            .iter()
            .any(|l| l.name == "main" && l.kind == RefLabelKind::Branch)
    );
    assert!(
        whole
            .labels
            .iter()
            .any(|l| l.name == "기능" && l.kind == RefLabelKind::Branch)
    );
}

#[test]
fn unpushed_matches_rev_list() {
    let repo = repo();
    let dir = repo.path();
    // 기능의 upstream을 처음 버전에 둔다(원격 ref를 직접 만든다).
    git(
        dir,
        &["remote", "add", "origin", "https://example.com/x.git"],
    );
    let base = git(dir, &["rev-parse", "v1"]);
    git(dir, &["update-ref", "refs/remotes/origin/기능", &base]);
    git(dir, &["config", "branch.기능.remote", "origin"]);
    git(dir, &["config", "branch.기능.merge", "refs/heads/기능"]);
    let core = core(dir);
    let first = page(
        &core,
        dir,
        LogQuery {
            limit: 50,
            ..Default::default()
        },
    );
    let expected: Vec<String> = git(dir, &["rev-list", "origin/기능..HEAD"])
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(first.unpushed, expected);
    assert_eq!(first.unpushed.len(), 2);
    // 떨어진 HEAD: 지금 브랜치 없음, 안 올림 없음.
    git(dir, &["switch", "-q", "--detach", "HEAD~1"]);
    let detached = page(
        &core,
        dir,
        LogQuery {
            limit: 50,
            ..Default::default()
        },
    );
    assert!(detached.current_branch.is_none());
    assert!(detached.unpushed.is_empty());
}

#[test]
fn bad_tips_and_unopened_repos_are_refused() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    let code = |q: LogQuery| ErrorView::from(&core.queries().log(dir, q).unwrap_err()).code;
    assert_eq!(
        code(LogQuery {
            limit: 2,
            tips: Some(vec!["--all".into()]),
            ..Default::default()
        }),
        ErrorCode::InvalidInput
    );
    let many = vec![git(dir, &["rev-parse", "HEAD"]); 202];
    assert_eq!(
        code(LogQuery {
            limit: 2,
            tips: Some(many),
            ..Default::default()
        }),
        ErrorCode::InvalidInput
    );
    let other = tempfile::tempdir().unwrap();
    let err = core
        .queries()
        .log(other.path(), LogQuery::default())
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
}

#[test]
fn an_empty_repo_has_an_empty_page() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "main"]);
    let core = core(repo.path());
    let first = page(
        &core,
        repo.path(),
        LogQuery {
            limit: 50,
            ..Default::default()
        },
    );
    assert!(first.commits.is_empty() && first.tips.is_empty() && first.head.is_none());
}
