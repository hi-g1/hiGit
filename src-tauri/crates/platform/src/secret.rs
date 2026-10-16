//! OS 키체인에 비밀 하나를 쓰고 읽고 지운다(D-040, D-110).
//!
//! Windows는 자격 증명 관리자, Linux는 Secret Service. 평문 파일로 대신하는 길은 만들지 않는다.
//! 값은 [`SecretString`]으로만 주고받아 `Debug`·로그에 찍히지 않는다.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use keyring_core::{CredentialStore, Entry, Error};
use secrecy::{ExposeSecret, SecretString};

/// 키체인에 넣을 수 있는 값의 최대 바이트 수. GitHub 토큰은 수십 바이트다.
pub const SECRET_MAX_BYTES: usize = 4096;

/// 키체인을 쓰지 못한 까닭. 메시지에 비밀 값은 들어가지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeychainError {
    /// 이 컴퓨터에 쓸 수 있는 키체인이 없다(Secret Service 없음, 세션 버스 없음 등).
    Unavailable,
    /// 키체인이 잠겨 있고 잠금 해제를 거절했거나 접근이 막혔다.
    Denied,
    /// 그 밖의 실패. 값은 OS가 준 설명(자세히 보기용)이다.
    Other(String),
}

impl fmt::Display for KeychainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("keychain unavailable"),
            Self::Denied => f.write_str("keychain access denied"),
            Self::Other(detail) => write!(f, "keychain error: {detail}"),
        }
    }
}

impl std::error::Error for KeychainError {}

#[derive(Clone)]
enum Source {
    Os,
    Given(Arc<CredentialStore>),
    Missing,
}

/// 서비스 이름 하나에 묶인 키체인. 계정 이름마다 값 하나를 둔다.
#[derive(Clone)]
pub struct Keychain {
    service: String,
    source: Source,
}

impl fmt::Debug for Keychain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keychain")
            .field("service", &self.service)
            .finish_non_exhaustive()
    }
}

impl Keychain {
    /// OS 키체인을 쓴다. 저장소는 처음 쓸 때 만든다.
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_owned(),
            source: Source::Os,
        }
    }

    /// 주어진 저장소를 쓴다(시험용 모의 저장소 등).
    pub fn with_store(service: &str, store: Arc<CredentialStore>) -> Self {
        Self {
            service: service.to_owned(),
            source: Source::Given(store),
        }
    }

    /// 메모리에만 두는 키체인. 복제본끼리 값을 나눈다. 다른 크레이트의 시험용이다.
    pub fn in_memory(service: &str) -> Self {
        match keyring_core::mock::Store::new() {
            Ok(store) => Self::with_store(service, store),
            Err(_) => Self::unavailable(service),
        }
    }

    /// 늘 [`KeychainError::Unavailable`]을 내는 키체인. 키체인 없는 컴퓨터를 흉내 내는 시험용이다.
    pub fn unavailable(service: &str) -> Self {
        Self {
            service: service.to_owned(),
            source: Source::Missing,
        }
    }

    /// **테스트 전용.** 모의 저장소면 이 계정의 다음 호출 하나를 `Denied`로 실패시킨다.
    #[doc(hidden)]
    pub fn deny_next_for_tests(&self, account: &str) {
        let Source::Given(store) = &self.source else {
            return;
        };
        let Ok(entry) = store.build(&self.service, account, None) else {
            return;
        };
        if let Some(cred) = entry.as_any().downcast_ref::<keyring_core::mock::Cred>() {
            cred.set_error(Error::NoStorageAccess(Box::new(std::io::Error::other(
                "locked",
            ))));
        }
    }

    /// 값을 읽는다. 없으면 `None`.
    pub fn get(&self, account: &str) -> Result<Option<SecretString>, KeychainError> {
        let entry = self.entry(account)?;
        match entry.get_password() {
            Ok(value) if value.len() > SECRET_MAX_BYTES => {
                Err(KeychainError::Other("stored value too long".into()))
            }
            Ok(value) => Ok(Some(SecretString::from(value))),
            Err(Error::NoEntry) => Ok(None),
            Err(err) => Err(map_error(err)),
        }
    }

    /// 값을 쓴다. 있던 값은 덮어쓴다. 4KB를 넘거나 비어 있으면 거절한다.
    pub fn set(&self, account: &str, value: &SecretString) -> Result<(), KeychainError> {
        let text = value.expose_secret();
        if text.is_empty() || text.len() > SECRET_MAX_BYTES {
            return Err(KeychainError::Other("invalid value length".into()));
        }
        let entry = self.entry(account)?;
        entry.set_password(text).map_err(map_error)
    }

    /// 값을 지운다. 없어도 성공이다.
    pub fn delete(&self, account: &str) -> Result<(), KeychainError> {
        let entry = self.entry(account)?;
        match entry.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(err) => Err(map_error(err)),
        }
    }

    fn entry(&self, account: &str) -> Result<Entry, KeychainError> {
        let (store, modifiers) = match &self.source {
            Source::Os => (os_store()?, os_modifiers()),
            Source::Given(store) => (store.clone(), None),
            Source::Missing => return Err(KeychainError::Unavailable),
        };
        store
            .build(&self.service, account, modifiers.as_ref())
            .map_err(map_error)
    }
}

