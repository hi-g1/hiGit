//! 브랜치별로 맡겨 둔 고친 파일의 ref 이름(D-047, D-064).
//!
//! 브랜치 이름의 `%`와 `/`를 바꿔 적어 한 조각으로 만든다. 그래야 `a`와 `a/b`를 함께 맡길 수 있다.

pub(crate) const PARKED_PREFIX: &str = "refs/higit/parked/";

/// `feature/login` → `refs/higit/parked/feature%2Flogin`.
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

/// 앱이 시작한 "겹침 해결" 꺼내기의 표식(D-083). `<git 폴더>/higit/unpark.json`.
/// 이 파일이 있을 때만 충돌 해결 화면이 "맡겨 둔 파일" 맥락으로 읽는다(터미널의 `stash pop`과 구분).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct UnparkMarker {
    pub branch: String,
    pub oid: String,
}

fn marker_path(git_dir: &std::path::Path) -> std::path::PathBuf {
    git_dir.join("higit").join("unpark.json")
}

/// `higit` 폴더와 표식 파일이 링크가 아닐 때만 쓰고 읽는다(journal과 같은 까닭).
fn place_ok(git_dir: &std::path::Path) -> bool {
    let dir = git_dir.join("higit");
    let not_link = |p: &std::path::Path, want_dir: bool| match std::fs::symlink_metadata(p) {
        Ok(m) => !m.file_type().is_symlink() && m.is_dir() == want_dir,
        Err(_) => true,
    };
    not_link(&dir, true) && not_link(&marker_path(git_dir), false)
}

pub(crate) fn write_marker(git_dir: &std::path::Path, marker: &UnparkMarker) -> crate::Result<()> {
    let fail = |_| crate::CoreError::InvalidInput("겹침 해결을 시작하지 못했어요");
    if !place_ok(git_dir) {
        return Err(crate::CoreError::InvalidInput(
            "겹침 해결을 시작하지 못했어요",
        ));
    }
    std::fs::create_dir_all(git_dir.join("higit")).map_err(fail)?;
    if !place_ok(git_dir) {
        return Err(crate::CoreError::InvalidInput(
            "겹침 해결을 시작하지 못했어요",
        ));
    }
    let text = serde_json::to_string(marker)
        .map_err(|_| crate::CoreError::InvalidInput("겹침 해결을 시작하지 못했어요"))?;
    std::fs::write(marker_path(git_dir), text).map_err(fail)
}

pub(crate) fn read_marker(git_dir: &std::path::Path) -> Option<UnparkMarker> {
    if !place_ok(git_dir) {
        return None;
    }
    let text = std::fs::read_to_string(marker_path(git_dir)).ok()?;
    let marker: UnparkMarker = serde_json::from_str(&text).ok()?;
    let valid = git::Oid::parse(&marker.oid).is_some()
        && crate::actions::names::validate_ref_like(&marker.branch).is_ok();
    valid.then_some(marker)
}

pub(crate) fn clear_marker(git_dir: &std::path::Path) {
    if place_ok(git_dir) {
        let _ = std::fs::remove_file(marker_path(git_dir));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_round_trip_and_bad_values() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_marker(dir.path()), None);
        let m = UnparkMarker {
            branch: "기능/한글".into(),
            oid: "a".repeat(40),
        };
        write_marker(dir.path(), &m).unwrap();
        assert_eq!(read_marker(dir.path()), Some(m));
        std::fs::write(
            dir.path().join("higit/unpark.json"),
            r#"{"branch":"-x","oid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        )
        .unwrap();
        assert_eq!(read_marker(dir.path()), None);
        clear_marker(dir.path());
        assert!(!dir.path().join("higit/unpark.json").exists());
    }

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
