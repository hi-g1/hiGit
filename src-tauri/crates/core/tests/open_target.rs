//! "폴더에서 보기"·"기본 앱으로 열기" 경로 검사(D-216).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use higit_core::open_target::Purpose;
use higit_core::view::{ErrorCode, ErrorView, OpenHow};
use higit_core::{Core, GitBackend};

fn setup() -> (tempfile::TempDir, PathBuf, Core) {
    let repo = tempfile::tempdir().unwrap();
    let out = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let dir = platform::normalize_repo_path(repo.path()).unwrap();
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(&dir, OpenHow::Picked).unwrap();
    (repo, dir, core)
}

fn write(dir: &Path, rel: &str, bytes: &[u8]) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn code(core: &Core, dir: &Path, rel: &str, purpose: Purpose) -> Option<ErrorCode> {
    core.open_target(dir, rel, purpose)
        .err()
        .map(|e| ErrorView::from(&e).code)
}

#[test]
fn plain_files_open_and_reveal() {
    let (_repo, dir, core) = setup();
    for rel in ["a.txt", "그림.png", "문서/설명.md", "README"] {
        write(&dir, rel, b"hello");
        let path = core.open_target(&dir, rel, Purpose::Open).unwrap();
        assert_eq!(path, dir.join(rel));
        assert!(core.open_target(&dir, rel, Purpose::Reveal).is_ok());
    }
    // 폴더는 보기만 한다.
    assert!(core.open_target(&dir, "문서", Purpose::Reveal).is_ok());
    assert_eq!(
        code(&core, &dir, "문서", Purpose::Open),
        Some(ErrorCode::OpenRefused)
    );
}

#[test]
fn paths_outside_or_inside_git_are_refused() {
    let (_repo, dir, core) = setup();
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "secret.txt", b"x");
    write(&dir, "sub/a.txt", b"x");
    let abs = outside
        .path()
        .join("secret.txt")
        .to_string_lossy()
        .into_owned();
    for rel in [
        "../secret.txt",
        "sub/../../secret.txt",
        "sub/./a.txt",
        "sub//a.txt",
        abs.as_str(),
        "/etc/passwd",
        r"C:\Windows\win.ini",
        ".git/config",
        ".GIT/HEAD",
        "sub/.git/config",
        "",
    ] {
        for purpose in [Purpose::Open, Purpose::Reveal] {
            assert_eq!(
                code(&core, &dir, rel, purpose),
                Some(ErrorCode::OpenRefused),
                "{rel:?}"
            );
        }
    }
    assert_eq!(
        code(&core, &dir, "없는.txt", Purpose::Open),
        Some(ErrorCode::OpenMissing)
    );
    assert_eq!(
        code(&core, &dir, "sub/a.txt/b", Purpose::Reveal),
        Some(ErrorCode::OpenMissing)
    );
}

#[test]
fn links_are_not_followed() {
    let (_repo, dir, core) = setup();
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "secret.txt", b"x");
    if platform::symlink_file(&outside.path().join("secret.txt"), &dir.join("link.txt")).is_err()
        || platform::symlink_dir(outside.path(), &dir.join("linkdir")).is_err()
    {
        eprintln!("링크를 만들 수 없어 건너뜀");
        return;
    }
    for rel in ["link.txt", "linkdir/secret.txt", "linkdir"] {
        assert_eq!(
            code(&core, &dir, rel, Purpose::Open),
            Some(ErrorCode::OpenRefused),
            "{rel}"
        );
        assert_eq!(
            code(&core, &dir, rel, Purpose::Reveal),
            Some(ErrorCode::OpenRefused),
            "{rel}"
        );
    }
}

#[test]
fn files_that_can_run_are_not_opened() {
    let (_repo, dir, core) = setup();
    let cases: [(&str, &[u8]); 8] = [
        ("setup.EXE", b"x"),
        ("report.pdf.exe", b"x"),
        ("tool.Ps1", b"x"),
        ("install.sh", b"echo"),
        ("app.js", b"x"),
        ("program", b"\x7fELF\x02\x01"),
        ("script", b"#!/bin/sh\necho hi\n"),
        ("data.bin2", b"MZ\x90\x00"),
    ];
    for (rel, bytes) in cases {
        write(&dir, rel, bytes);
        assert_eq!(
            code(&core, &dir, rel, Purpose::Open),
            Some(ErrorCode::OpenRefused),
            "{rel}"
        );
        // 폴더에서 보기는 된다.
        assert!(
            core.open_target(&dir, rel, Purpose::Reveal).is_ok(),
            "{rel}"
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        write(&dir, "plain.txt", b"text");
        std::fs::set_permissions(
            dir.join("plain.txt"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert_eq!(
            code(&core, &dir, "plain.txt", Purpose::Open),
            Some(ErrorCode::OpenRefused)
        );
    }
}

#[cfg(windows)]
#[test]
fn windows_device_names_and_streams_are_refused() {
    let (_repo, dir, core) = setup();
    for rel in ["CON", "nul.txt", "sub/COM1", "a.txt:stream", "a.txt::$DATA"] {
        assert_eq!(
            code(&core, &dir, rel, Purpose::Reveal),
            Some(ErrorCode::OpenRefused),
            "{rel}"
        );
    }
}

#[test]
fn only_opened_repos() {
    let (_repo, _dir, core) = setup();
    let other = tempfile::tempdir().unwrap();
    write(other.path(), "a.txt", b"x");
    let err = core
        .open_target(other.path(), "a.txt", Purpose::Reveal)
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    assert!(core.project_root(other.path()).is_err());
}
