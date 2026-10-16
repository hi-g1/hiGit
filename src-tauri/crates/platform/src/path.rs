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

/// 경로 구성요소 하나를 그대로 파일 이름으로 써도 되는지(D-216). Windows는 장치 이름(`CON`, `nul.txt`, `COM1` 등),
/// 대체 데이터 스트림·드라이브의 `:`, 끝의 점·공백을 거절한다. 다른 OS는 NUL만 거절한다.
pub fn is_plain_component(name: &str) -> bool {
    if name.is_empty() || name.contains('\0') {
        return false;
    }
    if cfg!(windows) {
        if name.contains(':') || name.ends_with(['.', ' ']) {
            return false;
        }
        let stem = name
            .split('.')
            .next()
            .unwrap_or(name)
            .trim_end()
            .to_ascii_uppercase();
        let device = matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.chars().count() == 4
            && stem[3..]
                .chars()
                .all(|c| c.is_ascii_digit() || "¹²³".contains(c)));
        if device {
            return false;
        }
    }
    true
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
    fn plain_components() {
        assert!(is_plain_component("a.txt"));
        assert!(is_plain_component("한글 파일.md"));
        assert!(!is_plain_component(""));
        assert!(!is_plain_component("a\0b"));
        if cfg!(windows) {
            for bad in [
                "CON",
                "con.txt",
                "Nul",
                "COM1",
                "lpt9.log",
                "a.txt:stream",
                "C:",
                "끝점.",
                "끝공백 ",
            ] {
                assert!(!is_plain_component(bad), "{bad}");
            }
            assert!(is_plain_component("console.txt"));
            assert!(is_plain_component("COM10"));
        } else {
            assert!(is_plain_component("CON"));
            assert!(is_plain_component("a:b"));
        }
    }

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
