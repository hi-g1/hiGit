//! 중간에 끊긴 git 작업이 남긴 잠금 파일(D-041, D-081).
//!
//! 지우는 대상은 git 폴더 최상위와 `refs/` 아래의 일반 파일 `*.lock`뿐이다. 링크는 따라가지 않는다.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// 수정한 지 이만큼 지나야 버려진 잠금으로 본다.
pub(crate) const STALE_AFTER: Duration = Duration::from_secs(60);
/// `refs/` 아래를 이 깊이까지, 이 개수까지 본다.
const MAX_DEPTH: usize = 6;
const MAX_LOCKS: usize = 500;

/// 잠금 파일 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LockFile {
    pub path: PathBuf,
    /// git 폴더 기준 이름(`index.lock`, `refs/heads/main.lock`).
    pub name: String,
}

/// git 폴더(작업 트리별)와 공용 폴더의 최상위, 공용 폴더 `refs/` 아래의 `*.lock`.
pub(crate) fn find(git_dir: &Path, common_dir: &Path) -> Vec<LockFile> {
    let mut out = Vec::new();
    top_level(git_dir, &mut out);
    if !platform::same_path(git_dir, common_dir) {
        top_level(common_dir, &mut out);
    }
    walk(&common_dir.join("refs"), "refs", 1, &mut out);
    out
}

/// 버려진 잠금. 모두 수정한 지 60초가 지났고 시스템에 도는 git이 없을 때만 돌려준다. 하나라도
/// 새것이면 무언가 작업 중이므로 빈 목록이다. 앱의 작업이 도는지는 부르는 쪽이 본다.
pub(crate) fn stale(
    git_dir: &Path,
    common_dir: &Path,
    now: SystemTime,
    git_running: impl FnOnce() -> bool,
) -> Vec<LockFile> {
    let found = find(git_dir, common_dir);
    if found.is_empty() {
        return found;
    }
    let all_old = found.iter().all(|l| {
        std::fs::symlink_metadata(&l.path)
            .and_then(|m| m.modified())
            .is_ok_and(|m| now.duration_since(m).is_ok_and(|age| age >= STALE_AFTER))
    });
    if !all_old || git_running() {
        return Vec::new();
    }
    found
}

fn is_lock_file(entry: &std::fs::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|n| n.ends_with(".lock"))
        && entry.file_type().is_ok_and(|t| t.is_file())
}

fn top_level(dir: &Path, out: &mut Vec<LockFile>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_LOCKS {
            return;
        }
        if is_lock_file(&entry) {
            out.push(LockFile {
                path: entry.path(),
                name: entry.file_name().to_string_lossy().into_owned(),
            });
        }
    }
}

fn walk(dir: &Path, name: &str, depth: usize, out: &mut Vec<LockFile>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_LOCKS {
            return;
        }
        let child = format!("{name}/{}", entry.file_name().to_string_lossy());
        if is_lock_file(&entry) {
            out.push(LockFile {
                path: entry.path(),
                name: child,
            });
        } else if depth < MAX_DEPTH && entry.file_type().is_ok_and(|t| t.is_dir()) {
            // DirEntry::file_type은 링크를 따라가지 않는다(링크 폴더는 is_dir이 아니다).
            walk(&entry.path(), &child, depth + 1, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("refs/heads/feature")).unwrap();
        dir
    }

    fn names(locks: &[LockFile]) -> Vec<&str> {
        let mut n: Vec<&str> = locks.iter().map(|l| l.name.as_str()).collect();
        n.sort_unstable();
        n
    }

    fn later() -> SystemTime {
        SystemTime::now() + STALE_AFTER + Duration::from_secs(5)
    }

    #[test]
    fn finds_top_level_and_ref_locks_only() {
        let dir = git_dir();
        let d = dir.path();
        for f in [
            "index.lock",
            "HEAD.lock",
            "refs/heads/feature/x.lock",
            "index",
            "lockfile",
        ] {
            std::fs::write(d.join(f), "").unwrap();
        }
        // 이름만 .lock인 폴더, refs 밖 하위 폴더의 잠금은 보지 않는다.
        std::fs::create_dir_all(d.join("dir.lock")).unwrap();
        std::fs::create_dir_all(d.join("objects")).unwrap();
        std::fs::write(d.join("objects/x.lock"), "").unwrap();
        assert_eq!(
            names(&find(d, d)),
            vec!["HEAD.lock", "index.lock", "refs/heads/feature/x.lock"]
        );
    }

    #[test]
    fn links_are_not_followed() {
        let dir = git_dir();
        let d = dir.path();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("victim.lock"), "").unwrap();
        std::fs::create_dir_all(outside.path().join("deep")).unwrap();
        std::fs::write(outside.path().join("deep/y.lock"), "").unwrap();
        if platform::symlink_file(&outside.path().join("victim.lock"), &d.join("index.lock"))
            .is_err()
            || platform::symlink_dir(&outside.path().join("deep"), &d.join("refs/heads/deep"))
                .is_err()
        {
            // Windows에서 링크를 만들 권한이 없으면 건너뛴다.
            return;
        }
        assert!(find(d, d).is_empty());
    }

    #[test]
    fn stale_only_when_old_and_no_git_runs() {
        let dir = git_dir();
        let d = dir.path();
        std::fs::write(d.join("index.lock"), "").unwrap();
        // 방금 만든 것은 아니다.
        assert!(stale(d, d, SystemTime::now(), || false).is_empty());
        // git이 돌면 아니다.
        assert!(stale(d, d, later(), || true).is_empty());
        assert_eq!(names(&stale(d, d, later(), || false)), vec!["index.lock"]);
    }

    #[test]
    fn a_fresh_lock_among_old_ones_means_work_is_running() {
        let dir = git_dir();
        let d = dir.path();
        std::fs::write(d.join("index.lock"), "").unwrap();
        let old = SystemTime::now() - STALE_AFTER - Duration::from_secs(5);
        std::fs::File::options()
            .write(true)
            .open(d.join("index.lock"))
            .unwrap()
            .set_modified(old)
            .unwrap();
        assert_eq!(
            names(&stale(d, d, SystemTime::now(), || false)),
            vec!["index.lock"]
        );
        std::fs::write(d.join("refs/heads/feature/x.lock"), "").unwrap();
        assert!(stale(d, d, SystemTime::now(), || false).is_empty());
    }

    #[test]
    fn stops_at_the_limit() {
        let dir = git_dir();
        let d = dir.path();
        for i in 0..(MAX_LOCKS + 20) {
            std::fs::write(d.join(format!("refs/heads/b{i}.lock")), "").unwrap();
        }
        assert_eq!(find(d, d).len(), MAX_LOCKS);
    }
}
