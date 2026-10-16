//! github.com 구현. 주소는 설정의 상수만 쓰고 서버가 준 URL을 따라가지 않는다.

use std::sync::Arc;

use secrecy::{ExposeSecret, SecretString};

use crate::HostingProvider;
use crate::account::{self, ACCOUNT_EMAILS_MAX, AccountEmail};
use crate::api::{self, API_ACCEPT, API_VERSION, API_VERSION_HEADER, USER_AGENT, encode};
use crate::config::GitHubConfig;
use crate::error::{HostingError, HostingErrorKind};
use crate::http::{HttpRequest, HttpResponse, HttpTransport, Method};
use crate::login::{self, HostUser, LoginChallenge, LoginPoll, LoginScopes};
use crate::pr::{
    self, CHECK_RUNS_PAGE, NewPr, PR_PAGE_MAX, PrDetail, PrPage, PrQuery, PullRequest,
};
use crate::pr_write::{self, Comment, MergeRequest, MergeResult};
use crate::remote::{RemoteRepo, parse_remote, valid_name, valid_owner};
use crate::repo::{self, CreatedRepo, NewRepo, REPO_PAGE_MAX, RepoInfo, RepoOwner, RepoPage};

const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
/// 같은 API 주소 안의 리디렉트를 따르는 최대 횟수.
const REDIRECT_MAX: usize = 3;

/// 검사한 저장소의 API 경로(`/repos/{o}/{r}`). 소유자·이름 조각은 퍼센트 인코딩한다.
fn repo_path(repo: &RemoteRepo) -> Result<String, HostingError> {
    if !valid_owner(&repo.owner) || !valid_name(&repo.name) {
        return Err(invalid_input("repository"));
    }
    Ok(format!(
        "/repos/{}/{}",
        encode(&repo.owner),
        encode(&repo.name)
    ))
}

fn invalid_input(detail: &str) -> HostingError {
    HostingError::with_detail(
        HostingErrorKind::Validation(crate::error::ValidationKind::Invalid),
        detail,
    )
}

pub struct GitHub {
    transport: Arc<dyn HttpTransport>,
    config: GitHubConfig,
}

impl GitHub {
    pub fn new(transport: Arc<dyn HttpTransport>, config: GitHubConfig) -> Self {
        Self { transport, config }
    }

    pub fn config(&self) -> &GitHubConfig {
        &self.config
    }

    /// github.com 저장소 주소면 저장소. 호스트가 정확히 `github.com`일 때만.
    pub fn detect(url: &str) -> Option<RemoteRepo> {
        parse_remote(url).filter(RemoteRepo::is_github)
    }

    fn client_id(&self) -> Result<&str, HostingError> {
        self.config
            .client_id
            .as_deref()
            .ok_or_else(|| HostingError::new(HostingErrorKind::NotConfigured))
    }

    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, HostingError> {
        self.transport.send(req).map_err(api::transport_error)
    }

    /// 로그인 주소(`web_base`)에 폼을 보낸다. 오류도 JSON(`{"error": …}`)으로 오므로 본문을 돌려준다.
    fn login_post(&self, path: &str, fields: &[(&str, &str)]) -> Result<Vec<u8>, HostingError> {
        let req = HttpRequest::new(Method::Post, format!("{}{path}", self.config.web_base))
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(api::form_body(fields));
        let res = self.send(&req)?;
        match res.status {
            200..=299 | 400 | 401 => Ok(res.body),
            // 없는 client id
            404 => Err(HostingError::with_detail(
                HostingErrorKind::NotConfigured,
                "HTTP 404",
            )),
            _ => Err(api::status_error(&res)),
        }
    }

    /// API 요청을 보낸다. 같은 API 주소(https, 같은 호스트·포트) 아래로 가는 리디렉트만
    /// [`REDIRECT_MAX`]번까지 같은 요청으로 따른다. 다른 곳이면 보내지 않고 `Moved`.
    fn api_send(&self, mut req: HttpRequest) -> Result<HttpResponse, HostingError> {
        for _ in 0..=REDIRECT_MAX {
            let res = self.send(&req)?;
            if !matches!(res.status, 301 | 302 | 307 | 308) {
                return Ok(res);
            }
            let Some(next) = res.header("location").and_then(|l| self.same_api_url(l)) else {
                return Err(HostingError::with_detail(
                    HostingErrorKind::Moved,
                    &format!("HTTP {}", res.status),
                ));
            };
            req.url = next;
        }
        Err(HostingError::with_detail(
            HostingErrorKind::Moved,
            "too many redirects",
        ))
    }

    /// 리디렉트 주소가 `api_base` 바로 아래면 절대 주소. 사용자 정보·포트·다른 호스트는 `None`.
    fn same_api_url(&self, location: &str) -> Option<String> {
        let base = self.config.api_base.as_str();
        let url = if location.starts_with('/') && !location.starts_with("//") {
            format!("{base}{location}")
        } else {
            location.to_owned()
        };
        let rest = url.strip_prefix(base)?;
        let clean = rest.starts_with('/')
            && !rest
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '\\' | '@' | '#'));
        clean.then_some(url)
    }

    fn api_request(&self, method: Method, token: &SecretString, path: &str) -> HttpRequest {
        HttpRequest::new(method, format!("{}{path}", self.config.api_base))
            .header("User-Agent", USER_AGENT)
            .header("Accept", API_ACCEPT)
            .header(API_VERSION_HEADER, API_VERSION)
            .bearer(token.clone())
    }

    /// API 주소에 JSON 본문(없으면 빈 본문)을 실어 쓰기 요청(POST·PUT·PATCH, D-176). 2xx만 응답으로 준다.
    fn api_write(
        &self,
        method: Method,
        token: &SecretString,
        path: &str,
        json: Option<Vec<u8>>,
    ) -> Result<HttpResponse, HostingError> {
        let mut req = self.api_request(method, token, path);
        if let Some(body) = json {
            req = req.header("Content-Type", "application/json").body(body);
        }
        let res = self.api_send(req)?;
        api::check_status(&res)?;
        Ok(res)
    }

    /// API 주소(`api_base`)에 토큰을 실어 GET. 2xx만 응답으로 준다.
    fn api_get(&self, token: &SecretString, path: &str) -> Result<HttpResponse, HostingError> {
        let res = self.api_send(self.api_request(Method::Get, token, path))?;
        api::check_status(&res)?;
        Ok(res)
    }
}

