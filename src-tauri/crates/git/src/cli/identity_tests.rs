//! 전역 이름·이메일을 시험마다 새 임시 `GIT_CONFIG_GLOBAL` 파일로 시험한다(계획 010 R5).

use std::path::{Path, PathBuf};

use crate::testutil;
use crate::{CliBackend, ConfigureCommand, GitBackend, GitIdentity, GlobalOp};

/// 전역은 `global`, 시스템은 `system`(없으면 읽지 않음)인 백엔드.
fn backend(global: &Path, system: Option<&Path>) -> CliBackend {
    let global: PathBuf = global.to_path_buf();
    let system: Option<PathBuf> = system.map(Path::to_path_buf);
    let configure: ConfigureCommand =
        std::sync::Arc::new(move |cmd: &mut std::process::Command| {
            cmd.env("GIT_CONFIG_GLOBAL", &global);
            match &system {
                Some(path) => cmd
                    .env("GIT_CONFIG_SYSTEM", path)
                    .env_remove("GIT_CONFIG_NOSYSTEM"),
                None => cmd.env("GIT_CONFIG_NOSYSTEM", "1"),
            };
        });
    CliBackend::new("git", configure)
}

fn set(name: &str, email: &str) -> GlobalOp {
    GlobalOp::SetIdentity {
        name: name.into(),
        email: email.into(),
    }
}

#[test]
fn empty_global_has_no_identity() {
    let global = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        backend(global.path(), None).identity().unwrap(),
        GitIdentity::default()
    );
}

#[test]
fn write_then_read_and_overwrite() {
    let global = tempfile::NamedTempFile::new().unwrap();
    let git = backend(global.path(), None);
    git.run_global(&set("김지원", "ji+won@example.com"))
        .unwrap();
    assert_eq!(
        git.identity().unwrap(),
        GitIdentity {
            name: Some("김지원".into()),
            email: Some("ji+won@example.com".into())
        }
    );
    git.run_global(&set("Jiwon Kim", "jiwon@example.com"))
        .unwrap();
    assert_eq!(git.identity().unwrap().name.as_deref(), Some("Jiwon Kim"));
    let file = std::fs::read_to_string(global.path()).unwrap();
    assert_eq!(file.matches("name =").count(), 1, "{file}");
}

#[test]
fn includes_and_system_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let included = dir.path().join("included");
    std::fs::write(&included, "[user]\n\tname = 가져온 이름\n").unwrap();
    let global = dir.path().join("global");
    std::fs::write(
        &global,
        format!(
            "[include]\n\tpath = {}\n",
            included.to_string_lossy().replace('\\', "/")
        ),
    )
    .unwrap();
    let system = dir.path().join("system");
    std::fs::write(&system, "[user]\n\temail = system@example.com\n").unwrap();
    let id = backend(&global, Some(&system)).identity().unwrap();
    assert_eq!(id.name.as_deref(), Some("가져온 이름"));
    assert_eq!(id.email.as_deref(), Some("system@example.com"));
}

#[test]
fn repository_local_identity_is_not_mixed_in() {
    let global = tempfile::NamedTempFile::new().unwrap();
    let repo = testutil::init_repo();
    testutil::git(repo.path(), &["config", "user.name", "저장소 이름"]);
    // 저장소 안에서 앱이 돌아도 전역만 읽는다(cwd를 쓰지 않음).
    let id = backend(global.path(), None).identity().unwrap();
    assert_eq!(id.name, None);
}

#[test]
fn multiple_values_refuse_overwrite() {
    let global = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(global.path(), "[user]\n\tname = 하나\n\tname = 둘\n").unwrap();
    let git = backend(global.path(), None);
    assert!(git.run_global(&set("셋", "a@example.com")).is_err());
    let file = std::fs::read_to_string(global.path()).unwrap();
    assert!(file.contains("하나") && file.contains("둘"), "{file}");
}

#[test]
fn dash_value_is_stored_as_value() {
    // core가 `-`로 시작하는 값을 먼저 거절한다(더 엄격한 정책). git은 값으로 그대로 저장한다(2.34.1).
    let global = tempfile::NamedTempFile::new().unwrap();
    let git = backend(global.path(), None);
    git.run_global(&set("-x", "a@example.com")).unwrap();
    assert_eq!(git.identity().unwrap().name.as_deref(), Some("-x"));
}

#[cfg(unix)]
#[test]
fn read_only_global_fails_to_write() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let global = dir.path().join("global");
    std::fs::write(&global, "").unwrap();
    // git은 잠금 파일을 폴더에 만들므로 폴더를 읽기 전용으로 한다.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    // root(CI의 기본 git 컨테이너)는 읽기 전용 폴더에도 쓸 수 있어 이 경우를 만들 수 없다.
    if std::fs::write(dir.path().join("probe"), "").is_ok() {
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let result = backend(&global, None).run_global(&set("이름", "a@example.com"));
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
}

#[test]
fn failed_write_does_not_carry_the_values() {
    let dir = tempfile::tempdir().unwrap();
    let git = backend(&dir.path().join("없는 폴더").join("config"), None);
    let err = git
        .run_global(&set("김지원", "secret@example.com"))
        .unwrap_err();
    let shown = format!("{err} {err:?}");
    assert!(shown.contains("user.name"), "{shown}");
    assert!(!shown.contains("김지원"), "{shown}");
    assert!(!shown.contains("secret@example.com"), "{shown}");
}
