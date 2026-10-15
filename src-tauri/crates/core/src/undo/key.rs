//! journal 서명 키(D-063). 앱 데이터 폴더에 한 번 만들어 둔다.

use std::path::Path;

use hmac::{Hmac, Mac};
use sha2::Sha256;

const FILE_NAME: &str = "journal.key";
/// HMAC-SHA256의 블록 크기와 같은 길이라 키를 늘리거나 줄이지 않고 그대로 쓴다.
const KEY_LEN: usize = 64;

#[derive(Clone)]
pub struct JournalKey([u8; KEY_LEN]);

impl std::fmt::Debug for JournalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 키 값은 Debug 출력에 내보내지 않는다(D-040).
        f.write_str("JournalKey(***)")
    }
}

impl JournalKey {
    /// 메모리에만 있는 새 키(테스트, 앱 데이터 폴더를 못 쓸 때). 다음 실행에서는 이전 journal을 되돌리지 않는다.
    pub fn ephemeral() -> Self {
        let mut bytes = [0u8; KEY_LEN];
        if getrandom::fill(&mut bytes).is_err() {
            // 무작위 값을 못 얻으면 시각으로라도 채운다. 남이 만든 journal을 막는 목적이라 충분하다.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            for (i, b) in bytes.iter_mut().enumerate() {
                *b = (now >> ((i % 16) * 8)) as u8 ^ (i % 256) as u8;
            }
        }
        Self(bytes)
    }

    /// `dir/journal.key`를 읽고, 없거나 깨졌으면 새로 만들어 쓴다.
    pub fn load_or_create(dir: &Path) -> Self {
        let path = dir.join(FILE_NAME);
        if let Ok(bytes) = std::fs::read(&path)
            && let Ok(key) = <[u8; KEY_LEN]>::try_from(bytes.as_slice())
        {
            return Self(key);
        }
        let key = Self::ephemeral();
        let _ = platform::write_private_file(&path, &key.0);
        key
    }

    pub(crate) fn sign(&self, text: &str) -> String {
        let mut mac = self.mac();
        mac.update(text.as_bytes());
        hex(&mac.finalize().into_bytes())
    }

    pub(crate) fn verify(&self, text: &str, sig: &str) -> bool {
        let Some(bytes) = unhex(sig) else {
            return false;
        };
        let mut mac = self.mac();
        mac.update(text.as_bytes());
        mac.verify_slice(&bytes).is_ok()
    }

    fn mac(&self) -> Hmac<Sha256> {
        let key = hmac::digest::Key::<Hmac<Sha256>>::from_slice(&self.0);
        <Hmac<Sha256> as hmac::digest::KeyInit>::new(key)
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify() {
        let key = JournalKey::ephemeral();
        let sig = key.sign("줄");
        assert!(key.verify("줄", &sig));
        assert!(!key.verify("다른 줄", &sig));
        assert!(!JournalKey::ephemeral().verify("줄", &sig));
        assert!(!key.verify("줄", "zz"));
        assert_eq!(format!("{key:?}"), "JournalKey(***)");
    }

    #[test]
    fn key_file_is_reused() {
        let dir = tempfile::tempdir().unwrap();
        let a = JournalKey::load_or_create(dir.path());
        let b = JournalKey::load_or_create(dir.path());
        assert!(b.verify("x", &a.sign("x")));
    }
}
