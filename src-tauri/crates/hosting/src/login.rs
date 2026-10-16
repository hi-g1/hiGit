//! GitHub device flow의 응답을 읽고 검사한다(D-109).
//!
//! 서버가 준 값은 모양을 확인한 뒤에만 쓴다. `verification_uri`는 설정의 주소와 정확히 같아야 하고,
//! 화면은 이 값이 아니라 상수 주소를 연다.

use std::fmt;

use secrecy::SecretString;
use serde::Deserialize;

use crate::error::{HostingError, HostingErrorKind};

const EXPIRES_MIN: u64 = 60;
const EXPIRES_MAX: u64 = 1800;
const INTERVAL_MIN: u64 = 1;
const INTERVAL_MAX: u64 = 60;
const INTERVAL_DEFAULT: u64 = 5;
/// `slow_down`에 간격이 없을 때 더하는 초(RFC 8628).
const SLOW_DOWN_STEP: u64 = 5;
const CODE_MAX: usize = 256;
const TOKEN_MAX: usize = 255;

/// 로그인 시작으로 받은 것. 사람이 브라우저에 `user_code`를 넣는다.
#[derive(Clone)]
pub struct LoginChallenge {
    pub device_code: SecretString,
    pub user_code: String,
    pub expires_in_secs: u64,
    pub interval_secs: u64,
}

impl fmt::Debug for LoginChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginChallenge")
            .field("device_code", &"<가림>")
            .field("user_code", &self.user_code)
            .field("expires_in_secs", &self.expires_in_secs)
            .field("interval_secs", &self.interval_secs)
            .finish()
    }
}

/// device flow로 요청할 권한(D-109, D-135). 폼의 `scope` 값을 이것만 만든다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginScope {
    /// 비공개 저장소 올리기·검토 요청에 필요한 최소.
    Repo,
    /// 거기에 `.github/workflows` 바꾸기를 더한다(사용자가 고를 때만).
    RepoWorkflow,
}

impl LoginScope {
    pub fn form_value(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::RepoWorkflow => "repo workflow",
        }
    }
}

/// 한 번 물어본 결과.
#[derive(Clone)]
pub enum LoginPoll {
    /// 아직 사람이 승인하지 않았다.
    Pending,
    /// 너무 자주 물었다. 다음부터 이 간격(초)으로.
    SlowDown {
        interval_secs: u64,
    },
    Expired,
    Denied,
    /// 토큰과, 받은 권한에 `workflow`가 들었는지.
    Granted {
        token: SecretString,
        workflow: bool,
    },
}

impl fmt::Debug for LoginPoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("Pending"),
            Self::SlowDown { interval_secs } => f
                .debug_struct("SlowDown")
                .field("interval_secs", interval_secs)
                .finish(),
            Self::Expired => f.write_str("Expired"),
            Self::Denied => f.write_str("Denied"),
            Self::Granted { workflow, .. } => f
                .debug_struct("Granted")
                .field("token", &"<가림>")
                .field("workflow", workflow)
                .finish(),
        }
    }
}

/// 토큰 주인.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
}

