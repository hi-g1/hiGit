//! 비밀 파일 검사 규칙(D-040, D-077). 정규식 crate 없이 이름·접두사·글자 종류·길이로 본다.
//!
//! 결과에는 규칙 이름과 줄 번호만 담는다. 일치한 글자는 어디에도 담지 않는다.

use crate::view::SecretRule;

/// 이 크기 이상이면 GitHub 한도(100MB)에 닿기 전에 알린다.
pub(crate) const LARGE_BYTES: u64 = 50 * 1024 * 1024;
/// 파일마다 앞에서부터 이만큼만 내용을 본다.
pub(crate) const SCAN_BYTES: usize = 256 * 1024;

/// 경로 이름으로 걸리는 규칙.
pub(crate) fn name_rule(path: &str) -> Option<SecretRule> {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    if name == ".env"
        || (name.starts_with(".env.")
            && ![".example", ".sample", ".template"]
                .iter()
                .any(|s| name.ends_with(s)))
    {
        return Some(SecretRule::EnvFile);
    }
    if matches!(
        name.as_str(),
        "id_rsa" | "id_dsa" | "id_ecdsa" | "id_ed25519"
    ) {
        return Some(SecretRule::SshKey);
    }
    if [".pem", ".key", ".p12", ".pfx", ".keystore", ".jks"]
        .iter()
        .any(|ext| name.ends_with(ext))
    {
        return Some(SecretRule::KeyFile);
    }
    if name == "credentials.json"
        || (name.starts_with("service-account") && name.ends_with(".json"))
    {
        return Some(SecretRule::Credentials);
    }
    None
}

/// 내용으로 걸리는 첫 규칙과 그 줄 번호(1부터).
pub(crate) fn content_rule(bytes: &[u8]) -> Option<(SecretRule, u32)> {
    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(SCAN_BYTES)]);
    for (i, line) in text.lines().enumerate() {
        let n = u32::try_from(i + 1).unwrap_or(u32::MAX);
        if line.contains("-----BEGIN ") && line.contains("PRIVATE KEY-----") {
            return Some((SecretRule::PrivateKey, n));
        }
        if let Some(rule) = token_in(line) {
            return Some((rule, n));
        }
    }
    None
}

/// (접두사, 뒤에 와야 할 글자 수, 허용 글자, 규칙)
type TokenRule = (&'static str, usize, fn(char) -> bool, SecretRule);

fn alnum(c: char) -> bool {
    c.is_ascii_alphanumeric()
}
fn alnum_underscore(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
fn upper_digit(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit()
}
fn alnum_dash(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}
fn key_chars(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

const TOKENS: &[TokenRule] = &[
    ("ghp_", 36, alnum, SecretRule::GithubToken),
    ("gho_", 36, alnum, SecretRule::GithubToken),
    ("ghu_", 36, alnum, SecretRule::GithubToken),
    ("ghs_", 36, alnum, SecretRule::GithubToken),
    ("ghr_", 36, alnum, SecretRule::GithubToken),
    ("github_pat_", 22, alnum_underscore, SecretRule::GithubToken),
    ("AKIA", 16, upper_digit, SecretRule::AwsKey),
    ("xoxb-", 10, alnum_dash, SecretRule::SlackToken),
    ("xoxp-", 10, alnum_dash, SecretRule::SlackToken),
    ("sk-", 20, key_chars, SecretRule::ApiKey),
];

/// 줄 안에서 토큰 모양을 찾는다. 접두사 앞이 글자·숫자면(`disk-…`) 토큰으로 보지 않는다.
fn token_in(line: &str) -> Option<SecretRule> {
    for (prefix, min, allowed, rule) in TOKENS {
        let mut from = 0;
        while let Some(pos) = line[from..].find(prefix) {
            let start = from + pos;
            let before_ok = line[..start]
                .chars()
                .next_back()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'));
            let run = line[start + prefix.len()..]
                .chars()
                .take_while(|c| allowed(*c))
                .count();
            if before_ok && run >= *min {
                return Some(*rule);
            }
            from = start + prefix.len();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(name_rule(".env"), Some(SecretRule::EnvFile));
        assert_eq!(name_rule("app/.env.local"), Some(SecretRule::EnvFile));
        assert_eq!(name_rule(".env.example"), None);
        assert_eq!(name_rule("config/.env.sample"), None);
        assert_eq!(name_rule("home/id_ed25519"), Some(SecretRule::SshKey));
        assert_eq!(name_rule("id_ed25519.pub"), None);
        assert_eq!(name_rule("cert/server.PEM"), Some(SecretRule::KeyFile));
        assert_eq!(name_rule("credentials.json"), Some(SecretRule::Credentials));
        assert_eq!(
            name_rule("service-account-prod.json"),
            Some(SecretRule::Credentials)
        );
        assert_eq!(name_rule("한글 문서.md"), None);
        assert_eq!(name_rule("keyboard.ts"), None);
    }

    #[test]
    fn contents() {
        let token = format!("GITHUB_TOKEN=ghp_{}", "a1".repeat(18));
        assert_eq!(
            content_rule(token.as_bytes()),
            Some((SecretRule::GithubToken, 1))
        );
        let pem = "설명\n-----BEGIN OPENSSH PRIVATE KEY-----\nxx\n";
        assert_eq!(
            content_rule(pem.as_bytes()),
            Some((SecretRule::PrivateKey, 2))
        );
        let aws = "a\nb\nkey = AKIAABCDEFGHIJKLMNOP\n";
        assert_eq!(content_rule(aws.as_bytes()), Some((SecretRule::AwsKey, 3)));
        let slack = "token: xoxb-1234567890-abc";
        assert_eq!(
            content_rule(slack.as_bytes()),
            Some((SecretRule::SlackToken, 1))
        );
        let api = format!("OPENAI_API_KEY=sk-{}", "x".repeat(24));
        assert_eq!(content_rule(api.as_bytes()), Some((SecretRule::ApiKey, 1)));
        // 짧거나 단어 속에 든 것은 아니다.
        for clean in [
            "ghp_x",
            "risk-management-is-important-for-everyone",
            "disk-0123456789012345678901234",
            "AKIA는 접두사일 뿐",
            "평범한 한글 문장",
        ] {
            assert_eq!(content_rule(clean.as_bytes()), None, "{clean}");
        }
    }

    #[test]
    fn only_the_head_is_scanned() {
        let mut bytes = vec![b'a'; SCAN_BYTES];
        bytes.extend_from_slice(format!("\nghp_{}", "a".repeat(40)).as_bytes());
        assert_eq!(content_rule(&bytes), None);
    }
}
