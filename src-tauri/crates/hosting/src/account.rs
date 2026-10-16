//! 계정 이메일(D-186). 주소는 개인정보라 `Debug`가 찍지 않고, core가 비교한 뒤 버린다.

use std::fmt;

use serde::Deserialize;

use crate::error::{HostingError, HostingErrorKind};

/// 한 번에 읽는 이메일 수(GitHub 한 쪽 최대).
pub const ACCOUNT_EMAILS_MAX: u32 = 100;
const EMAIL_MAX_BYTES: usize = 254;

/// 계정에 등록된 이메일 하나.
#[derive(Clone, PartialEq, Eq)]
pub struct AccountEmail {
    pub address: String,
    pub verified: bool,
}

impl fmt::Debug for AccountEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountEmail")
            .field("address", &"<가림>")
            .field("verified", &self.verified)
            .finish()
    }
}

#[derive(Deserialize)]
struct EmailJson {
    email: String,
    #[serde(default)]
    verified: bool,
}

fn plausible(address: &str) -> bool {
    address.len() <= EMAIL_MAX_BYTES
        && address.contains('@')
        && !address.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// `GET user/emails`. 모양이 이상한 주소는 뺀다.
pub(crate) fn parse_emails(body: &[u8]) -> Result<Vec<AccountEmail>, HostingError> {
    let items: Vec<EmailJson> = serde_json::from_slice(body).map_err(|_| {
        HostingError::with_detail(HostingErrorKind::BadResponse, "emails are not JSON")
    })?;
    Ok(items
        .into_iter()
        .filter(|e| plausible(&e.email))
        .map(|e| AccountEmail {
            address: e.email,
            verified: e.verified,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emails_are_read_and_hidden_in_debug() {
        let list = parse_emails(
            br#"[
            {"email":"me@example.com","verified":true,"primary":true,"visibility":"private"},
            {"email":"1+me@users.noreply.github.com","verified":true,"primary":false,"visibility":null},
            {"email":"other@example.com","verified":false},
            {"email":"no-at-sign","verified":true},
            {"email":"a b@example.com","verified":true}
        ]"#,
        )
        .unwrap();
        assert_eq!(list.len(), 3);
        assert!(list[0].verified && !list[2].verified);
        let shown = format!("{list:?}");
        assert!(
            !shown.contains("example.com") && !shown.contains("noreply"),
            "{shown}"
        );
        assert!(parse_emails(b"nope").is_err());
    }
}
