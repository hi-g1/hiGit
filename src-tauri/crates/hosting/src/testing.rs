//! 시험용 가짜 전송. 다른 크레이트의 시험도 쓰므로 `cfg(test)`가 아니다.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::http::{HttpRequest, HttpResponse, HttpTransport, TransportError};

/// 넣어 둔 응답을 차례로 돌려주고 받은 요청을 기록한다. 응답이 없으면 `Connect` 오류.
#[derive(Default)]
pub struct FakeTransport {
    responses: Mutex<VecDeque<Result<HttpResponse, TransportError>>>,
    requests: Mutex<Vec<HttpRequest>>,
}

impl FakeTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// JSON 응답 하나를 넣는다.
    pub fn push_json(&self, status: u16, body: &str) {
        self.push(HttpResponse {
            status,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: body.as_bytes().to_vec(),
        });
    }

    pub fn push(&self, response: HttpResponse) {
        lock(&self.responses).push_back(Ok(response));
    }

    pub fn push_error(&self, error: TransportError) {
        lock(&self.responses).push_back(Err(error));
    }

    /// 지금까지 받은 요청.
    pub fn requests(&self) -> Vec<HttpRequest> {
        lock(&self.requests).clone()
    }

    /// 아직 쓰지 않은 응답 수.
    pub fn pending(&self) -> usize {
        lock(&self.responses).len()
    }
}

impl HttpTransport for FakeTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        lock(&self.requests).push(req.clone());
        lock(&self.responses)
            .pop_front()
            .unwrap_or(Err(TransportError::Connect))
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
