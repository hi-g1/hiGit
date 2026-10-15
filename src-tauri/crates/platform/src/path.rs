use std::io;
use std::path::{Path, PathBuf};

/// 사용자가 고른 폴더 경로를 절대 경로로 정규화한다.
///
/// Windows에서는 `\\?\C:\…` 형식 대신 `C:\…`를 돌려준다(git과 화면이 다루기 쉬운 형식).
/// 존재하지 않는 경로는 오류다.
pub fn normalize_repo_path(path: &Path) -> io::Result<PathBuf> {
    dunce::canonicalize(path)
}

/// 정규화된 두 경로가 같은 곳인지. Windows는 대소문자를 구분하지 않는다.
pub fn same_path(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

/// git 설정 값으로 쓸 경로 글자. Windows는 `C:/Users/…`처럼 `/`로 쓴다(safe.directory 등).
pub fn git_path_string(path: &Path) -> String {
    let text = path.to_string_lossy().into_owned();
    if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text
    }
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
    fn same_path_follows_os_rules() {
        assert!(same_path(Path::new("/a/b"), Path::new("/a/b")));
        #[cfg(windows)]
        assert!(same_path(Path::new(r"C:\A\b"), Path::new(r"c:\a\B")));
        #[cfg(not(windows))]
        assert!(!same_path(Path::new("/a/B"), Path::new("/a/b")));
    }

    #[test]
    fn missing_path_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(normalize_repo_path(&tmp.path().join("없는 폴더")).is_err());
    }
}
