use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

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

/// Git 내려받기 페이지(D-053). 화면은 주소를 넘기지 않고 앱 명령만 부른다.
pub fn git_download_url() -> &'static str {
    if cfg!(windows) {
        "https://git-scm.com/download/win"
    } else if cfg!(target_os = "macos") {
        "https://git-scm.com/download/mac"
    } else {
        "https://git-scm.com/download/linux"
    }
}

/// git이 어떻게 설치됐는지. 배포판 패키지는 보안 수정을 버전 번호를 올리지 않고 넣으므로
/// 버전 문자열만으로 보안 수정 여부를 판단할 수 없다(D-050).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitBuild {
    /// Git for Windows처럼 git 프로젝트 버전 번호를 그대로 쓰는 설치.
    Official,
    /// dpkg가 관리하는 패키지. `version`은 패키지 버전(예: `1:2.34.1-1ubuntu1.17`).
    DistroPackage {
        version: String,
    },
    Unknown,
}

/// `git` 실행 파일의 설치 방식을 알아낸다. 시작할 때 한 번 부른다.
pub fn git_build(git: &Path) -> GitBuild {
    #[cfg(target_os = "linux")]
    {
        linux_package(git).unwrap_or(GitBuild::Unknown)
    }
    #[cfg(windows)]
    {
        // Git for Windows는 `...\Git\cmd\git.exe`나 `...\Git\bin\git.exe`에 깔린다.
        let parent = git
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str());
        let file = git.file_name().and_then(|n| n.to_str());
        match (parent, file) {
            (Some(p), Some(f))
                if f.eq_ignore_ascii_case("git.exe")
                    && (p.eq_ignore_ascii_case("cmd") || p.eq_ignore_ascii_case("bin")) =>
            {
                GitBuild::Official
            }
            _ => GitBuild::Unknown,
        }
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = git;
        GitBuild::Unknown
    }
}

#[cfg(target_os = "linux")]
fn linux_package(git: &Path) -> Option<GitBuild> {
    use std::process::Command;
    const DPKG_QUERY: &str = "/usr/bin/dpkg-query";
    if !Path::new(DPKG_QUERY).is_file() {
        return None;
    }
    let real = std::fs::canonicalize(git).ok()?;
    let owner = Command::new(DPKG_QUERY)
        .arg("-S")
        .arg(&real)
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    if !owner.status.success() {
        return None;
    }
    // `git: /usr/bin/git` 또는 `git, git-man: /path`. 첫 패키지 이름을 쓴다.
    let text = String::from_utf8_lossy(&owner.stdout);
    let package = text.split(": ").next()?.split(", ").next()?.trim();
    if package.is_empty() || package.starts_with('-') {
        return None;
    }
    let version = Command::new(DPKG_QUERY)
        .args(["-W", "-f=${Version}", package])
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    let version = String::from_utf8_lossy(&version.stdout).trim().to_string();
    (!version.is_empty()).then_some(GitBuild::DistroPackage { version })
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
    fn git_build_does_not_fail() {
        let git = find_git().unwrap();
        // 결과는 환경마다 다르다(CI 러너의 git은 PPA일 수 있다). 패닉 없이 끝나기만 확인한다.
        eprintln!("git build: {:?}", git_build(&git));
        assert_eq!(git_build(Path::new("/없는/git")), GitBuild::Unknown);
    }

    #[test]
    fn finds_git_on_this_machine() {
        let git = find_git().expect("개발 환경과 CI에는 git이 설치되어 있어야 한다");
        assert!(git.is_file(), "{git:?}");
    }
}
