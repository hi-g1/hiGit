//! 비밀이 아닌 연결 정보 파일 `github.json`(D-109). 토큰은 여기 두지 않는다(키체인에만).
//!
//! 이 파일이 있으면 "키체인에 토큰을 저장한 연결"이다. 화면 상태는 키체인을 건드리지 않고 이 파일로
//! 그린다.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};

const FILE: &str = "github.json";
const MAX_BYTES: u64 = 4096;
const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StatusFile {
    pub v: u32,
    pub login: String,
    pub id: u64,
    /// 받은 권한에 `workflow`가 있다(D-135). 옛 파일에는 없어 거짓으로 읽고, 거짓이면 쓰지 않는다(`v`는 1 그대로).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub workflow: bool,
}

impl StatusFile {
    pub(crate) fn new(login: &str, id: u64, workflow: bool) -> Self {
        Self {
            v: VERSION,
            login: login.to_owned(),
            id,
            workflow,
        }
    }
}

/// 앱 데이터 폴더의 `github.json`, 또는 메모리(시험, 앱 데이터 폴더를 못 쓸 때).
pub(crate) struct StatusStore {
    path: Option<PathBuf>,
    memory: Mutex<Option<StatusFile>>,
}

impl StatusStore {
    pub(crate) fn open(dir: &Path) -> Self {
        Self {
            path: Some(dir.join(FILE)),
            memory: Mutex::new(None),
        }
    }

    pub(crate) fn in_memory() -> Self {
        Self {
            path: None,
            memory: Mutex::new(None),
        }
    }

    /// 모양이 맞는 파일만 읽는다. 없거나 틀리면 `None`.
    pub(crate) fn read(&self) -> Option<StatusFile> {
        let file = match &self.path {
            None => self
                .memory
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()?,
            Some(path) => {
                let meta = std::fs::symlink_metadata(path).ok()?;
                if !meta.is_file() || meta.len() > MAX_BYTES {
                    return None;
                }
                let bytes = std::fs::read(path).ok()?;
                serde_json::from_slice::<StatusFile>(&bytes).ok()?
            }
        };
        (file.v == VERSION && hosting::valid_owner(&file.login)).then_some(file)
    }

    pub(crate) fn write(&self, file: &StatusFile) -> std::io::Result<()> {
        match &self.path {
            None => {
                *self.memory.lock().unwrap_or_else(PoisonError::into_inner) = Some(file.clone());
                Ok(())
            }
            Some(path) => {
                let bytes = serde_json::to_vec(file).map_err(std::io::Error::other)?;
                platform::write_private_file(path, &bytes)
            }
        }
    }

    /// 없어도 성공.
    pub(crate) fn remove(&self) -> std::io::Result<()> {
        match &self.path {
            None => {
                *self.memory.lock().unwrap_or_else(PoisonError::into_inner) = None;
                Ok(())
            }
            Some(path) => match std::fs::remove_file(path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_rejects_odd_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = StatusStore::open(dir.path());
        assert_eq!(store.read(), None);
        store.write(&StatusFile::new("octocat", 1, false)).unwrap();
        assert_eq!(store.read(), Some(StatusFile::new("octocat", 1, false)));
        let text = std::fs::read_to_string(dir.path().join(FILE)).unwrap();
        assert_eq!(text, r#"{"v":1,"login":"octocat","id":1}"#);

        for bad in [
            r#"{"v":2,"login":"octocat","id":1}"#,
            r#"{"v":1,"login":"../x","id":1}"#,
            r#"{"v":1,"login":"octocat"}"#,
            "not json",
        ] {
            std::fs::write(dir.path().join(FILE), bad).unwrap();
            assert_eq!(store.read(), None, "{bad}");
        }
        store.remove().unwrap();
        store.remove().unwrap();
        assert!(!dir.path().join(FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = StatusStore::open(dir.path());
        store.write(&StatusFile::new("octocat", 1, false)).unwrap();
        let mode = std::fs::metadata(dir.path().join(FILE))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
