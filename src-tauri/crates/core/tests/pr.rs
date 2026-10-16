//! 검토 요청 조회(D-112, D-113): 대상 원격·브랜치, 목록·상세는 가짜 전송, 초안은 로컬만.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::github::{GitHubService, KEYCHAIN_SERVICE};
use higit_core::view::{
    ErrorCode, ErrorView, HostingProviderView, MergeableView, OpenHow, PrListRequest,
    PrStateFilterView, PrStateView,
};
use higit_core::{Core, GitBackend};
use hosting::testing::FakeTransport;

const TOKEN: &str = "gho_TESTSECRET0123456789";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "user.name=민지", "-c", "user.email=m@example.com"])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn commit(dir: &Path, file: &str, message: &str) {
    std::fs::write(dir.join(file), message).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

/// main에 버전 하나, `url`을 origin으로 두고 origin/main·origin/develop을 흉내 낸 저장소.
fn repo(url: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q", "-b", "main"]);
    commit(p, "a.txt", "첫 버전");
    if let Some(url) = url {
        git(p, &["remote", "add", "origin", url]);
        git(p, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git(p, &["update-ref", "refs/remotes/origin/develop", "HEAD"]);
        git(p, &["config", "branch.main.remote", "origin"]);
        git(p, &["config", "branch.main.merge", "refs/heads/main"]);
    }
    dir
}

struct Fixture {
    core: Core,
    fake: Arc<FakeTransport>,
}

fn core(dir: &Path, connected: bool) -> Fixture {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
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
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .with_github_for_tests(github);
    core.open_project(dir, OpenHow::Picked).unwrap();
    Fixture { core, fake }
}

fn code(err: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(err).code
}

#[test]
fn context_reads_local_state_only() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    let ctx = f.core.queries().hosting_context(dir.path()).unwrap();
    assert_eq!(ctx.provider, Some(HostingProviderView::Github));
    assert_eq!(ctx.repo.as_deref(), Some("octo/hello"));
    assert!(ctx.configured && ctx.connected);
    assert_eq!(ctx.branch.as_deref(), Some("main"));
    assert_eq!(ctx.head.as_deref(), Some("main"));
    assert_eq!(ctx.base.as_deref(), Some("main"));
    assert!(ctx.is_default_branch);
    assert!(f.fake.requests().is_empty(), "네트워크를 쓰지 않는다");

    let ssh = repo(Some("git@github.com:octo/hello.git"));
    let f = core(ssh.path(), false);
    let ctx = f.core.queries().hosting_context(ssh.path()).unwrap();
    assert_eq!(ctx.provider, Some(HostingProviderView::Github));
    assert!(!ctx.connected);

    for url in [
        Some("https://gitlab.com/octo/hello.git"),
        Some("https://evil.example/github.com/x"),
        None,
    ] {
        let dir = repo(url);
        let f = core(dir.path(), true);
        let ctx = f.core.queries().hosting_context(dir.path()).unwrap();
        assert_eq!(ctx.provider, None, "{url:?}");
        assert_eq!(ctx.repo, None);
        assert_eq!(ctx.branch.as_deref(), Some("main"));
    }
}

#[test]
fn push_url_decides_the_repo() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    git(
        dir.path(),
        &[
            "config",
            "remote.origin.pushurl",
            "https://gitlab.com/octo/hello.git",
        ],
    );
    let f = core(dir.path(), true);
    let ctx = f.core.queries().hosting_context(dir.path()).unwrap();
    assert_eq!(ctx.provider, None);
    let err = f.core.queries().pr_draft(dir.path()).unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotGithubRemote);
}

