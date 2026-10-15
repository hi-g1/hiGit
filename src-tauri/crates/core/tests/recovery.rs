//! 백업 정리와 작업 내역을 실제 git으로 시험한다(D-079, D-085).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::OpenHow;
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
    std::fs::write(dir.join("a.txt"), "1\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    repo
}

fn open(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn backups(dir: &Path) -> Vec<String> {
    let out = git(dir, &["for-each-ref", "--format=%(refname)", "refs/higit/"]);
    out.lines().map(str::to_string).collect()
}

#[test]
fn opening_prunes_old_backups_a_few_at_a_time() {
    let repo = repo();
    let dir = repo.path();
    let head = git(dir, &["rev-parse", "HEAD"]);
    // 2000년 백업 25개, 지금 백업 하나, 이름이 다른 것 하나, 맡긴 ref 하나.
    for n in 0..25 {
        git(
            dir,
            &[
                "update-ref",
                &format!("refs/higit/backup/20000101T000000Z-{n}/undo-main"),
                &head,
            ],
        );
    }
    git(
        dir,
        &[
            "update-ref",
            "refs/higit/backup/29990101T000000Z-0/undo-main",
            &head,
        ],
    );
    git(
        dir,
        &["update-ref", "refs/higit/backup/이상한-이름/x", &head],
    );
    git(dir, &["update-ref", "refs/higit/parked/main", &head]);
    open(dir);
    assert_eq!(backups(dir).len(), 28 - 20);
    open(dir);
    let left = backups(dir);
    assert_eq!(left.len(), 3, "{left:?}");
    assert!(left.iter().any(|r| r.contains("29990101")));
    assert!(left.iter().any(|r| r.contains("이상한-이름")));
    assert!(left.contains(&"refs/higit/parked/main".to_string()));
}