#[derive(Deserialize)]
struct DeviceCodeBody {
    device_code: Option<String>,
    user_code: Option<String>,
    verification_uri: Option<String>,
    expires_in: Option<u64>,
    interval: Option<u64>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct TokenBody {
    access_token: Option<String>,
    token_type: Option<String>,
    /// 받은 권한(쉼표나 공백으로 나눈 목록).
    scope: Option<String>,
    error: Option<String>,
    interval: Option<u64>,
}

#[derive(Deserialize)]
struct UserBody {
    login: String,
    id: u64,
    name: Option<String>,
}

fn bad(detail: &str) -> HostingError {
    HostingError::with_detail(HostingErrorKind::BadResponse, detail)
}

/// device flow를 쓸 수 없는 앱 설정(client id 틀림, device flow 꺼짐)을 알리는 오류 이름.
fn not_configured(error: &str) -> bool {
    matches!(
        error,
        "device_flow_disabled" | "incorrect_client_credentials" | "unauthorized_client"
    )
}

/// `POST /login/device/code`의 응답 본문.
pub(crate) fn parse_device_code(
    body: &[u8],
    expected_uri: &str,
) -> Result<LoginChallenge, HostingError> {
    let parsed: DeviceCodeBody =
        serde_json::from_slice(body).map_err(|_| bad("device code response is not JSON"))?;
    if let Some(error) = parsed.error.as_deref() {
        if not_configured(error) {
            return Err(HostingError::with_detail(
                HostingErrorKind::NotConfigured,
                error,
            ));
        }
        return Err(bad(error));
    }
    let (Some(device_code), Some(user_code), Some(uri)) = (
        parsed.device_code,
        parsed.user_code,
        parsed.verification_uri,
    ) else {
        return Err(bad("device code response is missing fields"));
    };
    if uri != expected_uri {
        return Err(bad("unexpected verification uri"));
    }
    if !valid_user_code(&user_code) {
        return Err(bad("unexpected user code"));
    }
    if !valid_opaque(&device_code, CODE_MAX) {
        return Err(bad("unexpected device code"));
    }
    Ok(LoginChallenge {
        device_code: SecretString::from(device_code),
        user_code,
        expires_in_secs: parsed
            .expires_in
            .unwrap_or(EXPIRES_MAX)
            .clamp(EXPIRES_MIN, EXPIRES_MAX),
        interval_secs: clamp_interval(parsed.interval.unwrap_or(INTERVAL_DEFAULT)),
    })
}

/// `POST /login/oauth/access_token`의 응답 본문.
pub(crate) fn parse_token(body: &[u8], current_interval: u64) -> Result<LoginPoll, HostingError> {
    let parsed: TokenBody =
        serde_json::from_slice(body).map_err(|_| bad("token response is not JSON"))?;
    if let Some(error) = parsed.error.as_deref() {
        return match error {
            "authorization_pending" => Ok(LoginPoll::Pending),
            "slow_down" => Ok(LoginPoll::SlowDown {
                interval_secs: clamp_interval(
                    parsed
                        .interval
                        .unwrap_or(current_interval.saturating_add(SLOW_DOWN_STEP)),
                ),
            }),
            "expired_token" => Ok(LoginPoll::Expired),
            "access_denied" => Ok(LoginPoll::Denied),
            other if not_configured(other) => Err(HostingError::with_detail(
                HostingErrorKind::NotConfigured,
                other,
            )),
            other => Err(bad(other)),
        };
    }
    let token = parsed
        .access_token
        .ok_or_else(|| bad("token response is missing fields"))?;
    let bearer = parsed
        .token_type
        .is_some_and(|kind| kind.eq_ignore_ascii_case("bearer"));
    if !bearer || !valid_opaque(&token, TOKEN_MAX) {
        return Err(bad("unexpected token"));
    }
    // 받은 권한에서는 `workflow`가 있는지만 본다. refresh_token 등 다른 값은 읽지 않고 버린다.
    let workflow = parsed.scope.as_deref().is_some_and(|scope| {
        scope
            .split(|c: char| c == ',' || c.is_whitespace())
            .any(|s| s == "workflow")
    });
    Ok(LoginPoll::Granted {
        token: SecretString::from(token),
        workflow,
    })
}

/// `GET /user`의 응답 본문.
pub(crate) fn parse_user(body: &[u8]) -> Result<HostUser, HostingError> {
    let parsed: UserBody =
        serde_json::from_slice(body).map_err(|_| bad("user response is not JSON"))?;
    if !crate::remote::valid_owner(&parsed.login) {
        return Err(bad("unexpected login"));
    }
    let name = parsed
        .name
        .map(|name| {
            name.chars()
                .filter(|c| !c.is_control())
                .take(256)
                .collect::<String>()
        })
        .filter(|name| !name.trim().is_empty());
    Ok(HostUser {
        login: parsed.login,
        id: parsed.id,
        name,
    })
}

fn clamp_interval(secs: u64) -> u64 {
    secs.clamp(INTERVAL_MIN, INTERVAL_MAX)
}

/// `XXXX-XXXX`(대문자·숫자).
fn valid_user_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    bytes.len() == 9
        && bytes[4] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || b.is_ascii_uppercase() || b.is_ascii_digit())
}

