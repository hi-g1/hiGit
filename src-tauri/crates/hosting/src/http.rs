//! 바꿔 끼울 수 있는 HTTP 전송. 실제 구현은 ureq, 시험은 가짜 전송을 쓴다.
//!
//! 토큰은 [`HttpRequest::bearer`]에만 두고 전송이 `Authorization` 헤더를 만든다.
//! 요청·응답의 `Debug`는 토큰과 본문을 찍지 않는다(응답 본문에 토큰이 올 수 있다).

use std::fmt;

use secrecy::SecretString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

/// 보낼 요청.
#[derive(Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    /// 있으면 전송이 `Authorization: Bearer …`로 보낸다.
    pub bearer: Option<SecretString>,
    /// `Authorization`은 여기 넣지 않는다.
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            bearer: None,
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn bearer(mut self, token: SecretString) -> Self {
        self.bearer = Some(token);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("bearer", &self.bearer.as_ref().map(|_| "<가림>"))
            .field("headers", &self.headers)
            .field("body_len", &self.body.as_ref().map(Vec::len))
            .finish()
    }
}

/// 받은 응답. 상태가 4xx·5xx여도 응답이다(오류로 바꾸는 것은 부른 쪽).
#[derive(Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// 이름이 같은 첫 헤더 값(대소문자 무시).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

impl fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body_len", &self.body.len())
            .finish()
    }
}

/// 응답을 받지 못한 까닭. 메시지에 URL·헤더를 넣지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    Timeout,
    Connect,
    /// 응답 본문이 상한을 넘었다.
    TooLarge,
    Other(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("timed out"),
            Self::Connect => f.write_str("could not connect"),
            Self::TooLarge => f.write_str("response too large"),
            Self::Other(detail) => write!(f, "transport error: {detail}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// HTTP 요청 하나를 보내고 응답을 받는다. 리디렉트는 따르지 않고 그대로 돌려준다.
pub trait HttpTransport: Send + Sync {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_TOKEN: &str = "gho_TESTSECRET0123456789";

    #[test]
    fn request_debug_hides_bearer_and_body() {
        let req = HttpRequest::new(Method::Post, "https://api.github.com/user")
            .bearer(SecretString::from(FAKE_TOKEN.to_owned()))
            .header("Accept", "application/json")
            .body(format!("device_code={FAKE_TOKEN}").into_bytes());
        let shown = format!("{req:?}");
        assert!(!shown.contains(FAKE_TOKEN), "{shown}");
        assert!(shown.contains("<가림>"), "{shown}");
    }

    #[test]
    fn response_debug_hides_body() {
        let res = HttpResponse {
            status: 200,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: format!(r#"{{"access_token":"{FAKE_TOKEN}"}}"#).into_bytes(),
        };
        let shown = format!("{res:?}");
        assert!(!shown.contains(FAKE_TOKEN), "{shown}");
        assert_eq!(res.header("content-type"), Some("application/json"));
        assert_eq!(res.header("x-missing"), None);
    }
}
