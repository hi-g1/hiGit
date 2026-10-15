//! 저장소에 이미 있는 이름(브랜치, 원격, upstream) 검사. git에 넘기기 전에 막는다(D-035).
//!
//! git 버전과 명령에 따라 `--end-of-options`를 쓸 수 없는 곳이 있어(checkout, reset),
//! `-`로 시작하는 이름이 옵션으로 읽히는 길은 여기서 닫는다. 새 브랜치 이름은
//! `GitBackend::check_branch_name`이 따로 검사한다.

use crate::{CoreError, Result};

/// 받은 저장소에 `refs/heads/-x` 같은 ref가 있으면 목록에는 보이되 다룰 때 이 오류가 난다.
pub(crate) const UNSUPPORTED_NAME: &str = "이 이름의 브랜치는 hiGit에서 다룰 수 없어요";

// 브랜치 갈아타기·동기화 단계에서 쓴다.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn validate_ref_like(name: &str) -> Result<()> {
    let bad = name.is_empty()
        || name.len() > 255
        || name.starts_with('-')
        || name.chars().any(char::is_control);
    if bad {
        return Err(CoreError::InvalidInput(UNSUPPORTED_NAME));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_option_like_and_control() {
        for name in [
            "",
            "-x",
            "--upload-pack=x",
            "-c",
            "a\nb",
            "a\0b",
            &"a".repeat(256),
        ] {
            assert!(validate_ref_like(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn accepts_normal_names() {
        for name in ["main", "기능/한글", "a-b", "origin", "release/1.0"] {
            assert!(validate_ref_like(name).is_ok(), "{name:?}");
        }
    }
}