/// Windows는 도메인 로밍 프로필로 퍼지지 않게 이 컴퓨터에만 둔다(기본은 Enterprise).
#[cfg(windows)]
fn os_modifiers() -> Option<HashMap<&'static str, &'static str>> {
    Some(HashMap::from([("persistence", "Local")]))
}

#[cfg(not(windows))]
fn os_modifiers() -> Option<HashMap<&'static str, &'static str>> {
    None
}

fn map_error(err: Error) -> KeychainError {
    match err {
        Error::NoDefaultStore => KeychainError::Unavailable,
        Error::NoEntry => KeychainError::Other("no entry".into()),
        Error::NoStorageAccess(_) => KeychainError::Denied,
        // 설명은 OS가 준 것만 남긴다. BadEncoding 등은 값 바이트를 들고 있으므로 찍지 않는다.
        Error::PlatformFailure(source) => KeychainError::Other(source.to_string()),
        Error::BadEncoding(_) | Error::BadDataFormat(..) => {
            KeychainError::Other("stored value is not text".into())
        }
        Error::TooLong(name, limit) => KeychainError::Other(format!("{name} longer than {limit}")),
        Error::Invalid(name, reason) => KeychainError::Other(format!("{name}: {reason}")),
        Error::Ambiguous(_) => KeychainError::Other("more than one entry".into()),
        Error::BadStoreFormat(detail) | Error::NotSupportedByStore(detail) => {
            KeychainError::Other(detail)
        }
        // keyring-core가 오류 종류를 더해도 비밀을 찍지 않게 이름만 남긴다.
        _ => KeychainError::Other("unknown keychain error".into()),
    }
}

/// 만든 OS 저장소. 실패는 남겨 두지 않아 다음 호출이 다시 시도한다(키체인 데몬이 늦게 뜨는 경우).
static OS_STORE: Mutex<Option<Arc<CredentialStore>>> = Mutex::new(None);

fn os_store() -> Result<Arc<CredentialStore>, KeychainError> {
    let mut guard = OS_STORE.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(store) = guard.as_ref() {
        return Ok(store.clone());
    }
    let store = new_os_store()?;
    *guard = Some(store.clone());
    Ok(store)
}

#[cfg(windows)]
fn new_os_store() -> Result<Arc<CredentialStore>, KeychainError> {
    let store =
        windows_native_keyring_store::Store::new().map_err(|_| KeychainError::Unavailable)?;
    Ok(store)
}

#[cfg(target_os = "linux")]
fn new_os_store() -> Result<Arc<CredentialStore>, KeychainError> {
    let store =
        zbus_secret_service_keyring_store::Store::new().map_err(|_| KeychainError::Unavailable)?;
    Ok(store)
}