impl HostingProvider for GitHub {
    fn id(&self) -> &'static str {
        "github"
    }

    fn detect(&self, url: &str) -> Option<RemoteRepo> {
        Self::detect(url)
    }

    fn begin_login(&self, scopes: LoginScopes) -> Result<LoginChallenge, HostingError> {
        let client_id = self.client_id()?;
        let body = self.login_post(
            "/login/device/code",
            &[("client_id", client_id), ("scope", scopes.form_value())],
        )?;
        let expected_uri = format!("{}/login/device", self.config.web_base);
        login::parse_device_code(&body, &expected_uri)
    }

    fn poll_login(&self, challenge: &LoginChallenge) -> Result<LoginPoll, HostingError> {
        let client_id = self.client_id()?;
        let body = self.login_post(
            "/login/oauth/access_token",
            &[
                ("client_id", client_id),
                ("device_code", challenge.device_code.expose_secret()),
                ("grant_type", DEVICE_GRANT),
            ],
        )?;
        login::parse_token(&body, challenge.interval_secs)
    }

    fn current_user(&self, token: &SecretString) -> Result<HostUser, HostingError> {
        let res = self.api_get(token, "/user")?;
        login::parse_user(&res.body)
    }

    fn list_prs(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        query: &PrQuery,
    ) -> Result<PrPage, HostingError> {
        let per_page = query.per_page.clamp(1, PR_PAGE_MAX);
        let page = query.page.max(1);
        let mut path = format!(
            "{}/pulls?state={}&per_page={per_page}&page={page}&sort=updated&direction=desc",
            repo_path(repo)?,
            query.state.as_str()
        );
        if let Some(branch) = &query.head_branch {
            if !pr::valid_branch(branch) {
                return Err(invalid_input("branch name"));
            }
            path.push_str("&head=");
            path.push_str(&encode(&format!("{}:{branch}", repo.owner)));
        }
        let res = self.api_get(token, &path)?;
        Ok(PrPage {
            items: pr::parse_list(&res.body, &repo.owner)?,
            has_more: pr::has_next_link(res.header("link")),
        })
    }

    fn pr_detail(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        number: u64,
    ) -> Result<PrDetail, HostingError> {
        let base = repo_path(repo)?;
        let res = self.api_get(token, &format!("{base}/pulls/{number}"))?;
        let mut detail = pr::parse_detail(&res.body, &repo.owner)?;
        // head sha는 parse_detail이 16진수 40·64자인지 확인했다.
        let checks_path = format!(
            "{base}/commits/{}/check-runs?per_page={CHECK_RUNS_PAGE}",
            detail.head_sha
        );
        detail.checks = match self.api_get(token, &checks_path) {
            Ok(res) => pr::parse_check_runs(&res.body).ok(),
            // 시험 결과를 볼 권한이 없거나 없으면 상세는 그대로 보인다.
            Err(err)
                if matches!(
                    err.kind(),
                    HostingErrorKind::Forbidden { .. } | HostingErrorKind::NotFound
                ) =>
            {
                None
            }
            Err(err) => return Err(err),
        };
        // 리뷰 요약(D-134): 볼 수 없으면 줄을 빼고, 100개를 넘으면(다음 쪽이 있으면) 빼고, 요청 한도는 알린다.
        let reviews_path = format!(
            "{base}/pulls/{number}/reviews?per_page={}",
            pr::REVIEWS_PAGE
        );
        detail.reviews = match self.api_get(token, &reviews_path) {
            Ok(res) if pr::has_next_link(res.header("link")) => None,
            Ok(res) => pr::parse_reviews(&res.body).ok(),
            Err(err)
                if matches!(
                    err.kind(),
                    HostingErrorKind::Forbidden {
                        rate_limited: false,
                        ..
                    } | HostingErrorKind::NotFound
                ) =>
            {
                None
            }
            Err(err) => return Err(err),
        };
        Ok(detail)
    }

    fn create_pr(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        new: &NewPr,
    ) -> Result<PullRequest, HostingError> {
        pr::check_new_pr(new)?;
        let body = pr::new_pr_json(new)?;
        let path = format!("{}/pulls", repo_path(repo)?);
        let res = self.api_write(Method::Post, token, &path, Some(body))?;
        pr::parse_one(&res.body, &repo.owner)
    }

    fn create_repo(
        &self,
        token: &SecretString,
        new: &NewRepo,
    ) -> Result<CreatedRepo, HostingError> {
        repo::check_new_repo(new)?;
        let body = repo::new_repo_json(new)?;
        let path = match &new.owner {
            RepoOwner::Me => "/user/repos".to_owned(),
            RepoOwner::Org(org) => format!("/orgs/{}/repos", encode(org)),
        };
        let res = self.api_write(Method::Post, token, &path, Some(body))?;
        let created = repo::parse_created(&res.body)?;
        if let RepoOwner::Org(org) = &new.owner
            && !created.owner.eq_ignore_ascii_case(org)
        {
            return Err(HostingError::with_detail(
                HostingErrorKind::BadResponse,
                "unexpected repository owner",
            ));
        }
        Ok(created)
    }

    fn account_emails(&self, token: &SecretString) -> Result<Vec<AccountEmail>, HostingError> {
        let res = self.api_get(
            token,
            &format!("/user/emails?per_page={ACCOUNT_EMAILS_MAX}"),
        )?;
        account::parse_emails(&res.body)
    }

    fn list_comments(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        number: u64,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<Comment>, HostingError> {
        pr_write::check_number(number)?;
        let per_page = per_page.clamp(1, PR_PAGE_MAX);
        let page = page.max(1);
        let path = format!(
            "{}/issues/{number}/comments?per_page={per_page}&page={page}",
            repo_path(repo)?
        );
        let res = self.api_get(token, &path)?;
        pr_write::parse_comments(&res.body)
    }

    fn create_comment(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        number: u64,
        body: &str,
    ) -> Result<u64, HostingError> {
        pr_write::check_number(number)?;
        pr_write::check_comment(body)?;
        let path = format!("{}/issues/{number}/comments", repo_path(repo)?);
        let res = self.api_write(
            Method::Post,
            token,
            &path,
            Some(pr_write::comment_json(body)?),
        )?;
        pr_write::parse_created_id(&res.body)
    }

    fn merge_pr(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        number: u64,
        request: &MergeRequest,
    ) -> Result<MergeResult, HostingError> {
        pr_write::check_number(number)?;
        let json = pr_write::merge_json(request)?;
        let path = format!("{}/pulls/{number}/merge", repo_path(repo)?);
        let res = self.api_write(Method::Put, token, &path, Some(json))?;
        pr_write::parse_merged(&res.body)
    }

    fn set_pr_state(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        number: u64,
        open: bool,
    ) -> Result<(), HostingError> {
        pr_write::check_number(number)?;
        let path = format!("{}/pulls/{number}", repo_path(repo)?);
        self.api_write(
            Method::Patch,
            token,
            &path,
            Some(pr_write::state_json(open)),
        )?;
        Ok(())
    }

    fn create_fork(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
    ) -> Result<RemoteRepo, HostingError> {
        let path = format!("{}/forks", repo_path(repo)?);
        let res = self.api_write(Method::Post, token, &path, None)?;
        pr_write::parse_fork(&res.body)
    }

    fn repo_exists(&self, token: &SecretString, repo: &RemoteRepo) -> Result<bool, HostingError> {
        match self.api_get(token, &repo_path(repo)?) {
            Ok(_) => Ok(true),
            Err(err) if err.kind() == HostingErrorKind::NotFound => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn repo_info(&self, token: &SecretString, repo: &RemoteRepo) -> Result<RepoInfo, HostingError> {
        let res = self.api_get(token, &repo_path(repo)?)?;
        repo::parse_info(&res.body)
    }

    fn branch_protected(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        branch: &str,
    ) -> Result<Option<bool>, HostingError> {
        if !pr::valid_branch(branch) {
            return Err(invalid_input("branch name"));
        }
        let path = format!("{}/branches/{}", repo_path(repo)?, encode(branch));
        match self.api_get(token, &path) {
            Ok(res) => Ok(repo::parse_protected(&res.body)),
            // 볼 수 없거나 없으면 모른다(안내만 빠진다).
            Err(err)
                if matches!(
                    err.kind(),
                    HostingErrorKind::Forbidden {
                        rate_limited: false,
                        sso: false,
                    } | HostingErrorKind::NotFound
                ) =>
            {
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    fn list_repos(
        &self,
        token: &SecretString,
        page: u32,
        per_page: u32,
    ) -> Result<RepoPage, HostingError> {
        let per_page = per_page.clamp(1, REPO_PAGE_MAX);
        let page = page.max(1);
        let path = format!(
            "/user/repos?affiliation=owner,collaborator,organization_member&sort=pushed&direction=desc&per_page={per_page}&page={page}"
        );
        let res = self.api_get(token, &path)?;
        Ok(RepoPage {
            items: repo::parse_list(&res.body)?,
            has_more: pr::has_next_link(res.header("link")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pr::{Mergeable, PrStateFilter};
    use crate::testing::FakeTransport;

    const CLIENT_ID: &str = "Ov23liTESTCLIENT01";
    const TOKEN: &str = "gho_TESTSECRET0123456789";
    const DEVICE_CODE: &str = "3584d83530557fdd1f46af8289938c8ef79f9dc5";

    fn github(fake: &Arc<FakeTransport>) -> GitHub {
        GitHub::new(fake.clone(), GitHubConfig::new(Some(CLIENT_ID)))
    }

    fn device_json() -> String {
        format!(
            r#"{{"device_code":"{DEVICE_CODE}","user_code":"WDJB-MJHT",
                "verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}}"#
        )
    }

    fn header<'a>(req: &'a HttpRequest, name: &str) -> Option<&'a str> {
        req.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    fn body(req: &HttpRequest) -> String {
        String::from_utf8(req.body.clone().unwrap_or_default()).unwrap()
    }

    #[test]
    fn full_login_flow() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, &device_json());
        fake.push_json(200, r#"{"error":"authorization_pending"}"#);
        fake.push_json(200, r#"{"error":"authorization_pending"}"#);
        fake.push_json(200, r#"{"error":"slow_down","interval":10}"#);
        fake.push_json(
            200,
            &format!(r#"{{"access_token":"{TOKEN}","token_type":"bearer","scope":"repo"}}"#),
        );
        fake.push_json(200, r#"{"login":"octocat","id":1,"name":"Mona"}"#);
        let gh = github(&fake);

        let challenge = gh.begin_login(LoginScopes::default()).unwrap();
        assert_eq!(challenge.user_code, "WDJB-MJHT");
        assert!(matches!(gh.poll_login(&challenge), Ok(LoginPoll::Pending)));
        assert!(matches!(gh.poll_login(&challenge), Ok(LoginPoll::Pending)));
        assert!(matches!(
            gh.poll_login(&challenge),
            Ok(LoginPoll::SlowDown { interval_secs: 10 })
        ));
        let LoginPoll::Granted { token, scopes } = gh.poll_login(&challenge).unwrap() else {
            panic!("토큰을 받아야 한다");
        };
        assert_eq!(scopes, LoginScopes::default());
        let user = gh.current_user(&token).unwrap();
        assert_eq!(user.login, "octocat");
        assert_eq!(fake.pending(), 0);

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 6);
        assert_eq!(reqs[0].method, Method::Post);
        assert_eq!(reqs[0].url, "https://github.com/login/device/code");
        assert_eq!(body(&reqs[0]), format!("client_id={CLIENT_ID}&scope=repo"));
        assert_eq!(reqs[1].url, "https://github.com/login/oauth/access_token");
        assert_eq!(
            body(&reqs[1]),
            format!(
                "client_id={CLIENT_ID}&device_code={DEVICE_CODE}&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code"
            )
        );
        assert_eq!(reqs[5].method, Method::Get);
        assert_eq!(reqs[5].url, "https://api.github.com/user");
        assert_eq!(
            reqs[5]
                .bearer
                .as_ref()
                .map(|t| t.expose_secret().to_owned()),
            Some(TOKEN.to_owned())
        );
        for req in &reqs {
            assert_eq!(header(req, "User-Agent"), Some(USER_AGENT));
            assert!(
                header(req, "Authorization").is_none(),
                "전송만 Authorization을 만든다"
            );
            // 토큰은 bearer 칸에만 있다.
            let visible = format!("{} {:?} {}", req.url, req.headers, body(req));
            assert!(!visible.contains(TOKEN), "{visible}");
        }
        // device_code는 폴링 본문에만 있다.
        assert!(!body(&reqs[0]).contains(DEVICE_CODE));
        assert!(!reqs[5].url.contains(DEVICE_CODE));
        assert_eq!(
            header(&reqs[5], "Accept"),
            Some("application/vnd.github+json")
        );
        assert_eq!(header(&reqs[5], "X-GitHub-Api-Version"), Some("2022-11-28"));
        assert!(reqs[..5].iter().all(|r| r.bearer.is_none()));
    }

    #[test]
    fn login_end_states() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, &device_json());
        fake.push_json(200, r#"{"error":"expired_token"}"#);
        fake.push_json(200, r#"{"error":"access_denied"}"#);
        fake.push_json(200, r#"{"error":"device_flow_disabled"}"#);
        let gh = github(&fake);
        let challenge = gh.begin_login(LoginScopes::default()).unwrap();
        assert!(matches!(gh.poll_login(&challenge), Ok(LoginPoll::Expired)));
        assert!(matches!(gh.poll_login(&challenge), Ok(LoginPoll::Denied)));
        assert_eq!(
            gh.poll_login(&challenge).unwrap_err().kind(),
            HostingErrorKind::NotConfigured
        );
    }

    #[test]
    fn begin_login_failures() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(400, r#"{"error":"device_flow_disabled"}"#);
        fake.push_json(404, r#"{"error":"Not Found"}"#);
        fake.push_json(
            200,
            r#"{"device_code":"d","user_code":"WDJB-MJHT","verification_uri":"https://evil.example/login/device"}"#,
        );
        fake.push(HttpResponse {
            status: 200,
            headers: vec![("Content-Type".into(), "text/html".into())],
            body: b"<html>proxy sign-in</html>".to_vec(),
        });
        fake.push_json(200, "");
        fake.push_json(502, "bad gateway");
        fake.push_error(crate::TransportError::Timeout);
        let gh = github(&fake);
        let kinds: Vec<_> = (0..7)
            .map(|_| gh.begin_login(LoginScopes::default()).unwrap_err().kind())
            .collect();
        assert_eq!(
            kinds,
            [
                HostingErrorKind::NotConfigured,
                HostingErrorKind::NotConfigured,
                HostingErrorKind::BadResponse,
                HostingErrorKind::BadResponse,
                HostingErrorKind::BadResponse,
                HostingErrorKind::Unavailable,
                HostingErrorKind::Unreachable,
            ]
        );
    }

    #[test]
    fn not_configured_sends_nothing() {
        let fake = Arc::new(FakeTransport::new());
        let gh = GitHub::new(fake.clone(), GitHubConfig::new(None));
        assert_eq!(
            gh.begin_login(LoginScopes::default()).unwrap_err().kind(),
            HostingErrorKind::NotConfigured
        );
        assert!(fake.requests().is_empty());
    }

    #[test]
    fn current_user_unauthorized() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(401, r#"{"message":"Bad credentials"}"#);
        let gh = github(&fake);
        let err = gh
            .current_user(&SecretString::from(TOKEN.to_owned()))
            .unwrap_err();
        assert_eq!(err.kind(), HostingErrorKind::Unauthorized);
        assert!(!format!("{err} {err:?} {:?}", err.detail()).contains(TOKEN));
    }

    #[test]
    fn detect_is_github_only() {
        assert!(GitHub::detect("git@github.com:octo/hello.git").is_some());
        assert!(GitHub::detect("https://gitlab.com/octo/hello").is_none());
        assert!(GitHub::detect("https://evil.example/github.com/hello").is_none());
    }

    fn octo() -> RemoteRepo {
        GitHub::detect("https://github.com/octo/hello").unwrap()
    }

    fn token() -> SecretString {
        SecretString::from(TOKEN.to_owned())
    }

    fn pull_json(number: u64) -> String {
        format!(
            r#"{{"number":{number},"title":"Add login","state":"open","draft":false,"merged_at":null,
               "head":{{"ref":"feature/login","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
               "base":{{"ref":"main","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
               "user":{{"login":"mona"}},"created_at":"2026-10-01T09:30:00Z",
               "updated_at":"2026-10-02T00:00:00Z","mergeable_state":"dirty"}}"#,
            sha = "a".repeat(40)
        )
    }

    fn response(status: u16, headers: &[(&str, &str)], body: &str) -> HttpResponse {
        HttpResponse {
            status,
            headers: headers
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            body: body.as_bytes().to_vec(),
        }
    }

    fn query(head: Option<&str>) -> PrQuery {
        PrQuery {
            state: PrStateFilter::Open,
            head_branch: head.map(str::to_owned),
            page: 2,
            per_page: 500,
        }
    }

    #[test]
    fn list_builds_url_and_reads_next_link() {
        let fake = Arc::new(FakeTransport::new());
        fake.push(response(
            200,
            &[(
                "Link",
                r#"<https://api.github.com/repositories/1/pulls?page=3>; rel="next""#,
            )],
            &format!("[{}]", pull_json(7)),
        ));
        let gh = github(&fake);
        let page = gh
            .list_prs(&token(), &octo(), &query(Some("feature/a&b=c")))
            .unwrap();
        assert!(page.has_more);
        assert_eq!(page.items[0].number, 7);
        let reqs = fake.requests();
        assert_eq!(
            reqs[0].url,
            "https://api.github.com/repos/octo/hello/pulls?state=open&per_page=50&page=2&sort=updated&direction=desc&head=octo%3Afeature%2Fa%26b%3Dc"
        );
        assert!(reqs[0].bearer.is_some());
    }

    #[test]
    fn list_rejects_odd_branch_without_sending() {
        let fake = Arc::new(FakeTransport::new());
        let gh = github(&fake);
        let err = gh
            .list_prs(&token(), &octo(), &query(Some("has space")))
            .unwrap_err();
        assert_eq!(
            err.kind(),
            HostingErrorKind::Validation(crate::ValidationKind::Invalid)
        );
        let bad_repo = RemoteRepo {
            host: "github.com".into(),
            owner: "../x".into(),
            name: "hello".into(),
        };
        assert!(gh.list_prs(&token(), &bad_repo, &query(None)).is_err());
        assert!(fake.requests().is_empty());
    }

    #[test]
    fn error_table() {
        use crate::ValidationKind as V;
        let forbidden = |rate_limited, sso| HostingErrorKind::Forbidden { rate_limited, sso };
        let cases = [
            (
                response(401, &[], r#"{"message":"Bad credentials"}"#),
                HostingErrorKind::Unauthorized,
            ),
            (response(403, &[], "{}"), forbidden(false, false)),
            (
                response(403, &[("X-RateLimit-Remaining", "0")], "{}"),
                forbidden(true, false),
            ),
            (
                response(403, &[("X-GitHub-SSO", "required; url=x")], "{}"),
                forbidden(false, true),
            ),
            (response(404, &[], "{}"), HostingErrorKind::NotFound),
            (
                response(
                    422,
                    &[],
                    r#"{"errors":[{"message":"A pull request already exists for octo:x."}]}"#,
                ),
                HostingErrorKind::Validation(V::AlreadyExists),
            ),
            (
                response(
                    422,
                    &[],
                    r#"{"errors":[{"message":"No commits between main and x"}]}"#,
                ),
                HostingErrorKind::Validation(V::NoDifference),
            ),
            (
                response(422, &[], r#"{"errors":[{"code":"custom"}]}"#),
                HostingErrorKind::Validation(V::Invalid),
            ),
            (response(405, &[], "{}"), HostingErrorKind::NotAllowed),
            (response(409, &[], "{}"), HostingErrorKind::Conflict),
            (response(500, &[], "oops"), HostingErrorKind::Unavailable),
        ];
        for (res, want) in cases {
            let fake = Arc::new(FakeTransport::new());
            fake.push(res);
            let err = github(&fake)
                .list_prs(&token(), &octo(), &query(None))
                .unwrap_err();
            assert_eq!(err.kind(), want);
            assert!(!format!("{err} {err:?} {:?}", err.detail()).contains(TOKEN));
        }
        let fake = Arc::new(FakeTransport::new());
        fake.push_error(crate::TransportError::Timeout);
        assert_eq!(
            github(&fake)
                .list_prs(&token(), &octo(), &query(None))
                .unwrap_err()
                .kind(),
            HostingErrorKind::Unreachable
        );
    }

    #[test]
    fn follows_redirect_only_within_api_host() {
        // 같은 호스트: 따라간다.
        let fake = Arc::new(FakeTransport::new());
        fake.push(response(
            301,
            &[(
                "Location",
                "https://api.github.com/repositories/42/pulls?state=open",
            )],
            "",
        ));
        fake.push(response(307, &[("Location", "/repositories/43/pulls")], ""));
        fake.push(response(200, &[], "[]"));
        github(&fake)
            .list_prs(&token(), &octo(), &query(None))
            .unwrap();
        let reqs = fake.requests();
        assert_eq!(reqs.len(), 3);
        assert_eq!(
            reqs[1].url,
            "https://api.github.com/repositories/42/pulls?state=open"
        );
        assert_eq!(reqs[2].url, "https://api.github.com/repositories/43/pulls");

        // 다른 호스트·http·포트·사용자 정보: 두 번째 요청이 없다.
        for location in [
            "https://evil.example/repos/octo/hello/pulls",
            "http://api.github.com/repos/octo/hello/pulls",
            "https://api.github.com.evil.example/x",
            "https://api.github.com:8443/x",
            "https://api.github.com@evil.example/x",
            "//evil.example/x",
            "",
        ] {
            let fake = Arc::new(FakeTransport::new());
            fake.push(response(301, &[("Location", location)], ""));
            fake.push(response(200, &[], "[]"));
            let err = github(&fake)
                .list_prs(&token(), &octo(), &query(None))
                .unwrap_err();
            assert_eq!(err.kind(), HostingErrorKind::Moved, "{location}");
            assert_eq!(fake.requests().len(), 1, "{location}");
        }

        // 끝없는 리디렉트는 네 번 보내고 멈춘다.
        let fake = Arc::new(FakeTransport::new());
        for _ in 0..10 {
            fake.push(response(
                302,
                &[("Location", "https://api.github.com/loop")],
                "",
            ));
        }
        let err = github(&fake)
            .list_prs(&token(), &octo(), &query(None))
            .unwrap_err();
        assert_eq!(err.kind(), HostingErrorKind::Moved);
        assert_eq!(fake.requests().len(), 4);
    }

    #[test]
    fn detail_with_and_without_checks() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, &pull_json(7));
        fake.push_json(
            200,
            r#"{"total_count":2,"check_runs":[{"conclusion":"success"},{"conclusion":null}]}"#,
        );
        fake.push_json(200, r#"[{"user":{"login":"a"},"state":"APPROVED"},{"user":{"login":"b"},"state":"CHANGES_REQUESTED"}]"#);
        let gh = github(&fake);
        let detail = gh.pr_detail(&token(), &octo(), 7).unwrap();
        assert_eq!(detail.mergeable, Mergeable::Conflicts);
        assert_eq!(detail.checks.unwrap().pending, 1);
        assert_eq!(
            detail.reviews,
            Some(crate::pr::ReviewsSummary {
                approved: 1,
                changes_requested: 1
            })
        );
        let reqs = fake.requests();
        assert_eq!(
            reqs[0].url,
            "https://api.github.com/repos/octo/hello/pulls/7"
        );
        assert_eq!(
            reqs[1].url,
            format!(
                "https://api.github.com/repos/octo/hello/commits/{}/check-runs?per_page=100",
                "a".repeat(40)
            )
        );

        assert_eq!(
            fake.requests()[2].url,
            "https://api.github.com/repos/octo/hello/pulls/7/reviews?per_page=100"
        );

        for status in [403, 404] {
            let fake = Arc::new(FakeTransport::new());
            fake.push_json(200, &pull_json(7));
            fake.push_json(status, "{}");
            fake.push_json(status, "{}");
            let detail = github(&fake).pr_detail(&token(), &octo(), 7).unwrap();
            assert_eq!(detail.checks, None);
            assert_eq!(detail.reviews, None);
        }

        // 리뷰가 100개를 넘으면(다음 쪽이 있으면) 요약을 빼고 상세는 그대로다.
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, &pull_json(7));
        fake.push_json(200, r#"{"total_count":0,"check_runs":[]}"#);
        fake.push(HttpResponse {
            status: 200,
            headers: vec![
                ("Content-Type".into(), "application/json".into()),
                (
                    "Link".into(),
                    r#"<https://api.github.com/x?page=2>; rel="next""#.into(),
                ),
            ],
            body: b"[]".to_vec(),
        });
        assert_eq!(
            github(&fake)
                .pr_detail(&token(), &octo(), 7)
                .unwrap()
                .reviews,
            None
        );

        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, &pull_json(7));
        fake.push_json(401, "{}");
        assert_eq!(
            github(&fake)
                .pr_detail(&token(), &octo(), 7)
                .unwrap_err()
                .kind(),
            HostingErrorKind::Unauthorized
        );
    }

    #[test]
    fn repositories_are_created_and_listed() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(
            201,
            r#"{"name":"hello","owner":{"login":"octocat"},"private":true}"#,
        );
        fake.push_json(422, r#"{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"custom","field":"name","message":"name already exists on this account"}]}"#);
        fake.push_json(
            422,
            r#"{"message":"Repository creation failed.","errors":[{"message":"something else"}]}"#,
        );
        fake.push(HttpResponse {
            status: 200,
            headers: vec![
                ("Content-Type".into(), "application/json".into()),
                (
                    "Link".into(),
                    r#"<https://api.github.com/user/repos?page=2>; rel="next""#.into(),
                ),
            ],
            body: br#"[{"name":"a","owner":{"login":"me"},"private":false,"pushed_at":null}]"#
                .to_vec(),
        });
        let gh = github(&fake);
        let new = NewRepo {
            name: "hello".into(),
            description: "설명".into(),
            private: true,
            owner: RepoOwner::Me,
        };
        assert_eq!(
            gh.create_repo(&token(), &new).unwrap(),
            CreatedRepo {
                owner: "octocat".into(),
                name: "hello".into()
            }
        );
        assert_eq!(
            gh.create_repo(&token(), &new).unwrap_err().kind(),
            HostingErrorKind::Validation(crate::error::ValidationKind::AlreadyExists)
        );
        assert_eq!(
            gh.create_repo(&token(), &new).unwrap_err().kind(),
            HostingErrorKind::Validation(crate::error::ValidationKind::Invalid)
        );
        let page = gh.list_repos(&token(), 2, 500).unwrap();
        assert!(page.has_more && page.items.len() == 1);
        let reqs = fake.requests();
        assert_eq!(reqs[0].method, Method::Post);
        assert_eq!(reqs[0].url, "https://api.github.com/user/repos");
        assert_eq!(
            reqs[3].url,
            "https://api.github.com/user/repos?affiliation=owner,collaborator,organization_member&sort=pushed&direction=desc&per_page=50&page=2"
        );
        for req in &reqs {
            assert!(!format!("{} {:?} {}", req.url, req.headers, body(req)).contains(TOKEN));
        }
        // 규칙에 맞지 않는 입력은 보내지 않는다.
        let fake = Arc::new(FakeTransport::new());
        let bad = NewRepo {
            name: "../x".into(),
            ..new
        };
        assert!(github(&fake).create_repo(&token(), &bad).is_err());
        assert!(fake.requests().is_empty());
    }

    #[test]
    fn repository_info_and_branch_protection() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(
            200,
            r#"{"name":"hello","owner":{"login":"octo"},"permissions":{"push":true},"allow_squash_merge":true}"#,
        );
        fake.push_json(200, r#"{"name":"main","protected":true}"#);
        fake.push_json(403, "{}");
        fake.push_json(404, "{}");
        fake.push_json(401, "{}");
        let gh = github(&fake);
        let info = gh.repo_info(&token(), &octo()).unwrap();
        assert!(info.can_push && info.allow_squash_merge == Some(true));
        assert_eq!(
            gh.branch_protected(&token(), &octo(), "main").unwrap(),
            Some(true)
        );
        assert_eq!(
            gh.branch_protected(&token(), &octo(), "feature/x").unwrap(),
            None
        );
        assert_eq!(
            gh.branch_protected(&token(), &octo(), "기능").unwrap(),
            None
        );
        assert_eq!(
            gh.branch_protected(&token(), &octo(), "main")
                .unwrap_err()
                .kind(),
            HostingErrorKind::Unauthorized
        );
        // 이상한 브랜치 이름은 보내지 않는다.
        assert!(gh.branch_protected(&token(), &octo(), "a b").is_err());
        let reqs = fake.requests();
        assert_eq!(reqs.len(), 5);
        assert!(reqs.iter().all(|r| r.method == Method::Get));
        assert_eq!(reqs[0].url, "https://api.github.com/repos/octo/hello");
        assert_eq!(
            reqs[1].url,
            "https://api.github.com/repos/octo/hello/branches/main"
        );
        assert_eq!(
            reqs[2].url,
            "https://api.github.com/repos/octo/hello/branches/feature%2Fx"
        );
        assert_eq!(
            reqs[3].url,
            "https://api.github.com/repos/octo/hello/branches/%EA%B8%B0%EB%8A%A5"
        );
        assert!(reqs.iter().all(|r| !r.url.contains(TOKEN)));
    }

    #[test]
    fn pull_request_writes_and_fork() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(
            200,
            r#"[{"id":1,"user":{"login":"mona"},"created_at":"2026-10-01T00:00:00Z","body":"hi"}]"#,
        );
        fake.push_json(201, r#"{"id":77,"body":"고마워요"}"#);
        fake.push_json(
            200,
            r#"{"sha":"x","merged":true,"message":"Pull Request successfully merged"}"#,
        );
        fake.push_json(405, r#"{"message":"Pull Request is not mergeable"}"#);
        fake.push_json(
            409,
            r#"{"message":"Head branch was modified. Review and try the merge again."}"#,
        );
        fake.push_json(200, r#"{"number":3,"state":"closed"}"#);
        fake.push_json(
            422,
            r#"{"message":"Validation Failed","errors":[{"message":"state cannot be changed"}]}"#,
        );
        fake.push_json(
            202,
            r#"{"name":"hello","owner":{"login":"mona"},"clone_url":"https://evil"}"#,
        );
        fake.push_json(200, "{}");
        fake.push_json(404, "{}");
        let gh = github(&fake);
        let sha = "b".repeat(40);
        let merge = MergeRequest {
            method: crate::MergeMethod::Merge,
            sha: sha.clone(),
        };
        assert_eq!(
            gh.list_comments(&token(), &octo(), 3, 2, 20).unwrap().len(),
            1
        );
        assert_eq!(
            gh.create_comment(&token(), &octo(), 3, " 고마워요 \u{1F600} ")
                .unwrap(),
            77
        );
        assert!(gh.merge_pr(&token(), &octo(), 3, &merge).unwrap().merged);
        assert_eq!(
            gh.merge_pr(&token(), &octo(), 3, &merge)
                .unwrap_err()
                .kind(),
            HostingErrorKind::NotAllowed
        );
        assert_eq!(
            gh.merge_pr(&token(), &octo(), 3, &merge)
                .unwrap_err()
                .kind(),
            HostingErrorKind::Conflict
        );
        gh.set_pr_state(&token(), &octo(), 3, false).unwrap();
        assert!(matches!(
            gh.set_pr_state(&token(), &octo(), 3, true)
                .unwrap_err()
                .kind(),
            HostingErrorKind::Validation(_)
        ));
        let fork = gh.create_fork(&token(), &octo()).unwrap();
        assert_eq!((fork.owner.as_str(), fork.name.as_str()), ("mona", "hello"));
        assert!(gh.repo_exists(&token(), &fork).unwrap());
        assert!(!gh.repo_exists(&token(), &fork).unwrap());
        // 보내기 전에 막히는 것: 빈 댓글, 번호 0, 짧은 해시.
        assert!(gh.create_comment(&token(), &octo(), 3, "  ").is_err());
        assert!(gh.set_pr_state(&token(), &octo(), 0, false).is_err());
        let short = MergeRequest {
            method: crate::MergeMethod::Squash,
            sha: "abc".into(),
        };
        assert!(gh.merge_pr(&token(), &octo(), 3, &short).is_err());

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 10);
        let base = "https://api.github.com/repos/octo/hello";
        let seen: Vec<(Method, String)> = reqs.iter().map(|r| (r.method, r.url.clone())).collect();
        assert_eq!(
            seen[0],
            (
                Method::Get,
                format!("{base}/issues/3/comments?per_page=20&page=2")
            )
        );
        assert_eq!(seen[1], (Method::Post, format!("{base}/issues/3/comments")));
        assert_eq!(seen[2], (Method::Put, format!("{base}/pulls/3/merge")));
        assert_eq!(seen[5], (Method::Patch, format!("{base}/pulls/3")));
        assert_eq!(seen[7], (Method::Post, format!("{base}/forks")));
        assert_eq!(
            seen[8],
            (
                Method::Get,
                "https://api.github.com/repos/mona/hello".to_string()
            )
        );
        assert_eq!(body(&reqs[1]), "{\"body\":\"고마워요 \u{1F600}\"}");
        let merge_body: serde_json::Value =
            serde_json::from_slice(reqs[2].body.as_ref().unwrap()).unwrap();
        assert_eq!(merge_body["merge_method"], "merge");
        assert_eq!(merge_body["sha"], sha.as_str());
        assert_eq!(body(&reqs[5]), r#"{"state":"closed"}"#);
        assert_eq!(body(&reqs[6]), r#"{"state":"open"}"#);
        assert!(reqs[7].body.is_none());
        assert!(
            reqs.iter()
                .all(|r| !r.url.contains(TOKEN) && !body(r).contains(TOKEN))
        );
    }

    #[test]
    fn organization_repository_and_emails() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(
            201,
            r#"{"name":"tool","owner":{"login":"My-Org","type":"Organization"}}"#,
        );
        fake.push_json(201, r#"{"name":"tool","owner":{"login":"someone"}}"#);
        fake.push_json(403, r#"{"message":"Must have admin rights"}"#);
        fake.push_json(200, r#"[{"email":"me@example.com","verified":true}]"#);
        let gh = github(&fake);
        let new = NewRepo {
            name: "tool".into(),
            description: String::new(),
            private: true,
            owner: RepoOwner::Org("my-org".into()),
        };
        assert_eq!(gh.create_repo(&token(), &new).unwrap().owner, "My-Org");
        assert_eq!(
            gh.create_repo(&token(), &new).unwrap_err().kind(),
            HostingErrorKind::BadResponse
        );
        assert!(matches!(
            gh.create_repo(&token(), &new).unwrap_err().kind(),
            HostingErrorKind::Forbidden { .. }
        ));
        assert_eq!(gh.account_emails(&token()).unwrap().len(), 1);
        let reqs = fake.requests();
        assert_eq!(reqs[0].url, "https://api.github.com/orgs/my-org/repos");
        assert_eq!(
            reqs[3].url,
            "https://api.github.com/user/emails?per_page=100"
        );
        // 조직 이름 규칙에 맞지 않으면 보내지 않는다.
        let fake = Arc::new(FakeTransport::new());
        for org in ["-a", "", &"a".repeat(40), "a/b"] {
            let bad = NewRepo {
                owner: RepoOwner::Org(org.to_string()),
                ..new.clone()
            };
            assert!(github(&fake).create_repo(&token(), &bad).is_err(), "{org}");
        }
        assert!(fake.requests().is_empty());
    }

    #[test]
    fn writes_send_json_once() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, "{}");
        fake.push_json(409, r#"{"message":"Head branch was modified."}"#);
        let gh = github(&fake);
        let res = gh
            .api_write(
                Method::Put,
                &token(),
                "/repos/octo/hello/pulls/1/merge",
                Some(br#"{"sha":"a"}"#.to_vec()),
            )
            .unwrap();
        assert_eq!(res.status, 200);
        let err = gh
            .api_write(
                Method::Patch,
                &token(),
                "/repos/octo/hello/pulls/1",
                Some(b"{}".to_vec()),
            )
            .unwrap_err();
        assert_eq!(err.kind(), HostingErrorKind::Conflict);
        let reqs = fake.requests();
        assert_eq!(
            (reqs[0].method, reqs[1].method),
            (Method::Put, Method::Patch)
        );
        assert_eq!(header(&reqs[0], "Content-Type"), Some("application/json"));
        assert_eq!(body(&reqs[0]), r#"{"sha":"a"}"#);
        assert!(!format!("{err} {err:?}").contains(TOKEN));
    }

    #[test]
    fn login_scope_is_sent_as_asked() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(200, &device_json());
        github(&fake)
            .begin_login(LoginScopes {
                workflow: true,
                email: false,
            })
            .unwrap();
        assert_eq!(
            body(&fake.requests()[0]),
            format!("client_id={CLIENT_ID}&scope=repo%20workflow")
        );
    }

    #[test]
    fn create_sends_json_and_reads_result() {
        let fake = Arc::new(FakeTransport::new());
        fake.push_json(201, &pull_json(12));
        let gh = github(&fake);
        let new = NewPr {
            title: "로그인 \"추가\"".into(),
            body: "본문\n둘째 줄".into(),
            head: "feature/login".into(),
            head_owner: None,
            base: "main".into(),
            draft: false,
        };
        let created = gh.create_pr(&token(), &octo(), &new).unwrap();
        assert_eq!(created.number, 12);
        let reqs = fake.requests();
        assert_eq!(reqs[0].method, Method::Post);
        assert_eq!(reqs[0].url, "https://api.github.com/repos/octo/hello/pulls");
        assert_eq!(header(&reqs[0], "Content-Type"), Some("application/json"));
        let sent: serde_json::Value =
            serde_json::from_slice(reqs[0].body.as_ref().unwrap()).unwrap();
        assert_eq!(sent["title"], "로그인 \"추가\"");
        assert_eq!(sent["body"], "본문\n둘째 줄");
        assert_eq!(sent["head"], "feature/login");
        assert_eq!(sent["draft"], false);
        assert!(!body(&reqs[0]).contains(TOKEN));

        let fake = Arc::new(FakeTransport::new());
        let gh = github(&fake);
        let empty = NewPr {
            title: " ".into(),
            ..new
        };
        assert!(gh.create_pr(&token(), &octo(), &empty).is_err());
        assert!(fake.requests().is_empty());
    }
}
