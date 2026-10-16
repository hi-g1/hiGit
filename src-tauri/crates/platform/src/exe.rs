//! 지금 돌고 있는 앱 실행 파일의 경로. git credential helper가 이 파일을 다시 실행한다(D-111).

use std::path::{Path, PathBuf};

/// 앱 실행 파일의 절대 경로. Windows는 `\\?\` 없이 `C:\…`로 준다.
///
/// 경로에 NUL이나 줄바꿈이 있으면 `None`이다(git 설정·환경 변수로 넘길 수 없다).
pub fn current_exe_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    normalize_exe(&exe)
}

fn normalize_exe(exe: &Path) -> Option<PathBuf> {
    let path = dunce::canonicalize(exe).unwrap_or_else(|_| dunce::simplified(exe).to_path_buf());
    if !path.is_absolute() {
        return None;
    }
    let text = path.to_string_lossy();
    if text.contains(['\0', '\n', '\r']) {
        return None;
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_exe_is_absolute_existing_file() {
        let path = current_exe_path().unwrap();
        assert!(path.is_absolute(), "{path:?}");
        assert!(path.is_file(), "{path:?}");
        assert!(!path.to_string_lossy().starts_with(r"\\?\"), "{path:?}");
    }

    #[test]
    fn resolves_relative_parts_and_keeps_korean() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("한글 폴더");
        std::fs::create_dir(&dir).unwrap();
        let file = dir.join("app.exe");
        std::fs::write(&file, b"").unwrap();

        let got = normalize_exe(&dir.join("..").join("한글 폴더").join("app.exe")).unwrap();
        assert_eq!(got, dunce::canonicalize(&file).unwrap());
    }

    #[test]
    fn rejects_line_breaks_and_relative_paths() {
        assert!(normalize_exe(Path::new("relative/app")).is_none());
        let base = std::env::temp_dir();
        assert!(normalize_exe(&base.join("bad\nname")).is_none());
        assert!(normalize_exe(&base.join("bad\rname")).is_none());
    }
}
