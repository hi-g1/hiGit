//! `HostingProvider` 트레이트와 GitHub 구현(반복 4).
//!
//! 다른 내부 크레이트에 의존하지 않는다. 네트워크는 [`HttpTransport`] 뒤에 두어
//! 시험은 가짜 전송으로 돈다. 토큰은 `SecretString`으로만 다룬다(D-040).

#![forbid(unsafe_code)]

mod config;
mod error;
mod http;
mod remote;

pub use config::{GITHUB_API_BASE, GITHUB_SCOPE, GITHUB_WEB_BASE, GitHubConfig, valid_client_id};
pub use error::{DETAIL_MAX_CHARS, HostingError, HostingErrorKind, ValidationKind};
pub use http::{HttpRequest, HttpResponse, HttpTransport, Method, TransportError};
pub use remote::{GITHUB_HOST, RemoteRepo, parse_remote, valid_name, valid_owner};

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

    // G2 스모크 시험: rustls(ring) + OS 인증서 확인으로 에이전트를 만들 수 있고,
    // 아무도 듣지 않는 포트는 패닉 없이 연결 실패로 끝난다.
    #[test]
    fn rustls_agent_reports_refused_connection() {
        let tls = TlsConfig::builder()
            .provider(TlsProvider::Rustls)
            .unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .root_certs(RootCerts::PlatformVerifier)
            .build();
        let agent = ureq::Agent::config_builder()
            .tls_config(tls)
            .proxy(None)
            .timeout_connect(Some(Duration::from_secs(5)))
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .new_agent();
        let err = agent
            .get("https://127.0.0.1:1/")
            .call()
            .expect_err("포트 1에는 서버가 없다");
        assert!(
            matches!(
                err,
                ureq::Error::Io(_) | ureq::Error::ConnectionFailed | ureq::Error::Timeout(_)
            ),
            "{err:?}"
        );
    }
}
