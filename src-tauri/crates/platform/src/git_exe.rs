use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// 시스템 git 실행 파일을 찾는다. PATH를 먼저 보고, 없으면 OS별 기본 설치 위치를 본다.
/// 결과는 늘 절대 경로다(D-035).
pub fn find_git() -> Option<PathBuf> {
    find_git_in(std::env::var_os("PATH").as_deref())
        .or_else(|| default_locations().into_iter().find(|p| p.is_file()))
}

/// PATH 값에서 절대 경로 항목만 남겨 git을 찾는다. `.`이나 상대 경로 항목은 버린다.
/// 그래야 저장소 폴더에 심은 가짜 `git`이 고르지 않는다. 환경을 읽지 않아 테스트가 바로 부른다.
fn find_git_in(path_var: Option<&OsStr>) -> Option<PathBuf> {
    let absolute: Vec<PathBuf> = std::env::split_paths(path_var?)
        .filter(|p| p.is_absolute())
        .collect();
    if absolute.is_empty() {
        return None;
    }
    let joined: OsString = std::env::join_paths(&absolute).ok()?;
    // 찾을 이름에 경로가 없으므로 cwd는 쓰이지 않는다. 그래도 절대 경로를 준다.
    let found = which::which_in("git", Some(joined), &absolute[0]).ok()?;
    found.is_absolute().then_some(found)
}

#[cfg(windows)]
fn default_locations() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for var in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"] {
        if let Some(dir) = std::env::var_os(var) {
            found.push(PathBuf::from(dir).join(r"Git\cmd\git.exe"));
        }
    }
    if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
        found.push(PathBuf::from(dir).join(r"Programs\Git\cmd\git.exe"));
    }
    found.push(PathBuf::from(r"C:\Program Files\Git\cmd\git.exe"));
    found
}

#[cfg(not(windows))]
fn default_locations() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 실행 가능한 가짜 git을 만든다. 진짜 git보다 앞에 두어도 골라지면 안 된다.
    fn fake_git(dir: &std::path::Path) {
        #[cfg(windows)]
        std::fs::write(dir.join("git.exe"), b"").unwrap();
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = dir.join("git");
            std::fs::write(&path, b"#!/bin/sh\nexit 0\n").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn relative_path_entries_are_ignored() {
        let real = find_git().expect("git 설치 필요");
        let real_dir = real.parent().unwrap().to_path_buf();

        // 현재 폴더 기준 상대 항목에 가짜 git을 두고, 그 항목을 진짜보다 앞에 둔다.
        let cwd = std::env::current_dir().unwrap();
        let fake = tempfile::tempdir_in(&cwd).unwrap();
        fake_git(fake.path());
        let relative = fake.path().strip_prefix(&cwd).unwrap().to_path_buf();
        for first in [PathBuf::from("."), relative] {
            let path = std::env::join_paths([first.clone(), real_dir.clone()]).unwrap();
            let got = find_git_in(Some(&path)).expect("진짜 git을 찾아야 한다");
            assert!(got.is_absolute(), "{got:?}");
            assert!(!got.starts_with(fake.path()), "{first:?} → {got:?}");
        }
    }

    #[test]
    fn absolute_entries_are_used_and_empty_is_none() {
        let fake = tempfile::tempdir().unwrap();
        fake_git(fake.path());
        let path = std::env::join_paths([fake.path()]).unwrap();
        let got = find_git_in(Some(&path)).unwrap();
        assert!(got.starts_with(fake.path()));
        assert_eq!(find_git_in(Some(OsStr::new(""))), None);
        assert_eq!(find_git_in(None), None);
    }

    #[test]
    fn finds_git_on_this_machine() {
        let git = find_git().expect("개발 환경과 CI에는 git이 설치되어 있어야 한다");
        assert!(git.is_file(), "{git:?}");
    }
}
