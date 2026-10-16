//! 큰 파일 비교 상한(001 10b)을 실제 git으로 시험한다.

use crate::testutil;
use crate::{DiffKind, DiffTarget, GitBackend, RepoPath};

fn big_text(lines: usize, tag: &str) -> String {
    (0..lines).map(|i| format!("{tag} 줄 {i:08}\n")).collect()
}

fn target(path: &str, kind: DiffKind) -> DiffTarget {
    DiffTarget {
        path: path.into(),
        orig_path: None,
        kind,
        full_context: false,
    }
}

#[test]
fn huge_untracked_file_is_read_partially_without_git() {
    let repo = testutil::init_repo();
    let text = big_text(200_000, "새"); // 약 4MB
    std::fs::write(repo.path().join("큰 로그.txt"), &text).unwrap();
    let diff = testutil::backend()
        .diff(
            &RepoPath::new(repo.path()),
            &target("큰 로그.txt", DiffKind::Untracked),
        )
        .unwrap();
    assert!(diff.truncated);
    let lines: usize = diff.hunks.iter().map(|h| h.lines.len()).sum();
    assert!(lines > 0 && lines <= super::diff::MAX_LINES, "{lines}");
    assert_eq!(diff.hunks[0].lines[0].text, "새 줄 00000000");
}

#[test]
fn huge_untracked_binary_is_binary() {
    let repo = testutil::init_repo();
    let mut bytes = vec![0u8; 3 * 1024 * 1024];
    bytes[10] = 1;
    std::fs::write(repo.path().join("큰.bin"), &bytes).unwrap();
    let diff = testutil::backend()
        .diff(
            &RepoPath::new(repo.path()),
            &target("큰.bin", DiffKind::Untracked),
        )
        .unwrap();
    assert!(diff.binary);
}

#[test]
fn huge_tracked_change_is_capped() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), big_text(150_000, "옛")).unwrap();
    testutil::git(dir, &["add", "a.txt"]);
    testutil::git(dir, &["commit", "-q", "-m", "큰 파일"]);
    std::fs::write(dir.join("a.txt"), big_text(150_000, "새")).unwrap();
    let diff = testutil::backend()
        .diff(&RepoPath::new(dir), &target("a.txt", DiffKind::Tracked))
        .unwrap();
    assert!(diff.truncated);
    let lines: usize = diff.hunks.iter().map(|h| h.lines.len()).sum();
    assert!(lines <= super::diff::MAX_LINES, "{lines}");
}

#[test]
fn small_files_are_whole() {
    let repo = testutil::init_repo();
    std::fs::write(repo.path().join("작은.txt"), "하나\n둘\n").unwrap();
    let diff = testutil::backend()
        .diff(
            &RepoPath::new(repo.path()),
            &target("작은.txt", DiffKind::Untracked),
        )
        .unwrap();
    assert!(!diff.truncated);
    assert_eq!(diff.hunks[0].lines.len(), 2);
}

/// 30줄 파일에서 2번째와 25번째 줄을 바꾼다: 보통은 묶음 둘, 전체 문맥이면 하나에 모든 줄(D-156).
#[test]
fn full_context_merges_hunks_and_shows_every_line() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    let lines = |changed: bool| {
        (1..=30)
            .map(|i| {
                if changed && (i == 2 || i == 25) {
                    format!("바뀐 {i}\n")
                } else {
                    format!("줄 {i}\n")
                }
            })
            .collect::<String>()
    };
    std::fs::write(dir.join("a.txt"), lines(false)).unwrap();
    testutil::git(dir, &["add", "a.txt"]);
    testutil::git(dir, &["commit", "-q", "-m", "처음"]);
    std::fs::write(dir.join("a.txt"), lines(true)).unwrap();
    let backend = testutil::backend();
    let normal = backend
        .diff(&RepoPath::new(dir), &target("a.txt", DiffKind::Tracked))
        .unwrap();
    assert_eq!(normal.hunks.len(), 2);
    assert_eq!(
        (normal.hunks[1].new_start, normal.hunks[1].new_len),
        (22, 7)
    );
    let full = backend
        .diff(
            &RepoPath::new(dir),
            &DiffTarget {
                full_context: true,
                ..target("a.txt", DiffKind::Tracked)
            },
        )
        .unwrap();
    assert_eq!(full.hunks.len(), 1);
    let new_side = full.hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind != crate::LineKind::Removed)
        .count();
    assert_eq!(new_side, 30);
    assert!(!full.truncated);
}

#[test]
fn full_context_for_untracked_files() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    std::fs::write(dir.join("새.txt"), "하나\n둘\n").unwrap();
    let backend = testutil::backend();
    let untracked = backend
        .diff(
            &RepoPath::new(dir),
            &DiffTarget {
                full_context: true,
                ..target("새.txt", DiffKind::Untracked)
            },
        )
        .unwrap();
    assert_eq!(untracked.hunks[0].lines.len(), 2);
}