#[test]
fn draft_counts_and_suggests_title() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let p = dir.path();
    git(p, &["switch", "-q", "-c", "feature/login-page"]);
    commit(p, "b.txt", "로그인 화면 추가");
    let f = core(p, true);
    let draft = f.core.queries().pr_draft(p).unwrap();
    assert_eq!(draft.branch, "feature/login-page");
    assert_eq!(draft.head, None, "아직 올리지 않았다");
    assert_eq!(draft.base, "main");
    assert_eq!(draft.base_options, ["main", "develop"]);
    assert_eq!(draft.ahead, 1);
    assert_eq!(draft.unpushed, 1);
    assert_eq!(draft.suggestion_title, "로그인 화면 추가");

    commit(p, "c.txt", "두 번째");
    // 올린 것처럼: upstream을 지금 버전에 둔다.
    git(
        p,
        &[
            "update-ref",
            "refs/remotes/origin/feature/login-page",
            "HEAD",
        ],
    );
    git(p, &["config", "branch.feature/login-page.remote", "origin"]);
    git(
        p,
        &[
            "config",
            "branch.feature/login-page.merge",
            "refs/heads/feature/login-page",
        ],
    );
    let draft = f.core.queries().pr_draft(p).unwrap();
    assert_eq!(draft.head.as_deref(), Some("feature/login-page"));
    assert_eq!(draft.ahead, 2);
    assert_eq!(draft.unpushed, 0);
    assert_eq!(draft.suggestion_title, "feature login page");
    // 지금 브랜치(올라간 이름)는 대상 후보에서 뺀다.
    assert_eq!(draft.base_options, ["main", "develop"]);
    assert!(f.fake.requests().is_empty());
}

fn pull_json(number: u64, head: &str, owner: &str) -> String {
    format!(
        r#"{{"number":{number},"title":"제목 {number}","state":"open","draft":false,"merged_at":null,
           "head":{{"ref":"{head}","sha":"{sha}","repo":{{"owner":{{"login":"{owner}"}}}}}},
           "base":{{"ref":"main","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
           "user":{{"login":"mona"}},"created_at":"2026-10-01T00:00:00Z",
           "updated_at":"2026-10-02T00:00:00Z","mergeable_state":"behind"}}"#,
        sha = "a".repeat(40)
    )
}

#[test]
fn list_and_detail_go_through_the_service() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    f.fake.push_json(
        200,
        &format!(
            "[{},{}]",
            pull_json(3, "main", "octo"),
            pull_json(4, "main", "forker")
        ),
    );
    let list = f
        .core
        .queries()
        .pr_list(
            dir.path(),
            &PrListRequest {
                state: PrStateFilterView::Open,
                branch: Some("main".into()),
                page: 0,
            },
        )
        .unwrap();
    assert_eq!(list.items.len(), 2);
    assert!(list.items[0].is_current_branch);
    assert!(!list.items[1].is_current_branch, "포크의 같은 이름 브랜치");
    assert_eq!(list.items[1].head_owner.as_deref(), Some("forker"));
    assert_eq!(list.items[0].state, PrStateView::Open);
    let reqs = f.fake.requests();
    assert_eq!(
        reqs[0].url,
        "https://api.github.com/repos/octo/hello/pulls?state=open&per_page=30&page=1&sort=updated&direction=desc&head=octo%3Amain"
    );

    f.fake.push_json(200, &pull_json(3, "main", "octo"));
    f.fake.push_json(403, "{}");
    // 리뷰를 볼 수 없어도(403) 상세는 그대로다(D-134).
    f.fake.push_json(403, "{}");
    let detail = f.core.queries().pr_detail(dir.path(), 3).unwrap();
    assert_eq!(detail.row.number, 3);
    assert_eq!(detail.mergeable, MergeableView::Behind);
    assert_eq!(detail.checks, None);
    let shown = format!("{list:?} {detail:?}");
    assert!(!shown.contains(TOKEN));
}

#[test]
fn list_needs_connection_and_drops_it_on_401() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), false);
    let request = PrListRequest {
        state: PrStateFilterView::All,
        branch: None,
        page: 1,
    };
    let err = f.core.queries().pr_list(dir.path(), &request).unwrap_err();
    assert_eq!(code(&err), ErrorCode::GithubNotConnected);

    let f = core(dir.path(), true);
    f.fake.push_json(401, r#"{"message":"Bad credentials"}"#);
    let err = f.core.queries().pr_list(dir.path(), &request).unwrap_err();
    assert_eq!(code(&err), ErrorCode::GithubExpired);
    assert!(!f.core.github_status().connected);

    let err = f
        .core
        .queries()
        .pr_list(
            dir.path(),
            &PrListRequest {
                branch: Some("bad name".into()),
                ..request
            },
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);
}

