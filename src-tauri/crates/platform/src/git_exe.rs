use std::path::PathBuf;

/// 시스템 git 실행 파일을 찾는다. PATH를 먼저 보고, 없으면 OS별 기본 설치 위치를 본다.
pub fn find_git() -> Option<PathBuf> {
    which::which("git")
        .ok()
        .or_else(|| default_locations().into_iter().find(|p| p.is_file()))
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

    #[test]
    fn finds_git_on_this_machine() {
        let git = find_git().expect("개발 환경과 CI에는 git이 설치되어 있어야 한다");
        assert!(git.is_file(), "{git:?}");
    }
}
