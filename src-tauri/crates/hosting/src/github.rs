//! github.com 구현. 주소는 설정의 상수만 쓰고 서버가 준 URL을 따라가지 않는다.

use std::sync::Arc;

use secrecy::{ExposeSecret, SecretString};

use crate::HostingProvider;
use crate::api::{self, API_ACCEPT, API_VERSION, API_VERSION_HEADER, USER_AGENT};
use crate::config::GitHubConfig;
use crate::error::{HostingError, HostingErrorKind};
use crate::http::{HttpRequest, HttpResponse, HttpTransport, Method};
use crate::login::{self, HostUser, LoginChallenge, LoginPoll};
use crate::remote::{RemoteRepo, parse_remote};

const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

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

    /// API 주소(`api_base`)에 토큰을 실어 GET.
    fn api_get(&self, token: &SecretString, path: &str) -> Result<HttpResponse, HostingError> {
        let req = HttpRequest::new(Method::Get, format!("{}{path}", self.config.api_base))
            .header("User-Agent", USER_AGENT)
            .header("Accept", API_ACCEPT)
            .header(API_VERSION_HEADER, API_VERSION)
            .bearer(token.clone());
        let res = self.send(&req)?;
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

    fn begin_login(&self) -> Result<LoginChallenge, HostingError> {
        let client_id = self.client_id()?;
        let body = self.login_post(
            "/login/device/code",
            &[("client_id", client_id), ("scope", &self.config.scope)],
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
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let challenge = gh.begin_login().unwrap();
        assert_eq!(challenge.user_code, "WDJB-MJHT");
        assert!(matches!(gh.poll_login(&challenge), Ok(LoginPoll::Pending)));
        assert!(matches!(gh.poll_login(&challenge), Ok(LoginPoll::Pending)));
        assert!(matches!(
            gh.poll_login(&challenge),
            Ok(LoginPoll::SlowDown { interval_secs: 10 })
        ));
        let LoginPoll::Granted(token) = gh.poll_login(&challenge).unwrap() else {
            panic!("토큰을 받아야 한다");
        };
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
        let challenge = gh.begin_login().unwrap();
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
            .map(|_| gh.begin_login().unwrap_err().kind())
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
            gh.begin_login().unwrap_err().kind(),
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
}
