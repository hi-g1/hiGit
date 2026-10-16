//! hosting 오류 → core 오류. 한 함수에서만 바꾼다.

use hosting::{HostingError, HostingErrorKind, ValidationKind};

use crate::error::{CoreError, GithubFailure};

/// `secrets`(서비스가 쥔 토큰·device_code)는 설명에서 정확한 글자로 지운다. 접두사 없는 값도
/// 가리려는 것이다(화면에 보내기 전 `redact`가 한 번 더 가린다).
pub(crate) fn from_hosting(err: &HostingError, secrets: &[&str]) -> CoreError {
    let kind = match err.kind() {
        HostingErrorKind::NotConfigured => GithubFailure::NotConfigured,
        HostingErrorKind::Unauthorized => GithubFailure::Expired,
        HostingErrorKind::Forbidden {
            rate_limited: true, ..
        } => GithubFailure::RateLimited,
        HostingErrorKind::Forbidden { sso: true, .. } => GithubFailure::SsoRequired,
        HostingErrorKind::Forbidden { .. } => GithubFailure::Forbidden,
        HostingErrorKind::NotFound => GithubFailure::NotFound,
        HostingErrorKind::Moved => GithubFailure::Moved,
        HostingErrorKind::Validation(ValidationKind::AlreadyExists) => {
            GithubFailure::PrAlreadyExists
        }
        HostingErrorKind::Validation(ValidationKind::NoDifference) => GithubFailure::PrNoDifference,
        HostingErrorKind::Validation(ValidationKind::Invalid) => GithubFailure::PrInvalid,
        HostingErrorKind::Unreachable => GithubFailure::Unreachable,
        HostingErrorKind::Unavailable => GithubFailure::Unavailable,
        HostingErrorKind::BadResponse => GithubFailure::BadResponse,
        HostingErrorKind::Cancelled => GithubFailure::Cancelled,
    };
    let detail = err.detail().map(|d| scrub(d, secrets));
    CoreError::Github { kind, detail }
}

pub(crate) fn scrub(text: &str, secrets: &[&str]) -> String {
    let mut out = text.to_owned();
    for secret in secrets.iter().filter(|s| s.len() >= 4) {
        out = out.replace(secret, "***");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_kinds_and_scrubs_secrets() {
        let err = HostingError::with_detail(
            HostingErrorKind::BadResponse,
            "echo 3584d835305 and gho_TESTSECRET0123456789",
        );
        let core = from_hosting(&err, &["3584d835305", "gho_TESTSECRET0123456789"]);
        let CoreError::Github { kind, detail } = &core else {
            panic!("{core:?}");
        };
        assert_eq!(*kind, GithubFailure::BadResponse);
        assert_eq!(detail.as_deref(), Some("echo *** and ***"));

        let cases = [
            (HostingErrorKind::Unauthorized, GithubFailure::Expired),
            (
                HostingErrorKind::Forbidden {
                    rate_limited: true,
                    sso: true,
                },
                GithubFailure::RateLimited,
            ),
            (
                HostingErrorKind::Forbidden {
                    rate_limited: false,
                    sso: true,
                },
                GithubFailure::SsoRequired,
            ),
            (
                HostingErrorKind::NotConfigured,
                GithubFailure::NotConfigured,
            ),
            (HostingErrorKind::Unreachable, GithubFailure::Unreachable),
        ];
        for (from, want) in cases {
            let CoreError::Github { kind, .. } = from_hosting(&HostingError::new(from), &[]) else {
                panic!();
            };
            assert_eq!(kind, want);
        }
    }
}
