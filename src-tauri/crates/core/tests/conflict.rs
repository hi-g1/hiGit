//! 충돌 해결 화면 조회를 실제 git으로 시험한다(D-069, D-086).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ConflictFileKind, ErrorCode, ErrorView, OpenHow, SideState};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn commit_all(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

const BASE: &str = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n";

/// main과 "기능"이 a.txt 세 곳과 이진 파일 b.bin을 다르게 고친 뒤 main에서 합치기를 시작한다.
/// c.txt는 기능 쪽에서만 고쳐 문제없이 합쳐진다.
fn merging() -> tempfile::TempDir {
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
    std::fs::write(dir.join("a.txt"), BASE).unwrap();
    std::fs::write(dir.join("b.bin"), b"\0base").unwrap();
    std::fs::write(dir.join("c.txt"), "c\n").unwrap();
    std::fs::write(dir.join("d.txt"), "d\n").unwrap();
    commit_all(dir, "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    std::fs::write(
        dir.join("a.txt"),
        BASE.replace("2\n", "기능2\n")
            .replace("6\n", "기능6\n")
            .replace("11\n", "기능11\n"),
    )
    .unwrap();
    std::fs::write(dir.join("b.bin"), b"\0theirs").unwrap();
    std::fs::write(dir.join("c.txt"), "c 기능\n").unwrap();
    std::fs::remove_file(dir.join("d.txt")).unwrap();
    commit_all(dir, "기능 쪽 고침");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(
        dir.join("a.txt"),
        BASE.replace("2\n", "내2\n")
            .replace("6\n", "내6\n")
            .replace("11\n", "내11\n"),
    )
    .unwrap();
    std::fs::write(dir.join("b.bin"), b"\0ours").unwrap();
    std::fs::write(dir.join("d.txt"), "d 내 고침\n").unwrap();
    commit_all(dir, "내 쪽 고침");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    repo
}

fn core(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

#[test]
fn state_lists_conflicted_files_and_both_sides() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    let state = core.queries().conflict_state(dir).unwrap();
    assert!(state.active && !state.other_operation);
    let ours = state.ours.unwrap();
    let theirs = state.theirs.unwrap();
    assert_eq!(
        (ours.label.as_str(), ours.summary.as_str()),
        ("main", "내 쪽 고침")
    );
    assert_eq!(
        (theirs.label.as_str(), theirs.summary.as_str()),
        ("기능", "기능 쪽 고침")
    );
    let files: Vec<(String, ConflictFileKind, u32)> = state
        .files
        .iter()
        .map(|f| (f.path.clone(), f.kind, f.hunks))
        .collect();
    assert_eq!(
        files,
        [
            ("a.txt".to_string(), ConflictFileKind::Text, 3),
            ("b.bin".to_string(), ConflictFileKind::Whole, 1),
            ("d.txt".to_string(), ConflictFileKind::Whole, 1),
        ]
    );
    assert_eq!(state.clean_files, 1, "c.txt");
}

#[test]
fn text_file_hunks_carry_no_markers() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    let file = core.queries().conflict_file(dir, "a.txt").unwrap();
    assert_eq!(file.kind, ConflictFileKind::Text);
    let lines: Vec<u32> = file.hunks.iter().map(|h| h.line).collect();
    assert_eq!(lines, [2, 6, 11]);
    assert_eq!(file.hunks[0].ours, ["내2"]);
    assert_eq!(file.hunks[0].theirs, ["기능2"]);
    assert_eq!(file.hunks[0].base, ["2"]);
    assert_eq!(file.hunks[0].before, ["1"]);
    assert_eq!(file.hunks[0].after, ["3", "4", "5"]);
    let json = serde_json::to_string(&file).unwrap();
    for marker in ["<<<<<<<", "=======", ">>>>>>>", "|||||||"] {
        assert!(!json.contains(marker), "{marker} in {json}");
    }
}

#[test]
fn binary_and_deleted_sides_are_whole_files() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    let bin = core.queries().conflict_file(dir, "b.bin").unwrap();
    assert_eq!(
        (bin.ours_side, bin.theirs_side),
        (SideState::Binary, SideState::Binary)
    );
    let deleted = core.queries().conflict_file(dir, "d.txt").unwrap();
    assert_eq!(deleted.kind, ConflictFileKind::Whole);
    assert_eq!(deleted.ours_side, SideState::Text);
    assert_eq!(deleted.theirs_side, SideState::Deleted);
    assert_eq!(deleted.ours_preview, ["d 내 고침"]);
    // 충돌이 아닌 파일·이상한 경로는 받지 않는다.
    for bad in ["c.txt", "../x"] {
        let err = core.queries().conflict_file(dir, bad).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput, "{bad}");
    }
}

#[test]
fn not_merging_and_other_operations() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    git(dir, &["merge", "--abort"]);
    let state = core.queries().conflict_state(dir).unwrap();
    assert!(!state.active && !state.other_operation && state.files.is_empty());
    // 다른 도구로 시작한 리베이스가 충돌로 멈췄다.
    git(dir, &["switch", "-q", "기능"]);
    git(dir, &["rebase", "main"]);
    let state = core.queries().conflict_state(dir).unwrap();
    assert!(!state.active);
    assert!(state.other_operation);
}
