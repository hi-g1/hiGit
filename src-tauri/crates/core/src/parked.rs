//! 브랜치별로 맡겨 둔 고친 파일의 ref 이름(D-047, D-064).
//!
//! 브랜치 이름의 `%`와 `/`를 바꿔 적어 한 조각으로 만든다. 그래야 `a`와 `a/b`를 함께 맡길 수 있다.

pub(crate) const PARKED_PREFIX: &str = "refs/higit/parked/";

/// `feature/login` → `refs/higit/parked/feature%2Flogin`.
// 맡겨 두는 갈아타기(계획 005 H9)에서 쓴다.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ref_name(branch: &str) -> String {
    let encoded = branch.replace('%', "%25").replace('/', "%2F");
    format!("{PARKED_PREFIX}{encoded}")
}

/// ref 이름에서 브랜치 이름을 되살린다. 맡긴 ref가 아니면 `None`.
pub(crate) fn branch_of(refname: &str) -> Option<String> {
    let encoded = refname.strip_prefix(PARKED_PREFIX)?;
    if encoded.contains('/') {
        return None;
    }
    Some(encoded.replace("%2F", "/").replace("%25", "%"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for name in ["main", "feature/login", "기능/한글", "100%/done", "a%2Fb"] {
            let r = ref_name(name);
            assert_eq!(r.matches('/').count(), 3, "{r}");
            assert_eq!(branch_of(&r).as_deref(), Some(name));
        }
        assert_ne!(ref_name("a/b"), ref_name("a%2Fb"));
        assert_eq!(branch_of("refs/heads/main"), None);
        assert_eq!(branch_of("refs/higit/parked/a/b"), None);
    }
}
