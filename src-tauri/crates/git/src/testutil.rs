//! 테스트 도우미. 실제 git으로 임시 저장소를 만든다.
//! CI 러너의 전역 설정(예: Windows의 core.autocrlf=true)에 기대지 않게 저장소마다 설정한다.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use tempfile::TempDir;

use crate::CliBackend;

pub fn backend() -> CliBackend {
    CliBackend::new("git", Arc::new(|_| {}))
}

/// `dir`에서 git 명령을 실행하고, 실패하면 테스트를 멈춘다.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=테스트",
            "-c",
            "user.email=test@example.com",
            "-c",
            "core.autocrlf=false",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .env("LC_ALL", "C")
        .output()
        .expect("git 실행");
    assert!(
        out.status.success(),
        "git {args:?} 실패: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// 저장이 하나도 없는 빈 저장소(기본 갈래 main).
pub fn init_repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["config", "core.autocrlf", "false"]);
    dir
}

/// 비교용 정규 경로. Windows의 `\\?\` 접두사와 8.3 짧은 이름 차이를 없앤다.
pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap()
}
