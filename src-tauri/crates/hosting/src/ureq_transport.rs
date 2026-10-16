//! ureq로 만든 실제 전송(D-110).
//!
//! https만, 리디렉트는 따르지 않고(부른 쪽이 같은 호스트일 때만 직접 따른다), 연결 5초·전체 15초,
//! 응답 본문 2MB. 프록시는 환경 변수(`HTTPS_PROXY` 등)를 따르고, 인증서는 OS 저장소로 확인한다.

use std::sync::Arc;
use std::time::Duration;

use secrecy::ExposeSecret;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

use crate::http::{HttpRequest, HttpResponse, HttpTransport, Method, TransportError};

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(15);
pub const BODY_MAX_BYTES: u64 = 2 * 1024 * 1024;

pub struct UreqTransport {
    agent: ureq::Agent,
    body_max: u64,
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl UreqTransport {
    pub fn new() -> Self {
        Self::build(true, CONNECT_TIMEOUT, TOTAL_TIMEOUT, BODY_MAX_BYTES)
    }

    /// 시험용: 평문 로컬 서버를 허용하고 시간·크기 상한을 바꾼다. 환경 변수 프록시는 쓰지 않는다.
    #[doc(hidden)]
    pub fn allow_http_for_tests(total: Duration, body_max: u64) -> Self {
        Self::build(false, CONNECT_TIMEOUT, total, body_max)
    }

    fn build(https_only: bool, connect: Duration, total: Duration, body_max: u64) -> Self {
        let tls = TlsConfig::builder()
            .provider(TlsProvider::Rustls)
            .unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .root_certs(RootCerts::PlatformVerifier)
            .build();
        let mut config = ureq::Agent::config_builder()
            .tls_config(tls)
            .https_only(https_only)
            // 4xx·5xx도 응답으로 받는다(오류로 바꾸는 것은 부른 쪽).
            .http_status_as_error(false)
            // 0이면 리디렉트 응답을 그대로 돌려준다.
            .max_redirects(0)
            .timeout_connect(Some(connect))
            .timeout_global(Some(total));
        if !https_only {
            config = config.proxy(None);
        }
        Self {
            agent: config.build().new_agent(),
            body_max,
        }
    }
}

impl HttpTransport for UreqTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let result = match req.method {
            Method::Get => {
                let mut builder = self.agent.get(&req.url);
                for (name, value) in &req.headers {
                    builder = builder.header(name, value);
                }
                if let Some(token) = &req.bearer {
                    builder = builder
                        .header("Authorization", format!("Bearer {}", token.expose_secret()));
                }
                builder.call()
            }
            Method::Post => {
                let mut builder = self.agent.post(&req.url);
                for (name, value) in &req.headers {
                    builder = builder.header(name, value);
                }
                if let Some(token) = &req.bearer {
                    builder = builder
                        .header("Authorization", format!("Bearer {}", token.expose_secret()));
                }
                match &req.body {
                    Some(body) => builder.send(&body[..]),
                    None => builder.send_empty(),
                }
            }
        };
        let mut res = result.map_err(map_error)?;
        let status = res.status().as_u16();
        let headers = res
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|v| (name.as_str().to_owned(), v.to_owned()))
            })
            .collect();
        let body = res
            .body_mut()
            .with_config()
            .limit(self.body_max)
            .read_to_vec()
            .map_err(map_error)?;
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

