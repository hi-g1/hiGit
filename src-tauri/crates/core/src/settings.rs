//! 앱 설정 `settings.json`(D-127). 앱 데이터에만 있어 받은 저장소가 바꿀 수 없다.
//!
//! 읽기는 링크가 아니고 4KB 이하·알 수 없는 칸 없음·`v == 1`일 때만, 아니면 기본값. 쓰기는
//! 임시 파일 뒤 바꿔 넣기(`platform::write_private_file`). 쓰지 못하면 이번 실행 동안만 메모리.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};

use crate::view::{AppSettingsPatch, AppSettingsView};

const FILE: &str = "settings.json";
const MAX_BYTES: u64 = 4096;
const VERSION: u32 = 1;

/// 실행마다 `ExecCtx`에 주는 설정 복사본.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppSettings {
    /// "올린 버전도 바꾸기"(D-127). 켜면 올린 버전의 기록도 고치고 덮어쓰며 올릴 수 있다.
    pub allow_published_rewrite: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFile {
    v: u32,
    allow_published_rewrite: bool,
}

pub(crate) struct SettingsStore {
    path: Option<PathBuf>,
    current: Mutex<AppSettings>,
}

impl SettingsStore {
    pub(crate) fn open(dir: &Path) -> Self {
        let path = dir.join(FILE);
        let current = read(&path).unwrap_or_default();
        Self {
            path: Some(path),
            current: Mutex::new(current),
        }
    }

    pub(crate) fn in_memory() -> Self {
        Self {
            path: None,
            current: Mutex::new(AppSettings::default()),
        }
    }

    pub(crate) fn get(&self) -> AppSettings {
        *self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 알려진 칸만 바꾼다. 값이 같으면 쓰지 않는다. 파일을 못 써도 이번 실행 동안은 바뀐 값을 쓴다.
    pub(crate) fn set(&self, patch: &AppSettingsPatch) -> AppSettings {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        let mut next = *current;
        if let Some(on) = patch.allow_published_rewrite {
            next.allow_published_rewrite = on;
        }
        if next != *current {
            *current = next;
            if let Some(path) = &self.path {
                let file = SettingsFile {
                    v: VERSION,
                    allow_published_rewrite: next.allow_published_rewrite,
                };
                if let Ok(bytes) = serde_json::to_vec(&file) {
                    let _ = platform::write_private_file(path, &bytes);
                }
            }
        }
        next
    }
}

fn read(path: &Path) -> Option<AppSettings> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let file: SettingsFile = serde_json::from_slice(&bytes).ok()?;
    (file.v == VERSION).then_some(AppSettings {
        allow_published_rewrite: file.allow_published_rewrite,
    })
}

impl From<AppSettings> for AppSettingsView {
    fn from(s: AppSettings) -> Self {
        Self {
            allow_published_rewrite: s.allow_published_rewrite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn_on() -> AppSettingsPatch {
        AppSettingsPatch {
            allow_published_rewrite: Some(true),
        }
    }

    #[test]
    fn defaults_off_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::open(dir.path());
        assert!(!store.get().allow_published_rewrite);
        assert!(store.set(&turn_on()).allow_published_rewrite);
        let again = SettingsStore::open(dir.path());
        assert!(again.get().allow_published_rewrite);
        let text = std::fs::read_to_string(dir.path().join(FILE)).unwrap();
        assert_eq!(text, r#"{"v":1,"allow_published_rewrite":true}"#);
        // 빈 패치는 아무것도 바꾸지 않는다.
        assert!(
            again
                .set(&AppSettingsPatch {
                    allow_published_rewrite: None
                })
                .allow_published_rewrite
        );
    }

    #[test]
    fn odd_files_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE);
        let big = format!(
            r#"{{"v":1,"allow_published_rewrite":true,"x":"{}"}}"#,
            "a".repeat(5000)
        );
        for bad in [
            r#"{"v":2,"allow_published_rewrite":true}"#,
            r#"{"v":1,"allow_published_rewrite":true,"extra":1}"#,
            r#"{"v":1}"#,
            "not json",
            big.as_str(),
        ] {
            std::fs::write(&path, bad).unwrap();
            assert!(
                !SettingsStore::open(dir.path())
                    .get()
                    .allow_published_rewrite,
                "{bad}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn link_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real.json");
        std::fs::write(&real, r#"{"v":1,"allow_published_rewrite":true}"#).unwrap();
        std::os::unix::fs::symlink(&real, dir.path().join(FILE)).unwrap();
        assert!(
            !SettingsStore::open(dir.path())
                .get()
                .allow_published_rewrite
        );
    }

    #[test]
    fn write_failure_keeps_value_in_memory() {
        let dir = tempfile::tempdir().unwrap();
        // 폴더가 없어 쓸 수 없는 경로.
        let store = SettingsStore::open(&dir.path().join("없음"));
        assert!(store.set(&turn_on()).allow_published_rewrite);
        assert!(store.get().allow_published_rewrite);
        let memory = SettingsStore::in_memory();
        assert!(memory.set(&turn_on()).allow_published_rewrite);
    }
}
