//! 화면이 보낸 경로 검사. git에 넘기기 전에 저장소 밖을 가리키는 경로를 막는다.

use crate::{CoreError, Result};

/// 경로가 하나 이상이고, 모두 저장소 안의 상대 경로(구분자 `/`)인지 확인한다.
/// OS와 상관없이 같은 규칙: 절대 경로, 드라이브(`C:`), `\`, `..`, 빈 경로를 거부한다.
pub(crate) fn validate(paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Err(CoreError::InvalidInput("파일을 고르지 않았어요"));
    }
    paths.iter().try_for_each(|p| validate_one(p))
}

/// 경로 하나 검사(비교 보기처럼 경로 하나만 받는 곳에서 쓴다).
pub(crate) fn validate_one(path: &str) -> Result<()> {
    let bytes = path.as_bytes();
    // Windows 드라이브(`C:`)만 거른다. Linux에서는 `a:b.txt`도 쓸 수 있는 이름이다.
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let bad = path.is_empty()
        || path.contains('\0')
        || path.contains('\\')
        || path.starts_with('/')
        || drive
        || path.split('/').any(|part| part == ".." || part.is_empty());
    if bad {
        return Err(CoreError::InvalidInput("저장소 밖이거나 잘못된 경로예요"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(paths: &[&str]) -> bool {
        validate(&paths.iter().map(|p| p.to_string()).collect::<Vec<_>>()).is_ok()
    }

    #[test]
    fn accepts_repo_relative_paths() {
        assert!(ok(&[
            "a.txt",
            "폴더/한글 파일.txt",
            "별*표[1].md",
            ".gitignore",
            "a..b.txt"
        ]));
        // 둘째 글자가 `:`이어도 드라이브가 아니면 받는다(Linux 파일 이름).
        assert!(ok(&["가:나.txt", "1:2.txt"]));
    }

    #[test]
    fn rejects_paths_outside_repo() {
        for bad in [
            "",
            "/etc/passwd",
            "../밖.txt",
            "폴더/../../밖",
            "C:/Windows/x",
            "c:\\x",
            "\\\\server\\share",
            "폴더\\파일",
            "a//b",
            "폴더/",
        ] {
            assert!(!ok(&[bad]), "{bad:?}");
        }
        assert!(!ok(&[]));
    }
}