#[test]
fn queries_need_an_opened_repo() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    let other = repo(Some("https://github.com/octo/other.git"));
    for err in [
        f.core.queries().hosting_context(other.path()).unwrap_err(),
        f.core.queries().pr_draft(other.path()).unwrap_err(),
        f.core.queries().pr_detail(other.path(), 1).unwrap_err(),
    ] {
        assert_eq!(code(&err), ErrorCode::NotOpened);
    }
}

/// 기능 브랜치를 만들고 버전 하나를 더한 뒤 올린 것처럼 둔다.
fn feature_pushed(p: &Path) {
    git(p, &["switch", "-q", "-c", "feature/x"]);
    commit(p, "b.txt", "기능");
    git(p, &["update-ref", "refs/remotes/origin/feature/x", "HEAD"]);
    git(p, &["config", "branch.feature/x.remote", "origin"]);
    git(
        p,
        &["config", "branch.feature/x.merge", "refs/heads/feature/x"],
    );
}

fn create(title: &str, base: &str) -> higit_core::view::ActionInput {
    higit_core::view::ActionInput::PrCreate(higit_core::view::PrCreateInput {
        title: title.into(),
        body: "본문\n\"따옴표\"".into(),
        base: base.into(),
        draft: true,
    })
}

#[test]
fn pr_create_sends_remote_branch_as_head() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    feature_pushed(dir.path());
    let f = core(dir.path(), true);
    f.fake.push_json(201, &pull_json(12, "feature/x", "octo"));
    let result = f
        .core
        .execute(dir.path(), &create(" 기능 추가 ", "main"), None)
        .unwrap();
    assert_eq!(
        result,
        higit_core::view::ActionResult::PrCreated { number: 12 }
    );
    let reqs = f.fake.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].url, "https://api.github.com/repos/octo/hello/pulls");
    let sent: serde_json::Value = serde_json::from_slice(reqs[0].body.as_ref().unwrap()).unwrap();
    assert_eq!(sent["title"], "기능 추가");
    assert_eq!(sent["head"], "feature/x");
    assert_eq!(sent["base"], "main");
    assert_eq!(sent["body"], "본문\n\"따옴표\"");
    assert_eq!(sent["draft"], true);
}

#[test]
fn pr_create_refuses_before_sending() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let p = dir.path();
    let f = core(p, true);
    // 기본 브랜치에서는 만들지 않는다.
    let err = f
        .core
        .execute(p, &create("t", "develop"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::PrSameBranch);

    // 올린 적 없는 브랜치
    git(p, &["switch", "-q", "-c", "local-only"]);
    commit(p, "c.txt", "로컬");
    let err = f.core.execute(p, &create("t", "main"), None).unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotPushed);

    // 올린 뒤 버전을 더했다
    git(p, &["switch", "-q", "main"]);
    feature_pushed(p);
    commit(p, "d.txt", "안 올림");
    let err = f.core.execute(p, &create("t", "main"), None).unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotPushed);
    git(p, &["update-ref", "refs/remotes/origin/feature/x", "HEAD"]);

    for (title, base, want) in [
        ("", "main", ErrorCode::InvalidInput),
        ("   ", "main", ErrorCode::InvalidInput),
        (&"가".repeat(86)[..], "main", ErrorCode::InvalidInput),
        ("a\0b", "main", ErrorCode::InvalidInput),
        ("a\nb", "main", ErrorCode::InvalidInput),
        ("t", "없는-브랜치", ErrorCode::InvalidInput),
        ("t", "bad name", ErrorCode::InvalidInput),
        ("t", "feature/x", ErrorCode::PrSameBranch),
    ] {
        let err = f.core.execute(p, &create(title, base), None).unwrap_err();
        assert_eq!(code(&err), want, "{title:?} {base:?}");
    }
    assert!(
        f.fake.requests().is_empty(),
        "검사에서 걸리면 보내지 않는다"
    );
}

