//! 브랜치별로 맡겨 둔 고친 파일의 ref 이름(D-047, D-064).
//!
//! 브랜치 이름의 `%`와 `/`를 바꿔 적어 한 조각으로 만든다. 그래야 `a`와 `a/b`를 함께 맡길 수 있다.

use crate::marker;
use crate::undo::JournalKey;

pub(crate) const PARKED_PREFIX: &str = "refs/higit/parked/";
/// 맡긴 ref 이름 한 조각의 한도(바이트). 파일 시스템 이름 한도 255바이트 안쪽.
pub(crate) const MAX_NAME_BYTES: usize = 200;

/// 브랜치를 갈아타다 끊겨 stash 목록에 남은 맡긴 항목(D-082). 최근 것부터.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Leftover {
    pub oid: git::Oid,
    pub branch: String,
}

/// stash 목록에서 메시지가 `higit parked <브랜치>`인 항목. 메시지만 믿지 않고 브랜치 이름과
/// 든 파일 경로를 검사한다(받은 폴더에 가짜 항목이 있을 수 있다). 검사를 통과하지 못한 항목은
/// 여기 들지 않고 임시 보관 목록에 그대로 보인다.
pub(crate) fn leftovers(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
) -> crate::Result<Vec<Leftover>> {
    use crate::actions::stash::{PARKED_PREFIX as MESSAGE, stash_message};
    let mut out = Vec::new();
    for e in git.stash_list(repo)? {
        let Some(branch) = stash_message(&e.subject)
            .strip_prefix(MESSAGE)
            .and_then(|b| b.strip_prefix(' '))
        else {
            continue;
        };
        let name_ok = crate::actions::names::validate_ref_like(branch).is_ok()
            && ref_name(branch).len() - PARKED_PREFIX.len() <= MAX_NAME_BYTES;
        // 든 파일을 읽지 못하는 항목(커밋이 아님, git이 거부한 경로)도 건너뛴다. 항목 하나가
        // 조회 전체를 막지 않게 한다.
        let paths_ok = git
            .parked_paths(repo, &e.oid)
            .is_ok_and(|paths| safe_paths(&paths));
        if !name_ok || !paths_ok {
            continue;
        }
        out.push(Leftover {
            oid: e.oid,
            branch: branch.to_string(),
        });
    }
    Ok(out)
}

/// `feature/login` → `refs/higit/parked/feature%2Flogin`.
pub(crate) fn ref_name(branch: &str) -> String {
    let encoded = branch.replace('%', "%25").replace('/', "%2F");
    format!("{PARKED_PREFIX}{encoded}")
}

/// 지운 브랜치의 맡긴 ref를 백업 ref(`parked-<브랜치>`)로 옮기고 지운다(D-103). 없으면 할 일 없음.
pub(crate) fn retire_to_backup(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    branch: &str,
) -> crate::Result<()> {
    let name = ref_name(branch);
    if let Some(oid) = git.resolve_ref(repo, &name)? {
        crate::undo::backup::backup_oid(git, repo, &oid, &format!("parked-{branch}"))?;
        git.delete_ref(repo, &name, &git::RefExpect::Is(oid))?;
    }
    Ok(())
}

/// 이름을 바꾼 브랜치의 맡긴 ref를 새 이름으로 옮긴다(D-054, D-103). 새 이름에 남은 것이 있으면
/// 먼저 백업으로 물린다.
pub(crate) fn move_ref(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    from: &str,
    to: &str,
) -> crate::Result<()> {
    let (old, new) = (ref_name(from), ref_name(to));
    let Some(oid) = git.resolve_ref(repo, &old)? else {
        return Ok(());
    };
    if old == new {
        return Ok(());
    }
    retire_to_backup(git, repo, to)?;
    git.update_ref(repo, &new, &oid, &git::RefExpect::Absent)?;
    git.delete_ref(repo, &old, &git::RefExpect::Is(oid))?;
    Ok(())
}

/// ref 이름에서 브랜치 이름을 되살린다. 맡긴 ref가 아니면 `None`.
pub(crate) fn branch_of(refname: &str) -> Option<String> {
    let encoded = refname.strip_prefix(PARKED_PREFIX)?;
    if encoded.contains('/') {
        return None;
    }
    Some(encoded.replace("%2F", "/").replace("%25", "%"))
}

/// 저장소 안의 상대 경로이고 `.git` 폴더(Windows의 다른 이름 포함)를 건드리지 않는다.
fn safe_paths(paths: &[String]) -> bool {
    crate::actions::paths::validate(paths).is_ok() && paths.iter().all(|p| git::safe_relative(p))
}

/// 앱이 시작한 "겹침 해결" 꺼내기의 표식(D-083). `<git 폴더>/higit/unpark.json`, 서명은 `marker`.
/// 이 파일이 있을 때만 충돌 해결 화면이 "맡겨 둔 파일" 맥락으로 읽는다(터미널의 `stash pop`과 구분).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct UnparkMarker {
    pub branch: String,
    pub oid: String,
}

/// 서명이 맞고(D-102) 값의 형식이 맞는 표식.
pub(crate) fn read_marker(git_dir: &std::path::Path, key: &JournalKey) -> Option<UnparkMarker> {
    let marker: UnparkMarker = marker::read(git_dir, marker::Name::Unpark, key)?;
    let valid = git::Oid::parse(&marker.oid).is_some()
        && crate::actions::names::validate_ref_like(&marker.branch).is_ok();
    valid.then_some(marker)
}

pub(crate) fn write_marker(
    git_dir: &std::path::Path,
    key: &JournalKey,
    value: &UnparkMarker,
) -> crate::Result<()> {
    marker::write(git_dir, marker::Name::Unpark, key, value)
}

pub(crate) fn clear_marker(git_dir: &std::path::Path) {
    marker::clear(git_dir, marker::Name::Unpark);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_round_trip_and_bad_values() {
        let dir = tempfile::tempdir().unwrap();
        let key = JournalKey::ephemeral();
        assert_eq!(read_marker(dir.path(), &key), None);
        let m = UnparkMarker {
            branch: "기능/한글".into(),
            oid: "a".repeat(40),
        };
        write_marker(dir.path(), &key, &m).unwrap();
        assert_eq!(read_marker(dir.path(), &key), Some(m));
        // 서명이 맞아도 값이 규칙에 어긋나면 읽지 않는다.
        let bad = UnparkMarker {
            branch: "-x".into(),
            oid: "a".repeat(40),
        };
        write_marker(dir.path(), &key, &bad).unwrap();
        assert_eq!(read_marker(dir.path(), &key), None);
        // 서명 없는 옛 형식은 읽지 않는다.
        std::fs::write(
            dir.path().join("higit/unpark.json"),
            r#"{"branch":"main","oid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        )
        .unwrap();
        assert_eq!(read_marker(dir.path(), &key), None);
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
