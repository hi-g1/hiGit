//! `HostingProvider` 트레이트와 GitHub 구현. 반복 4에서 채운다.
//!
//! 다른 내부 크레이트에 의존하지 않는다.

#![forbid(unsafe_code)]

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
