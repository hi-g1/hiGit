//! 화면이 넘긴 경로를 받아도 되는지(D-037, 계획 004 10단계).
//!
//! 웹뷰에 스크립트가 끼어들어도 연 적 없는 폴더에서 git이 돌지 않게 한다. 프로젝트 열기
//! (`Core::open_project`)를 거친 경로만 조회·액션·감시가 받는다. 시작 화면 상태 요약은
//! 최근 목록에 있는 경로도 받는다.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::{CoreError, Result};

/// 화면이 넘기는 경로 문자열의 최대 길이(바이트).
pub(crate) const MAX_PATH_BYTES: usize = 4096;

#[derive(Default)]
pub(crate) struct OpenedRepos(Mutex<Vec<PathBuf>>);

impl OpenedRepos {
    /// 정규화한 경로를 기록한다. 같은 경로(Windows는 대소문자 무시)는 한 번만.
    pub fn insert(&self, path: &Path) {
        let mut list = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if !list.iter().any(|p| platform::same_path(p, path)) {
            list.push(path.to_path_buf());
        }
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .any(|p| platform::same_path(p, path))
    }

    /// 연 적이 없으면 `NotOpened`.
    pub fn require(&self, path: &Path) -> Result<()> {
        if self.contains(path) {
            Ok(())
        } else {
            Err(CoreError::NotOpened)
        }
    }
}

/// 화면이 넘긴 경로가 너무 길면 git을 부르기 전에 막는다.
pub(crate) fn check_path_len(path: &Path) -> Result<()> {
    if path.as_os_str().len() > MAX_PATH_BYTES {
        return Err(CoreError::InvalidInput("경로가 너무 길어요"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_opened_paths() {
        let dir = tempfile::tempdir().unwrap();
        let opened = OpenedRepos::default();
        assert!(opened.require(dir.path()).is_err());
        opened.insert(dir.path());
        opened.insert(dir.path());
        assert!(opened.require(dir.path()).is_ok());
        assert_eq!(opened.0.lock().unwrap().len(), 1);
    }

    #[test]
    fn long_paths_are_rejected() {
        let long = PathBuf::from("a".repeat(MAX_PATH_BYTES + 1));
        assert!(check_path_len(&long).is_err());
        assert!(check_path_len(Path::new("/tmp/x")).is_ok());
    }
}
