//! ref 읽기·쓰기와 버전 사이 관계(되돌리기, 맡겨 두기용).

use crate::{GitError, Oid, Result};

/// `update-ref`에 넘기는 "지금 값이 이래야 한다" 조건. 그사이 바뀌었으면 git이 거부한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefExpect {
    /// 지금 값을 따지지 않는다.
    Any,
    /// 아직 없어야 한다.
    Absent,
    /// 이 값이어야 한다.
    Is(Oid),
}

impl RefExpect {
    /// `update-ref`의 이전 값 인자. `Absent`는 새 값과 같은 길이의 영(0) 해시다.
    pub(crate) fn arg(&self, new_len: usize) -> Option<String> {
        match self {
            RefExpect::Any => None,
            RefExpect::Absent => Some("0".repeat(new_len)),
            RefExpect::Is(oid) => Some(oid.as_str().to_string()),
        }
    }
}

/// core가 만든 ref 이름만 받는다. `refs/`로 시작하고 옵션처럼 보이지 않아야 한다.
pub(crate) fn check_ref_name(name: &str) -> Result<()> {
    let ok = name.starts_with("refs/")
        && !name.contains("..")
        && !name.chars().any(|c| c.is_control() || c == ' ');
    if ok {
        Ok(())
    } else {
        Err(GitError::Parse(format!("쓸 수 없는 ref 이름: {name}")))
    }
}

/// `rev-list --left-right --count` 출력 `왼쪽\t오른쪽`.
pub(crate) fn parse_counts(out: &str) -> Result<(u32, u32)> {
    let mut parts = out.trim().split('\t').map(str::parse::<u32>);
    match (parts.next(), parts.next(), parts.next()) {
        (Some(Ok(left)), Some(Ok(right)), None) => Ok((left, right)),
        _ => Err(GitError::Parse(format!("버전 수: {out:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_args() {
        assert_eq!(RefExpect::Any.arg(40), None);
        assert_eq!(RefExpect::Absent.arg(40), Some("0".repeat(40)));
        let oid = Oid::parse(&"a".repeat(40)).unwrap();
        assert_eq!(RefExpect::Is(oid).arg(40), Some("a".repeat(40)));
    }

    #[test]
    fn ref_names() {
        assert!(check_ref_name("refs/higit/backup/1/main").is_ok());
        for bad in ["HEAD", "-x", "refs/a..b", "refs/a b", "main"] {
            assert!(check_ref_name(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn counts() {
        assert_eq!(parse_counts("3\t1\n").unwrap(), (3, 1));
        assert!(parse_counts("3").is_err());
        assert!(parse_counts("a\tb").is_err());
    }
}