#[test]
fn pr_create_maps_server_answers() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    feature_pushed(dir.path());
    let cases = [
        (
            422,
            r#"{"errors":[{"message":"A pull request already exists for octo:feature/x."}]}"#,
            ErrorCode::PrAlreadyExists,
        ),
        (
            422,
            r#"{"errors":[{"message":"No commits between main and feature/x"}]}"#,
            ErrorCode::PrNoDifference,
        ),
        (
            422,
            r#"{"errors":[{"code":"custom"}]}"#,
            ErrorCode::PrInvalid,
        ),
        (500, "oops", ErrorCode::GithubUnavailable),
    ];
    for (status, body, want) in cases {
        let f = core(dir.path(), true);
        f.fake.push_json(status, body);
        let err = f
            .core
            .execute(dir.path(), &create("t", "main"), None)
            .unwrap_err();
        assert_eq!(code(&err), want, "{body}");
        assert!(!format!("{err:?} {:?}", ErrorView::from(&err)).contains(TOKEN));
    }
    let f = core(dir.path(), true);
    f.fake.push_json(401, "{}");
    let err = f
        .core
        .execute(dir.path(), &create("t", "main"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::GithubExpired);
    assert!(!f.core.github_status().connected);

    let f = core(dir.path(), false);
    let err = f
        .core
        .execute(dir.path(), &create("t", "main"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::GithubNotConnected);
}

#[test]
fn pr_create_needs_a_difference_and_a_github_remote() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let p = dir.path();
    git(p, &["switch", "-q", "-c", "same"]);
    git(p, &["update-ref", "refs/remotes/origin/same", "HEAD"]);
    git(p, &["config", "branch.same.remote", "origin"]);
    git(p, &["config", "branch.same.merge", "refs/heads/same"]);
    let f = core(p, true);
    let err = f.core.execute(p, &create("t", "main"), None).unwrap_err();
    assert_eq!(code(&err), ErrorCode::PrNoDifference);

    let gitlab = repo(Some("https://gitlab.com/octo/hello.git"));
    feature_pushed(gitlab.path());
    let f = core(gitlab.path(), true);
    let err = f
        .core
        .execute(gitlab.path(), &create("t", "main"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotGithubRemote);
}

#[test]
fn repo_info_decides_buttons() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    let q = f.core.queries();
    f.fake.push_json(
        200,
        r#"{"permissions":{"push":true},"allow_merge_commit":false,"allow_squash_merge":true,"allow_rebase_merge":true}"#,
    );
    f.fake.push_json(200, r#"{"permissions":{"push":false}}"#);
    f.fake.push_json(
        200,
        r#"{"permissions":{"push":true},"allow_merge_commit":false,"allow_squash_merge":false,"allow_rebase_merge":true,"archived":true}"#,
    );
    use higit_core::view::MergeMethodView as M;
    let mine = q.pr_repo(dir.path()).unwrap();
    assert!(mine.can_push && mine.can_merge && !mine.rebase_only);
    assert_eq!(mine.merge_methods, vec![M::Squash]);
    assert_eq!(mine.viewer.as_deref(), Some("octocat"));
    // 쓰기 권한이 없어 방법 칸이 없으면 합친 버전 만들기 하나(서버가 정함).
    let theirs = q.pr_repo(dir.path()).unwrap();
    assert!(!theirs.can_push && theirs.merge_methods == vec![M::Merge]);
    let rebase = q.pr_repo(dir.path()).unwrap();
    assert!(rebase.rebase_only && rebase.merge_methods.is_empty() && rebase.archived);
    assert_eq!(
        f.fake.requests()[0].url,
        "https://api.github.com/repos/octo/hello"
    );
}

fn comment_json(n: u32) -> String {
    let items: Vec<String> = (0..n)
        .map(|i| {
            format!(
                r#"{{"id":{i},"user":{{"login":"{}"}},"created_at":"2026-10-01T00:00:00Z","body":"댓글 {i}"}}"#,
                if i % 2 == 0 { "octocat" } else { "mona" }
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[test]
fn comments_read_the_last_pages() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    let q = f.core.queries();
    // 0개: 첫 쪽 하나.
    f.fake.push_json(200, "[]");
    let none = q.pr_comments(dir.path(), 7, 0).unwrap();
    assert!(none.items.is_empty() && !none.partial);
    // 20개: 첫 쪽만.
    f.fake.push_json(200, &comment_json(20));
    let full = q.pr_comments(dir.path(), 7, 20).unwrap();
    assert_eq!(full.items.len(), 20);
    assert!(!full.partial);
    assert!(full.items[0].mine && !full.items[1].mine);
    // 45개: 셋째 쪽(5개)과 둘째 쪽(20개)에서 최근 20개.
    f.fake.push_json(200, &comment_json(5));
    f.fake.push_json(200, &comment_json(20));
    let many = q.pr_comments(dir.path(), 7, 45).unwrap();
    assert_eq!(many.items.len(), 20);
    assert!(many.partial);
    assert_eq!(many.items[19].body, "댓글 4");
    let urls: Vec<String> = f.fake.requests().iter().map(|r| r.url.clone()).collect();
    let base = "https://api.github.com/repos/octo/hello/issues/7/comments?per_page=20&page=";
    assert_eq!(
        urls,
        vec![
            format!("{base}1"),
            format!("{base}1"),
            format!("{base}3"),
            format!("{base}2")
        ]
    );
    assert!(q.pr_comments(dir.path(), 0, 3).is_err());
}

fn comment(number: u32, body: &str) -> higit_core::view::ActionInput {
    higit_core::view::ActionInput::PrComment(higit_core::view::PrCommentInput {
        number,
        body: body.into(),
    })
}

#[test]
fn pr_comment_sends_trimmed_body_once() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    f.fake.push_json(201, r#"{"id":99}"#);
    let result = f
        .core
        .execute(
            dir.path(),
            &comment(7, "  좋아요 \"고마워요\" \u{1F600}\n"),
            None,
        )
        .unwrap();
    assert_eq!(
        result,
        higit_core::view::ActionResult::PrCommented { number: 7 }
    );
    let reqs = f.fake.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(
        reqs[0].url,
        "https://api.github.com/repos/octo/hello/issues/7/comments"
    );
    let body: serde_json::Value = serde_json::from_slice(reqs[0].body.as_ref().unwrap()).unwrap();
    assert_eq!(body["body"], "좋아요 \"고마워요\" \u{1F600}");
}

#[test]
fn pr_comment_refuses_before_sending_and_drops_on_401() {
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    for (number, body) in [
        (7, "   ".to_string()),
        (7, "a\0b".to_string()),
        (7, "a".repeat(64 * 1024 + 1)),
        (0, "좋아요".to_string()),
    ] {
        let err = f
            .core
            .execute(dir.path(), &comment(number, &body), None)
            .unwrap_err();
        assert_eq!(code(&err), ErrorCode::InvalidInput);
    }
    assert!(f.fake.requests().is_empty());
    f.fake.push_json(401, "{}");
    let err = f
        .core
        .execute(dir.path(), &comment(7, "좋아요"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::GithubExpired);
    assert!(!format!("{err:?}").contains(TOKEN));
    assert!(!f.core.github_status().connected);
}

fn merge(
    number: u32,
    method: higit_core::view::MergeMethodView,
    head: &str,
) -> higit_core::view::ActionInput {
    higit_core::view::ActionInput::PrMerge(higit_core::view::PrMergeInput {
        number,
        method,
        expect_head: head.into(),
    })
}

fn confirmed(
    f: &Fixture,
    dir: &Path,
    input: &higit_core::view::ActionInput,
) -> Result<higit_core::view::ActionResult, higit_core::CoreError> {
    let token = f.core.preview(dir, input).unwrap().token;
    f.core.execute(dir, input, Some(token))
}

const BOTH: &str =
    r#"{"permissions":{"push":true},"allow_merge_commit":true,"allow_squash_merge":true}"#;

#[test]
fn pr_merge_needs_confirmation_and_sends_the_seen_head() {
    use higit_core::view::{ActionResult, MergeMethodView as M};
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    let head = "ab".repeat(20);
    let input = merge(5, M::Squash, &head);
    // 확인 없이 실행하면 아무것도 보내지 않는다.
    let err = f.core.execute(dir.path(), &input, None).unwrap_err();
    assert_eq!(code(&err), ErrorCode::PreviewRequired);
    assert!(f.fake.requests().is_empty());
    // 다른 입력의 토큰은 쓸 수 없다.
    let token = f
        .core
        .preview(dir.path(), &merge(5, M::Merge, &head))
        .unwrap()
        .token;
    let err = f.core.execute(dir.path(), &input, Some(token)).unwrap_err();
    assert_eq!(code(&err), ErrorCode::PreviewMismatch);
    f.fake.push_json(200, BOTH);
    f.fake
        .push_json(200, r#"{"sha":"x","merged":true,"message":"merged"}"#);
    assert_eq!(
        confirmed(&f, dir.path(), &input).unwrap(),
        ActionResult::PrMerged { number: 5 }
    );
    let reqs = f.fake.requests();
    assert_eq!(reqs.len(), 2);
    assert_eq!(reqs[1].method, hosting::Method::Put);
    assert_eq!(
        reqs[1].url,
        "https://api.github.com/repos/octo/hello/pulls/5/merge"
    );
    let body: serde_json::Value = serde_json::from_slice(reqs[1].body.as_ref().unwrap()).unwrap();
    assert_eq!(body["sha"], head.as_str());
    assert_eq!(body["merge_method"], "squash");
}

#[test]
fn pr_merge_refusals() {
    use higit_core::view::MergeMethodView as M;
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    let head = "c".repeat(40);
    // 저장소가 허용하지 않은 방법은 합치기 요청을 보내지 않는다.
    f.fake.push_json(
        200,
        r#"{"permissions":{"push":true},"allow_merge_commit":true,"allow_squash_merge":false}"#,
    );
    let err = confirmed(&f, dir.path(), &merge(5, M::Squash, &head)).unwrap_err();
    assert_eq!(code(&err), ErrorCode::PrNotMergeable);
    assert_eq!(f.fake.requests().len(), 1);
    for (status, body, want) in [
        (
            405,
            r#"{"message":"Pull Request is not mergeable"}"#,
            ErrorCode::PrNotMergeable,
        ),
        (
            409,
            r#"{"message":"Head branch was modified."}"#,
            ErrorCode::PrHeadChanged,
        ),
        (
            200,
            r#"{"merged":false,"message":"nope"}"#,
            ErrorCode::PrNotMergeable,
        ),
    ] {
        f.fake.push_json(200, BOTH);
        f.fake.push_json(status, body);
        let err = confirmed(&f, dir.path(), &merge(5, M::Merge, &head)).unwrap_err();
        assert_eq!(code(&err), want, "{status}");
    }
    // 입력 검사는 보내기 전에.
    let sent = f.fake.requests().len();
    for input in [
        merge(0, M::Merge, &head),
        merge(5, M::Merge, "abc"),
        merge(5, M::Merge, &"g".repeat(40)),
    ] {
        let err = confirmed(&f, dir.path(), &input).unwrap_err();
        assert_eq!(code(&err), ErrorCode::InvalidInput);
    }
    assert_eq!(f.fake.requests().len(), sent);
}

#[test]
fn pr_close_and_reopen_send_state_only() {
    use higit_core::view::{ActionInput, ActionResult, PrNumberInput};
    let dir = repo(Some("https://github.com/octo/hello.git"));
    let f = core(dir.path(), true);
    f.fake.push_json(200, r#"{"number":5,"state":"closed"}"#);
    f.fake.push_json(200, r#"{"number":5,"state":"open"}"#);
    f.fake.push_json(422, r#"{"message":"Validation Failed","errors":[{"message":"state cannot be changed. The head branch was deleted."}]}"#);
    let close = ActionInput::PrClose(PrNumberInput { number: 5 });
    let reopen = ActionInput::PrReopen(PrNumberInput { number: 5 });
    assert_eq!(
        f.core.execute(dir.path(), &close, None).unwrap(),
        ActionResult::PrClosed { number: 5 }
    );
    assert_eq!(
        f.core.execute(dir.path(), &reopen, None).unwrap(),
        ActionResult::PrReopened { number: 5 }
    );
    let err = f.core.execute(dir.path(), &reopen, None).unwrap_err();
    assert_eq!(code(&err), ErrorCode::PrNotOpen);
    let reqs = f.fake.requests();
    assert!(reqs.iter().all(|r| r.method == hosting::Method::Patch));
    assert_eq!(
        reqs[0].url,
        "https://api.github.com/repos/octo/hello/pulls/5"
    );
    assert_eq!(
        String::from_utf8(reqs[0].body.clone().unwrap()).unwrap(),
        r#"{"state":"closed"}"#
    );
    assert_eq!(
        String::from_utf8(reqs[1].body.clone().unwrap()).unwrap(),
        r#"{"state":"open"}"#
    );
    let err = f
        .core
        .execute(
            dir.path(),
            &ActionInput::PrClose(PrNumberInput { number: 0 }),
            None,
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);
    assert_eq!(f.fake.requests().len(), 3);
}
