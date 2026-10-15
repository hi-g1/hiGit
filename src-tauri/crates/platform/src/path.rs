use std::io;
use std::path::{Path, PathBuf};

/// 사용자가 고른 폴더 경로를 절대 경로로 정규화한다.
///
/// Windows에서는 `\\?\C:\…` 형식 대신 `C:\…`를 돌려준다(git과 화면이 다루기 쉬운 형식).
/// 존재하지 않는 경로는 오류다.
pub fn normalize_repo_path(path: &Path) -> io::Result<PathBuf> {
    dunce::canonicalize(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_korean_and_spaces() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("한글 폴더");
        std::fs::create_dir(&dir).unwrap();

        let normalized = normalize_repo_path(&dir).unwrap();
        assert!(normalized.is_absolute());
        assert!(normalized.ends_with("한글 폴더"), "{normalized:?}");
        assert!(normalized.is_dir());
    }

    #[test]
    fn resolves_relative_parts() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("a");
        std::fs::create_dir(&dir).unwrap();

        let normalized = normalize_repo_path(&dir.join("..").join("a")).unwrap();
        assert_eq!(normalized, normalize_repo_path(&dir).unwrap());
    }

    #[test]
    fn has_no_verbatim_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        let normalized = normalize_repo_path(tmp.path()).unwrap();
        assert!(
            !normalized.to_string_lossy().starts_with(r"\\?\"),
            "{normalized:?}"
        );
    }

    #[test]
    fn missing_path_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(normalize_repo_path(&tmp.path().join("없는 폴더")).is_err());
    }
}
