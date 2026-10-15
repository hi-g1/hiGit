//! 테스트 도우미. 실제 git으로 임시 저장소를 만든다.
//! CI 러너의 전역 설정(예: Windows의 core.autocrlf=true)에 기대지 않게 저장소마다 설정한다.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use tempfile::TempDir;

use crate::CliBackend;

/// 테스트용 백엔드. 개발자 PC의 전역·시스템 git 설정(서명 강제, autocrlf 등)이 섞이지 않게 한다.
pub fn backend() -> CliBackend {
    CliBackend::new("git", Arc::new(isolate))
}

/// 전역 설정을 빈 파일로, 시스템 설정은 끈다.
pub fn isolate(cmd: &mut Command) {
    cmd.env("GIT_CONFIG_GLOBAL", empty_config())
        .env("GIT_CONFIG_NOSYSTEM", "1");
}

fn empty_config() -> &'static Path {
    static PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        let path = std::env::temp_dir().join("higit-test-empty-gitconfig");
        std::fs::write(&path, "").unwrap();
        path
    })
}

/// `dir`에서 git 명령을 실행하고, 실패하면 테스트를 멈춘다.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    isolate(&mut cmd);
    let out = cmd
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
    // 앱 백엔드로 저장할 때 쓰는 이름·이메일(전역 설정은 격리되어 있다).
    git(dir.path(), &["config", "user.name", "테스트"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    dir
}

/// 비교용 정규 경로. Windows의 `\\?\` 접두사와 8.3 짧은 이름 차이를 없앤다.
pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap()
}
