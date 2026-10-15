//! 비밀 파일 검사(D-040, D-077). 결과에는 경로·규칙·줄 번호만 담는다.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SecretCheckRequest {
    /// 넣은 파일이 없으면 고친 파일을 모두 넣고 저장할 때처럼 모든 고친 파일을 본다(D-062).
    pub stage_all: bool,
}

/// 걸린 까닭. 화면이 문구를 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SecretRule {
    /// `.env`, `.env.local` 등
    EnvFile,
    /// `id_rsa`, `id_ed25519` 등
    SshKey,
    /// `.pem`, `.key`, `.p12` 등
    KeyFile,
    /// `credentials.json`, `service-account*.json`
    Credentials,
    /// `-----BEGIN … PRIVATE KEY-----`
    PrivateKey,
    GithubToken,
    AwsKey,
    SlackToken,
    ApiKey,
    /// 50MB 이상
    LargeFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SecretFinding {
    pub path: String,
    pub rule: SecretRule,
    /// 내용 규칙이면 걸린 줄(1부터).
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SecretCheckView {
    pub findings: Vec<SecretFinding>,
    /// 검사 상한(파일 200개)을 넘어 보지 않은 파일 수.
    pub skipped: u32,
}
