//! GitHub 요청의 공통 헤더와 응답 상태 → 오류 표.

use crate::error::{HostingError, HostingErrorKind};
use crate::http::{HttpResponse, TransportError};

pub(crate) const USER_AGENT: &str = concat!("higit/", env!("CARGO_PKG_VERSION"));
pub(crate) const API_ACCEPT: &str = "application/vnd.github+json";
pub(crate) const API_VERSION_HEADER: &str = "X-GitHub-Api-Version";
pub(crate) const API_VERSION: &str = "2022-11-28";

/// 2xx면 `Ok`. 아니면 상태와 헤더로 오류 종류를 정한다.
pub(crate) fn check_status(res: &HttpResponse) -> Result<(), HostingError> {
    if (200..300).contains(&res.status) {
        return Ok(());
    }
    Err(status_error(res))
}

pub(crate) fn status_error(res: &HttpResponse) -> HostingError {
    let kind = match res.status {
        401 => HostingErrorKind::Unauthorized,
        403 => HostingErrorKind::Forbidden {
            // 보조 한도는 403이면서 Retry-After가 온다.
            rate_limited: res.header("x-ratelimit-remaining") == Some("0")
                || res.header("retry-after").is_some(),
            sso: res.header("x-github-sso").is_some(),
        },
        429 => HostingErrorKind::Forbidden {
            rate_limited: true,
            sso: false,
        },
        // 410: 저장소가 막혔거나 지워짐. 451: 법적 사유로 막힘.
        404 | 410 => HostingErrorKind::NotFound,
        451 => HostingErrorKind::Forbidden {
            rate_limited: false,
            sso: false,
        },
        301 | 302 | 303 | 307 | 308 => HostingErrorKind::Moved,
        500..=599 => HostingErrorKind::Unavailable,
        _ => HostingErrorKind::BadResponse,
    };
    HostingError::with_detail(kind, &format!("HTTP {}", res.status))
}

pub(crate) fn transport_error(err: TransportError) -> HostingError {
    match err {
        TransportError::Timeout => {
            HostingError::with_detail(HostingErrorKind::Unreachable, "timed out")
        }
        TransportError::Connect => {
            HostingError::with_detail(HostingErrorKind::Unreachable, "could not connect")
        }
        TransportError::TooLarge => {
            HostingError::with_detail(HostingErrorKind::BadResponse, "response too large")
        }
        TransportError::Other(detail) => {
            HostingError::with_detail(HostingErrorKind::Unreachable, &detail)
        }
    }
}

/// `application/x-www-form-urlencoded` 본문. 이름과 값을 모두 퍼센트 인코딩한다.
pub(crate) fn form_body(fields: &[(&str, &str)]) -> Vec<u8> {
    use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
    // RFC 3986의 예약되지 않은 글자만 그대로 둔다.
    const FORM: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~');
    fields
        .iter()
        .map(|(name, value)| {
            format!(
                "{}={}",
                utf8_percent_encode(name, FORM),
                utf8_percent_encode(value, FORM)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
        .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(status: u16, headers: &[(&str, &str)]) -> HttpResponse {
        HttpResponse {
            status,
            headers: headers
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            body: Vec::new(),
        }
    }

    #[test]
    fn status_table() {
        let forbidden = |rate_limited, sso| HostingErrorKind::Forbidden { rate_limited, sso };
        let cases = [
            (res(401, &[]), HostingErrorKind::Unauthorized),
            (res(403, &[]), forbidden(false, false)),
            (
                res(403, &[("X-RateLimit-Remaining", "0")]),
                forbidden(true, false),
            ),
            (
                res(403, &[("X-RateLimit-Remaining", "12")]),
                forbidden(false, false),
            ),
            (res(403, &[("Retry-After", "60")]), forbidden(true, false)),
            (
                res(
                    403,
                    &[(
                        "X-GitHub-SSO",
                        "required; url=https://github.com/orgs/x/sso",
                    )],
                ),
                forbidden(false, true),
            ),
            (res(429, &[("Retry-After", "60")]), forbidden(true, false)),
            (res(404, &[]), HostingErrorKind::NotFound),
            (res(410, &[]), HostingErrorKind::NotFound),
            (res(451, &[]), forbidden(false, false)),
            (res(301, &[]), HostingErrorKind::Moved),
            (res(304, &[]), HostingErrorKind::BadResponse),
            (res(500, &[]), HostingErrorKind::Unavailable),
            (res(503, &[]), HostingErrorKind::Unavailable),
            (res(418, &[]), HostingErrorKind::BadResponse),
        ];
        for (response, want) in cases {
            assert_eq!(
                check_status(&response).unwrap_err().kind(),
                want,
                "{response:?}"
            );
        }
        assert!(check_status(&res(200, &[])).is_ok());
        assert!(check_status(&res(201, &[])).is_ok());
    }

    #[test]
    fn transport_errors() {
        assert_eq!(
            transport_error(TransportError::Timeout).kind(),
            HostingErrorKind::Unreachable
        );
        assert_eq!(
            transport_error(TransportError::Connect).kind(),
            HostingErrorKind::Unreachable
        );
        assert_eq!(
            transport_error(TransportError::TooLarge).kind(),
            HostingErrorKind::BadResponse
        );
    }

    #[test]
    fn form_body_encodes_everything() {
        let body = form_body(&[
            ("client_id", "abc123"),
            ("scope", "repo"),
            ("x", "a b&c=d/한"),
        ]);
        assert_eq!(
            String::from_utf8(body).unwrap(),
            "client_id=abc123&scope=repo&x=a%20b%26c%3Dd%2F%ED%95%9C"
        );
    }
}
