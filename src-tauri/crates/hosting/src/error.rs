//! 호스팅 서비스 호출의 실패.
//!
//! `Display`·`Debug`에는 종류만 찍는다. 서버가 준 설명은 앞 200자만 [`HostingError::detail`]로
//! 주고, 화면에 보이기 전에 core가 가린다(redact).

use std::fmt;

/// 설명으로 남기는 최대 글자 수.
pub const DETAIL_MAX_CHARS: usize = 200;

/// 422 응답을 나눈 것.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationKind {
    /// 같은 브랜치의 검토 요청이 이미 있다.
    AlreadyExists,
    /// 두 브랜치 사이에 다른 버전이 없다.
    NoDifference,
    /// 그 밖의 입력 오류(초안이 안 되는 저장소 포함).
    Invalid,
}

/// 실패 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostingErrorKind {
    /// client id가 없거나 GitHub가 device flow를 거절했다.
    NotConfigured,
    /// 토큰이 없거나 더는 쓸 수 없다(401).
    Unauthorized,
    /// 권한이 없거나(403) 요청 한도에 걸렸거나 조직 SSO 승인이 필요하다.
    Forbidden { rate_limited: bool, sso: bool },
    /// 저장소나 검토 요청이 없거나 볼 권한이 없다(404).
    NotFound,
    /// 저장소가 다른 곳으로 옮겨졌고 따라갈 수 없다.
    Moved,
    /// 서버가 입력을 거절했다(422).
    Validation(ValidationKind),
    /// 연결하지 못했거나 시간이 지났다.
    Unreachable,
    /// 서버 쪽 문제(5xx)나 잠시 쓸 수 없음.
    Unavailable,
    /// 응답 모양이 예상과 다르다(프록시 로그인 페이지 등).
    BadResponse,
    /// 사람이 취소했다.
    Cancelled,
}

/// 호스팅 서비스 호출 실패. 종류와, 있으면 짧은 설명.
#[derive(Clone, PartialEq, Eq)]
pub struct HostingError {
    kind: HostingErrorKind,
    detail: Option<String>,
}

impl HostingError {
    pub fn new(kind: HostingErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// 설명을 붙인다. 앞 [`DETAIL_MAX_CHARS`]자만 남기고 제어 문자는 공백으로 바꾼다.
    pub fn with_detail(kind: HostingErrorKind, detail: &str) -> Self {
        let text: String = detail
            .chars()
            .take(DETAIL_MAX_CHARS)
            .map(|c| if c.is_control() && c != '\n' { ' ' } else { c })
            .collect();
        let text = text.trim();
        Self {
            kind,
            detail: (!text.is_empty()).then(|| text.to_owned()),
        }
    }

    pub fn kind(&self) -> HostingErrorKind {
        self.kind
    }

    /// 서버나 전송이 준 짧은 설명. 화면에 보이기 전에 가려야 한다.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl From<HostingErrorKind> for HostingError {
    fn from(kind: HostingErrorKind) -> Self {
        Self::new(kind)
    }
}

impl fmt::Debug for HostingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostingError")
            .field("kind", &self.kind)
            .field("has_detail", &self.detail.is_some())
            .finish()
    }
}

impl fmt::Display for HostingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self.kind {
            HostingErrorKind::NotConfigured => "hosting not configured",
            HostingErrorKind::Unauthorized => "unauthorized",
            HostingErrorKind::Forbidden {
                rate_limited: true, ..
            } => "rate limited",
            HostingErrorKind::Forbidden { sso: true, .. } => "sso authorization required",
            HostingErrorKind::Forbidden { .. } => "forbidden",
            HostingErrorKind::NotFound => "not found",
            HostingErrorKind::Moved => "moved",
            HostingErrorKind::Validation(ValidationKind::AlreadyExists) => "already exists",
            HostingErrorKind::Validation(ValidationKind::NoDifference) => "no difference",
            HostingErrorKind::Validation(ValidationKind::Invalid) => "invalid request",
            HostingErrorKind::Unreachable => "unreachable",
            HostingErrorKind::Unavailable => "service unavailable",
            HostingErrorKind::BadResponse => "unexpected response",
            HostingErrorKind::Cancelled => "cancelled",
        };
        f.write_str(text)
    }
}

impl std::error::Error for HostingError {}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_TOKEN: &str = "gho_TESTSECRET0123456789";

    #[test]
    fn display_and_debug_hide_detail() {
        let err = HostingError::with_detail(
            HostingErrorKind::BadResponse,
            &format!("server said {FAKE_TOKEN}"),
        );
        let shown = format!("{err} {err:?}");
        assert!(!shown.contains(FAKE_TOKEN), "{shown}");
        // 설명은 호출한 쪽이 가리도록 그대로 준다.
        assert!(err.detail().unwrap().contains("server said"));
    }

    #[test]
    fn detail_is_short_and_has_no_control_chars() {
        let long = format!("{}\u{1b}[31m", "가".repeat(300));
        let err = HostingError::with_detail(HostingErrorKind::Unavailable, &long);
        let detail = err.detail().unwrap();
        assert_eq!(detail.chars().count(), DETAIL_MAX_CHARS);
        assert!(!detail.contains('\u{1b}'));

        let err = HostingError::with_detail(HostingErrorKind::Unavailable, "a\u{7}b\nc");
        assert_eq!(err.detail(), Some("a b\nc"));
        assert_eq!(
            HostingError::with_detail(HostingErrorKind::Unavailable, "  ").detail(),
            None
        );
    }

    #[test]
    fn kind_round_trips() {
        let err: HostingError = HostingErrorKind::Forbidden {
            rate_limited: true,
            sso: false,
        }
        .into();
        assert_eq!(err.to_string(), "rate limited");
        assert_eq!(
            err.kind(),
            HostingErrorKind::Forbidden {
                rate_limited: true,
                sso: false
            }
        );
    }
}