#[cfg(not(any(windows, target_os = "linux")))]
fn new_os_store() -> Result<Arc<CredentialStore>, KeychainError> {
    Err(KeychainError::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring_core::api::CredentialStoreApi;

    const SERVICE: &str = "dev.higit.test";
    const ACCOUNT: &str = "github.com";

    fn secret(text: &str) -> SecretString {
        SecretString::from(text.to_owned())
    }

    #[test]
    fn writes_reads_overwrites_and_deletes() {
        let keychain = Keychain::in_memory(SERVICE);
        assert!(keychain.get(ACCOUNT).unwrap().is_none());

        keychain.set(ACCOUNT, &secret("first")).unwrap();
        let got = keychain.get(ACCOUNT).unwrap().unwrap();
        assert_eq!(got.expose_secret(), "first");

        keychain.set(ACCOUNT, &secret("second")).unwrap();
        let got = keychain.get(ACCOUNT).unwrap().unwrap();
        assert_eq!(got.expose_secret(), "second");

        keychain.delete(ACCOUNT).unwrap();
        assert!(keychain.get(ACCOUNT).unwrap().is_none());
        // 없는 값을 지워도 성공.
        keychain.delete(ACCOUNT).unwrap();
    }

    #[test]
    fn clones_share_values_and_accounts_are_separate() {
        let keychain = Keychain::in_memory(SERVICE);
        let copy = keychain.clone();
        keychain.set(ACCOUNT, &secret("token")).unwrap();
        assert_eq!(copy.get(ACCOUNT).unwrap().unwrap().expose_secret(), "token");
        assert!(copy.get("other.example").unwrap().is_none());
    }

    #[test]
    fn rejects_empty_and_too_long_values() {
        let keychain = Keychain::in_memory(SERVICE);
        let long = "a".repeat(SECRET_MAX_BYTES + 1);
        assert!(matches!(
            keychain.set(ACCOUNT, &secret(&long)),
            Err(KeychainError::Other(_))
        ));
        assert!(matches!(
            keychain.set(ACCOUNT, &secret("")),
            Err(KeychainError::Other(_))
        ));
        let max = "a".repeat(SECRET_MAX_BYTES);
        keychain.set(ACCOUNT, &secret(&max)).unwrap();
        assert_eq!(keychain.get(ACCOUNT).unwrap().unwrap().expose_secret(), max);
    }

    #[test]
    fn missing_store_is_unavailable() {
        let keychain = Keychain::unavailable(SERVICE);
        assert_eq!(
            keychain.get(ACCOUNT).unwrap_err(),
            KeychainError::Unavailable
        );
        assert_eq!(
            keychain.set(ACCOUNT, &secret("x")).unwrap_err(),
            KeychainError::Unavailable
        );
        assert_eq!(
            keychain.delete(ACCOUNT).unwrap_err(),
            KeychainError::Unavailable
        );
    }

    #[test]
    fn locked_store_is_denied() {
        let store = keyring_core::mock::Store::new().unwrap();
        let keychain = Keychain::with_store(SERVICE, store.clone());
        let entry = store.build(SERVICE, ACCOUNT, None).unwrap();
        let cred: &keyring_core::mock::Cred = entry.as_any().downcast_ref().unwrap();
        cred.set_error(Error::NoStorageAccess(Box::new(std::io::Error::other(
            "locked",
        ))));
        assert_eq!(keychain.get(ACCOUNT).unwrap_err(), KeychainError::Denied);
    }

    #[test]
    fn bad_encoding_does_not_echo_bytes() {
        let store = keyring_core::mock::Store::new().unwrap();
        let keychain = Keychain::with_store(SERVICE, store.clone());
        let entry = store.build(SERVICE, ACCOUNT, None).unwrap();
        let cred: &keyring_core::mock::Cred = entry.as_any().downcast_ref().unwrap();
        cred.set_error(Error::BadEncoding(b"ghp_secretbytes".to_vec()));
        let err = keychain.get(ACCOUNT).unwrap_err();
        assert!(!format!("{err:?} {err}").contains("secretbytes"), "{err:?}");
    }

    #[test]
    fn debug_never_shows_value() {
        let keychain = Keychain::in_memory(SERVICE);
        keychain.set(ACCOUNT, &secret("ghp_do_not_print")).unwrap();
        let got = keychain.get(ACCOUNT).unwrap();
        let text = format!("{keychain:?} {got:?}");
        assert!(!text.contains("ghp_do_not_print"), "{text}");
    }

    /// 실제 OS 키체인 왕복. CI에는 Secret Service가 없어 사람이 돌린다:
    /// `cargo test -p platform -- --ignored os_keychain_round_trip`
    #[test]
    #[ignore = "실제 OS 키체인이 필요하다"]
    fn os_keychain_round_trip() {
        let keychain = Keychain::new("dev.higit.test-only");
        keychain.set("round-trip", &secret("value-1")).unwrap();
        let got = keychain.get("round-trip").unwrap().unwrap();
        assert_eq!(got.expose_secret(), "value-1");
        keychain.delete("round-trip").unwrap();
        assert!(keychain.get("round-trip").unwrap().is_none());
    }
}
