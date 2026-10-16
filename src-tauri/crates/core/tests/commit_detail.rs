//! 버전 상세 조회와 버전 안의 비교(계획 008 E19, D-105). 실제 git으로.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ChangeKind, DiffRequest, ErrorCode, ErrorView, OpenHow};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args([
            "-c",
            "core.autocrlf=false",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.quotepath=off",
        ])
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
    ] {
        git(dir, &["config", k, v]);
    }
    repo
}

fn commit_all(dir: &Path, message: &str) -> String {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

#[test]
fn root_merge_and_korean_paths() {
    let repo = repo();
    let dir = repo.path();
    std::fs::create_dir(dir.join("폴더")).unwrap();
    std::fs::write(dir.join("폴더/한글.txt"), "가\n").unwrap();
    let root = commit_all(dir, "처음\n\n본문 줄");
    git(dir, &["switch", "-q", "-c", "기능"]);
    std::fs::write(dir.join("b.txt"), "b\n").unwrap();
    commit_all(dir, "기능");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(dir.join("폴더/한글.txt"), "나\n").unwrap();
    let mine = commit_all(dir, "내 것");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    let merged = git(dir, &["rev-parse", "HEAD"]);
    let core = core(dir);

    let d = core.queries().commit_detail(dir, &root).unwrap();
    assert_eq!(d.message.trim_end(), "처음\n\n본문 줄");
    assert_eq!((d.author.as_str(), d.parents.len()), ("테스트", 0));
    assert_eq!(d.files.len(), 1);
    assert_eq!(
        (d.files[0].path.as_str(), d.files[0].change),
        ("폴더/한글.txt", ChangeKind::Added)
    );

    let d = core.queries().commit_detail(dir, &merged).unwrap();
    assert_eq!(d.parents, [mine.clone(), git(dir, &["rev-parse", "기능"])]);
    // 첫 부모와 비교: 기능 쪽에서 들어온 b.txt만.
    let paths: Vec<_> = d.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["b.txt"]);
    assert_eq!(d.more_files, 0);

    let d = core.queries().commit_detail(dir, &mine).unwrap();
    assert_eq!(d.files[0].change, ChangeKind::Modified);
}

#[test]
fn bad_hashes_unopened_repos_and_limits() {
    let repo = repo();
    let dir = repo.path();
    for i in 0..501 {
        std::fs::write(dir.join(format!("{i:03}.txt")), "x").unwrap();
    }
    // 긴 메시지는 파일로 넘긴다(Windows 명령줄은 32K 글자까지).
    let note = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(note.path(), "긴 메시지 ".repeat(8000)).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-F", &note.path().to_string_lossy()]);
    let many = git(dir, &["rev-parse", "HEAD"]);
    let core = core(dir);
    let d = core.queries().commit_detail(dir, &many).unwrap();
    assert_eq!((d.files.len(), d.more_files), (500, 1));
    assert!(d.message.len() <= 64 * 1024 && d.message.starts_with("긴 메시지"));

    let tree = git(dir, &["rev-parse", "HEAD^{tree}"]);
    let dangling = git(dir, &["commit-tree", &tree, "-m", "떠돌이"]);
    for bad in [
        dangling.as_str(),
        "main",
        &many[..7],
        &"0".repeat(40),
        &tree,
    ] {
        let err = core.queries().commit_detail(dir, bad).unwrap_err();
        assert_eq!(code(&err), ErrorCode::InvalidInput, "{bad}");
    }
    let other = tempfile::tempdir().unwrap();
    let err = core
        .queries()
        .commit_detail(other.path(), &many)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotOpened);
}

#[test]
fn commit_diff_ignores_the_working_folder() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "1\n2\n").unwrap();
    commit_all(dir, "처음");
    std::fs::write(dir.join("a.txt"), "1\n둘\n").unwrap();
    let second = commit_all(dir, "둘");
    std::fs::write(dir.join("a.txt"), "작업 폴더에서 고침\n").unwrap();
    let core = core(dir);
    let request = |commit: Option<String>| DiffRequest {
        path: "a.txt".into(),
        orig_path: None,
        untracked: false,
        commit,
        full_context: false,
    };
    let view = core
        .queries()
        .diff(dir, &request(Some(second.clone())))
        .unwrap();
    let lines: Vec<String> = view
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter().map(|l| l.text.clone()))
        .collect();
    assert!(
        lines.contains(&"둘".to_string()) && lines.contains(&"2".to_string()),
        "{lines:?}"
    );
    assert!(!lines.iter().any(|l| l.contains("작업 폴더")));
    // 아무 해시나 받지 않는다.
    let err = core
        .queries()
        .diff(dir, &request(Some("0".repeat(40))))
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);
    let err = core
        .queries()
        .diff(
            dir,
            &DiffRequest {
                path: "../x".into(),
                ..request(Some(second))
            },
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);
}
