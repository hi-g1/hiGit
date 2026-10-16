//! 비교의 마지막 묶음 뒤 접힌 줄을 세는 파일 줄 수(D-219)를 실제 git으로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{DiffRequest, FileDiffView, OpenHow};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false"])
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

fn lines(n: usize, eol: &str) -> String {
    (1..=n).map(|i| format!("줄 {i}{eol}")).collect()
}

fn setup(files: &[(&str, String)]) -> (tempfile::TempDir, std::path::PathBuf, Core) {
    let repo = tempfile::tempdir().unwrap();
    let dir = platform::normalize_repo_path(repo.path()).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
        ("core.autocrlf", "false"),
    ] {
        git(&dir, &["config", k, v]);
    }
    for (path, text) in files {
        std::fs::write(dir.join(path), text).unwrap();
    }
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "처음"]);
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(&dir, OpenHow::Picked).unwrap();
    (repo, dir, core)
}

fn diff(
    core: &Core,
    dir: &Path,
    path: &str,
    untracked: bool,
    commit: Option<String>,
) -> FileDiffView {
    core.queries()
        .diff(
            dir,
            &DiffRequest {
                path: path.into(),
                orig_path: None,
                untracked,
                commit,
                full_context: false,
            },
        )
        .unwrap()
}

/// 3번째 줄을 고친다.
fn edit_third(text: &str) -> String {
    text.replacen("줄 3", "고친 줄 3", 1)
}

#[test]
fn work_tree_files_count_lines_like_git() {
    let (_repo, dir, core) = setup(&[
        ("lf.txt", lines(20, "\n")),
        ("crlf.txt", lines(20, "\r\n")),
        (
            "tail.txt",
            lines(20, "\n").trim_end_matches('\n').to_string(),
        ),
    ]);
    for (path, want) in [("lf.txt", 20), ("crlf.txt", 20), ("tail.txt", 20)] {
        let text = std::fs::read_to_string(dir.join(path)).unwrap();
        std::fs::write(dir.join(path), edit_third(&text)).unwrap();
        let view = diff(&core, &dir, path, false, None);
        assert_eq!(view.new_line_count, Some(want), "{path}");
        // 마지막 묶음은 파일 끝까지 가지 않는다(접힌 줄이 있다).
        let last = view.hunks.last().unwrap();
        assert!(last.new_start + last.new_len - 1 < want, "{path}");
    }
}

#[test]
fn deleted_untracked_and_big_files_have_no_count() {
    let big = "가".repeat(400_000) + "\n";
    let (_repo, dir, core) = setup(&[("gone.txt", lines(5, "\n")), ("big.txt", big.clone())]);
    std::fs::remove_file(dir.join("gone.txt")).unwrap();
    assert_eq!(
        diff(&core, &dir, "gone.txt", false, None).new_line_count,
        None
    );
    std::fs::write(dir.join("new.txt"), lines(5, "\n")).unwrap();
    assert_eq!(
        diff(&core, &dir, "new.txt", true, None).new_line_count,
        None
    );
    std::fs::write(dir.join("big.txt"), big + "더\n").unwrap();
    assert_eq!(
        diff(&core, &dir, "big.txt", false, None).new_line_count,
        None
    );
}

#[test]
fn commit_diffs_count_the_blob_in_that_version() {
    let (_repo, dir, core) = setup(&[("a.txt", lines(30, "\n")), ("b.txt", lines(4, "\n"))]);
    std::fs::write(dir.join("a.txt"), edit_third(&lines(30, "\n"))).unwrap();
    std::fs::remove_file(dir.join("b.txt")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "고침"]);
    let head = git(&dir, &["rev-parse", "HEAD"]);
    // 작업 폴더를 또 바꿔도 버전 안 비교는 그 버전의 줄 수다.
    std::fs::write(dir.join("a.txt"), lines(3, "\n")).unwrap();
    assert_eq!(
        diff(&core, &dir, "a.txt", false, Some(head.clone())).new_line_count,
        Some(30)
    );
    assert_eq!(
        diff(&core, &dir, "b.txt", false, Some(head)).new_line_count,
        None
    );
}