/// ureq 오류를 종류로만 바꾼다. 메시지에 URL·헤더가 들어갈 수 있어 원문은 버린다.
fn map_error(err: ureq::Error) -> TransportError {
    use std::io::ErrorKind;
    match err {
        ureq::Error::Timeout(_) => TransportError::Timeout,
        ureq::Error::Io(io) if io.kind() == ErrorKind::TimedOut => TransportError::Timeout,
        ureq::Error::Io(_) | ureq::Error::ConnectionFailed | ureq::Error::HostNotFound => {
            TransportError::Connect
        }
        ureq::Error::BodyExceedsLimit(_) => TransportError::TooLarge,
        ureq::Error::RequireHttpsOnly(_) => TransportError::Other("https only".into()),
        ureq::Error::Tls(_) | ureq::Error::Rustls(_) => TransportError::Other("tls".into()),
        ureq::Error::Protocol(_) | ureq::Error::LargeResponseHeader(..) => {
            TransportError::Other("protocol".into())
        }
        ureq::Error::BadUri(_) | ureq::Error::Http(_) => {
            TransportError::Other("bad request".into())
        }
        ureq::Error::ConnectProxyFailed(_) | ureq::Error::InvalidProxyUrl => {
            TransportError::Other("proxy".into())
        }
        _ => TransportError::Other("transport".into()),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;

    use secrecy::SecretString;

    use super::*;

    const TOKEN: &str = "gho_TESTSECRET0123456789";

    /// 요청 하나를 받아 원문을 돌려주고 `reply`를 쓰는 로컬 서버. 주소와 받은 원문 채널.
    fn serve(reply: Vec<u8>, delay: Duration) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut head = String::new();
            let mut length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
                let end = line == "\r\n";
                head.push_str(&line);
                if end {
                    break;
                }
            }
            let mut body = vec![0; length];
            let _ = reader.read_exact(&mut body);
            head.push_str(&String::from_utf8_lossy(&body));
            let _ = tx.send(head);
            std::thread::sleep(delay);
            let mut stream = stream;
            let _ = stream.write_all(&reply);
            let _ = stream.flush();
        });
        (format!("http://{addr}"), rx)
    }

    fn reply(status: &str, headers: &[&str], body: &[u8]) -> Vec<u8> {
        let mut out = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n", body.len());
        for h in headers {
            out.push_str(h);
            out.push_str("\r\n");
        }
        out.push_str("Connection: close\r\n\r\n");
        let mut bytes = out.into_bytes();
        bytes.extend_from_slice(body);
        bytes
    }

    fn transport() -> UreqTransport {
        UreqTransport::allow_http_for_tests(Duration::from_secs(10), 1024)
    }

    #[test]
    fn gets_json_and_sends_bearer_once() {
        let (base, rx) = serve(
            reply(
                "200 OK",
                &["Content-Type: application/json"],
                br#"{"ok":true}"#,
            ),
            Duration::ZERO,
        );
        let req = HttpRequest::new(Method::Get, format!("{base}/user"))
            .header("Accept", "application/json")
            .bearer(SecretString::from(TOKEN.to_owned()));
        let res = transport().send(&req).unwrap();
        assert_eq!(res.status, 200);
        assert_eq!(res.body, br#"{"ok":true}"#);
        assert_eq!(res.header("content-type"), Some("application/json"));

        let seen = rx.recv().unwrap();
        let first = seen.lines().next().unwrap();
        assert_eq!(first, "GET /user HTTP/1.1");
        assert_eq!(seen.matches(TOKEN).count(), 1, "{seen}");
        let auth = seen
            .lines()
            .filter(|l| l.to_ascii_lowercase().starts_with("authorization:"))
            .collect::<Vec<_>>();
        assert_eq!(auth.len(), 1, "{seen}");
        assert!(auth[0].ends_with(&format!("Bearer {TOKEN}")), "{auth:?}");
    }

    #[test]
    fn posts_body() {
        let (base, rx) = serve(reply("201 Created", &[], b"{}"), Duration::ZERO);
        let req = HttpRequest::new(Method::Post, format!("{base}/x"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(b"a=1&b=2".to_vec());
        let res = transport().send(&req).unwrap();
        assert_eq!(res.status, 201);
        let seen = rx.recv().unwrap();
        assert!(seen.starts_with("POST /x HTTP/1.1"), "{seen}");
        assert!(seen.ends_with("a=1&b=2"), "{seen}");
        assert!(
            !seen.to_ascii_lowercase().contains("authorization"),
            "{seen}"
        );
    }

    #[test]
    fn error_status_is_a_response() {
        let (base, _rx) = serve(
            reply("401 Unauthorized", &[], br#"{"message":"Bad credentials"}"#),
            Duration::ZERO,
        );
        let res = transport()
            .send(&HttpRequest::new(Method::Get, format!("{base}/user")))
            .unwrap();
        assert_eq!(res.status, 401);

        let (base, _rx) = serve(reply("404 Not Found", &[], b"{}"), Duration::ZERO);
        let res = transport()
            .send(&HttpRequest::new(Method::Get, format!("{base}/x")))
            .unwrap();
        assert_eq!(res.status, 404);
    }

    #[test]
    fn redirect_is_returned_not_followed() {
        let (base, _rx) = serve(
            reply(
                "301 Moved Permanently",
                &["Location: https://evil.example/x"],
                b"",
            ),
            Duration::ZERO,
        );
        let res = transport()
            .send(&HttpRequest::new(Method::Get, format!("{base}/x")))
            .unwrap();
        assert_eq!(res.status, 301);
        assert_eq!(res.header("location"), Some("https://evil.example/x"));
    }

    #[test]
    fn slow_response_times_out() {
        let (base, _rx) = serve(reply("200 OK", &[], b"{}"), Duration::from_secs(3));
        let slow = UreqTransport::allow_http_for_tests(Duration::from_millis(500), 1024);
        let err = slow
            .send(&HttpRequest::new(Method::Get, format!("{base}/x")))
            .unwrap_err();
        assert_eq!(err, TransportError::Timeout);
    }

    #[test]
    fn large_body_is_too_large() {
        let (base, _rx) = serve(reply("200 OK", &[], &vec![b'a'; 4096]), Duration::ZERO);
        let err = transport()
            .send(&HttpRequest::new(Method::Get, format!("{base}/x")))
            .unwrap_err();
        assert_eq!(err, TransportError::TooLarge);
    }

    #[test]
    fn https_only_refuses_plain_http() {
        let err = UreqTransport::new()
            .send(&HttpRequest::new(Method::Get, "http://127.0.0.1:1/x"))
            .unwrap_err();
        assert!(matches!(err, TransportError::Other(_)), "{err:?}");
        assert!(!err.to_string().contains("127.0.0.1"));
    }

    #[test]
    fn refused_https_connection_is_connect() {
        // TLS 설정으로 에이전트를 만들 수 있고, 아무도 듣지 않는 포트는 연결 실패다.
        let err = UreqTransport::new()
            .send(&HttpRequest::new(Method::Get, "https://127.0.0.1:1/"))
            .unwrap_err();
        assert!(
            matches!(err, TransportError::Connect | TransportError::Timeout),
            "{err:?}"
        );
    }
}