/// 공백·제어 문자가 없는 ASCII, 1~`max`자.
fn valid_opaque(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.bytes().all(|b| b.is_ascii_graphic())
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    const URI: &str = "https://github.com/login/device";

    fn device(json: &str) -> Result<LoginChallenge, HostingError> {
        parse_device_code(json.as_bytes(), URI)
    }

    #[test]
    fn reads_device_code_and_clamps_times() {
        let got = device(
            r#"{"device_code":"3584d83530557fdd1f46af8289938c8ef79f9dc5","user_code":"WDJB-MJHT",
               "verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}"#,
        )
        .unwrap();
        assert_eq!(got.user_code, "WDJB-MJHT");
        assert_eq!(got.expires_in_secs, 900);
        assert_eq!(got.interval_secs, 5);
        assert_eq!(
            got.device_code.expose_secret(),
            "3584d83530557fdd1f46af8289938c8ef79f9dc5"
        );
        assert!(!format!("{got:?}").contains("3584d835"));

        let got = device(
            r#"{"device_code":"d","user_code":"AAAA-0000","verification_uri":"https://github.com/login/device",
               "expires_in":999999,"interval":0}"#,
        )
        .unwrap();
        assert_eq!((got.expires_in_secs, got.interval_secs), (1800, 1));
        let got = device(
            r#"{"device_code":"d","user_code":"AAAA-0000","verification_uri":"https://github.com/login/device",
               "expires_in":1,"interval":600}"#,
        )
        .unwrap();
        assert_eq!((got.expires_in_secs, got.interval_secs), (60, 60));
    }

    #[test]
    fn rejects_unexpected_device_code_responses() {
        let cases = [
            // 다른 호스트로 보내려는 주소
            r#"{"device_code":"d","user_code":"WDJB-MJHT","verification_uri":"https://evil.example/login/device"}"#,
            r#"{"device_code":"d","user_code":"WDJB-MJHT","verification_uri":"https://github.com/login/device?x=1"}"#,
            r#"{"device_code":"d","user_code":"wdjb-mjht","verification_uri":"https://github.com/login/device"}"#,
            r#"{"device_code":"d","user_code":"WDJBMJHT1","verification_uri":"https://github.com/login/device"}"#,
            r#"{"device_code":"a b","user_code":"WDJB-MJHT","verification_uri":"https://github.com/login/device"}"#,
            r#"{"device_code":"","user_code":"WDJB-MJHT","verification_uri":"https://github.com/login/device"}"#,
            r#"{"user_code":"WDJB-MJHT","verification_uri":"https://github.com/login/device"}"#,
            r#"{"error":"something_else"}"#,
            "<html><body>proxy login</body></html>",
            "",
            "[]",
        ];
        for json in cases {
            let err = device(json).unwrap_err();
            assert_eq!(err.kind(), HostingErrorKind::BadResponse, "{json}");
        }
        let err = device(r#"{"error":"device_flow_disabled"}"#).unwrap_err();
        assert_eq!(err.kind(), HostingErrorKind::NotConfigured);
    }

    #[test]
    fn maps_token_poll_answers() {
        let poll = |json: &str| parse_token(json.as_bytes(), 5);
        assert!(matches!(
            poll(r#"{"error":"authorization_pending"}"#),
            Ok(LoginPoll::Pending)
        ));
        assert!(matches!(
            poll(r#"{"error":"slow_down","interval":10}"#),
            Ok(LoginPoll::SlowDown { interval_secs: 10 })
        ));
        assert!(matches!(
            poll(r#"{"error":"slow_down"}"#),
            Ok(LoginPoll::SlowDown { interval_secs: 10 })
        ));
        assert!(matches!(
            poll(r#"{"error":"expired_token"}"#),
            Ok(LoginPoll::Expired)
        ));
        assert!(matches!(
            poll(r#"{"error":"access_denied"}"#),
            Ok(LoginPoll::Denied)
        ));
        assert_eq!(
            poll(r#"{"error":"incorrect_client_credentials"}"#)
                .unwrap_err()
                .kind(),
            HostingErrorKind::NotConfigured
        );
        assert_eq!(
            poll(r#"{"error":"unsupported_grant_type"}"#)
                .unwrap_err()
                .kind(),
            HostingErrorKind::BadResponse
        );

        let granted = poll(
            r#"{"access_token":"gho_TESTSECRET0123456789","token_type":"bearer","scope":"repo",
               "refresh_token":"ghr_x"}"#,
        )
        .unwrap();
        let LoginPoll::Granted { token, workflow } = &granted else {
            panic!("{granted:?}");
        };
        assert_eq!(token.expose_secret(), "gho_TESTSECRET0123456789");
        assert!(!workflow);
        assert!(!format!("{granted:?}").contains("TESTSECRET"));
        // 받은 권한에서 workflow가 있는지만 본다(쉼표·공백 목록, D-135).
        for (scope, want) in [
            ("repo", false),
            ("repo,workflow", true),
            ("workflow repo", true),
            ("", false),
            ("workflows", false),
        ] {
            let body = format!(
                r#"{{"access_token":"gho_TESTSECRET0123456789","token_type":"bearer","scope":"{scope}"}}"#
            );
            let Ok(LoginPoll::Granted { workflow, .. }) = poll(&body) else {
                panic!("{scope}");
            };
            assert_eq!(workflow, want, "{scope}");
        }

        let bad_cases = [
            r#"{"access_token":"gho_x","token_type":"mac"}"#,
            r#"{"access_token":"gho_x"}"#,
            r#"{"access_token":"","token_type":"bearer"}"#,
            r#"{"access_token":"gho x","token_type":"bearer"}"#,
            r#"{"token_type":"bearer"}"#,
            "not json",
        ];
        for json in bad_cases {
            assert_eq!(
                poll(json).unwrap_err().kind(),
                HostingErrorKind::BadResponse,
                "{json}"
            );
        }
    }

    #[test]
    fn reads_user() {
        let user =
            parse_user(br#"{"login":"octocat","id":583231,"name":"The Octocat","x":1}"#).unwrap();
        assert_eq!(
            user,
            HostUser {
                login: "octocat".into(),
                id: 583231,
                name: Some("The Octocat".into())
            }
        );
        let user = parse_user(br#"{"login":"octocat","id":1,"name":null}"#).unwrap();
        assert_eq!(user.name, None);
        for json in [
            r#"{"login":"../x","id":1}"#,
            r#"{"login":"octocat"}"#,
            r#"{"login":"octocat","id":-1}"#,
            "<html>",
        ] {
            assert_eq!(
                parse_user(json.as_bytes()).unwrap_err().kind(),
                HostingErrorKind::BadResponse,
                "{json}"
            );
        }
    }
}
