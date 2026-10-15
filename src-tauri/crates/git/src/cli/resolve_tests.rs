//! 구간 나누기·충돌 해결·백업 커밋을 실제 git으로 시험한다.

use std::path::Path;

use crate::testutil;
use crate::{FailureKind, GitBackend, GitOp, MergeSegment, RepoPath};

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", file]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

fn same(s: &str) -> MergeSegment {
    MergeSegment::Same(s.as_bytes().to_vec())
}

fn conflict(ours: &str, base: &str, theirs: &str) -> MergeSegment {
    MergeSegment::Conflict {
        ours: ours.as_bytes().to_vec(),
        base: base.as_bytes().to_vec(),
        theirs: theirs.as_bytes().to_vec(),
    }
}

#[test]
fn segments_split_same_and_conflicting_lines() {
    let repo = testutil::init_repo();
    let backend = testutil::backend();
    let path = RepoPath::new(repo.path());
    let got = backend
        .merge_segments(
            &path,
            b"\xec\x95\x84\nb\nc\nd\ne\n",
            "아\n내 고침\nc\nd\ne\n".as_bytes(),
            "아\n남 고침\nc\nd\nE\n".as_bytes(),
        )
        .unwrap();
    assert_eq!(
        got,
        [
            same("아\n"),
            conflict("내 고침\n", "b\n", "남 고침\n"),
            same("c\nd\nE\n"),
        ]
    );
}

#[test]
fn short_marker_looking_lines_are_content() {
    let repo = testutil::init_repo();
    let backend = testutil::backend();
    let path = RepoPath::new(repo.path());
    let text = "<<<<<<< 제목\n=======\n>>>>>>> 끝\n";
    let ours = format!("{text}내 쪽\n");
    let theirs = format!("{text}남 쪽\n");
    let got = backend
        .merge_segments(
            &path,
            format!("{text}원래\n").as_bytes(),
            ours.as_bytes(),
            theirs.as_bytes(),
        )
        .unwrap();
    assert_eq!(got, [same(text), conflict("내 쪽\n", "원래\n", "남 쪽\n")]);
}

#[test]
fn long_marker_lines_and_binary_are_refused() {
    let repo = testutil::init_repo();
    let backend = testutil::backend();
    let path = RepoPath::new(repo.path());
    let marker = format!("{}\n", "=".repeat(40));
    assert!(
        backend
            .merge_segments(&path, b"a\n", marker.as_bytes(), b"b\n")
            .is_err()
    );
    assert!(
        backend
            .merge_segments(&path, b"a\0\n", b"b\0\n", b"c\0\n")
            .is_err()
    );
}

#[test]
fn crlf_lines_stay_as_they_are() {
    let repo = testutil::init_repo();
    let backend = testutil::backend();
    let path = RepoPath::new(repo.path());
    let got = backend
        .merge_segments(&path, b"a\r\nb\r\n", b"a\r\nX\r\n", b"a\r\nY\r\n")
        .unwrap();
    assert_eq!(got, [same("a\r\n"), conflict("X\r\n", "b\r\n", "Y\r\n")]);
}

/// main과 기능이 a.txt 같은 줄을 다르게 고친 뒤 합치기를 시작해 충돌 상태로 둔다.
fn conflicted() -> (tempfile::TempDir, RepoPath) {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "원래\n");
    testutil::git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "a.txt", "기능 쪽\n");
    testutil::git(dir, &["switch", "-q", "main"]);
    commit(dir, "a.txt", "내 쪽\n");
    let path = RepoPath::new(dir);
    let err = testutil::backend()
        .run(
            &path,
            &GitOp::Merge {
                rev: "기능".into(),
                message: "합침".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::MergeConflict);
    (repo, path)
}

#[test]
fn resolve_file_writes_index_and_worktree() {
    let (repo, path) = conflicted();
    let dir = repo.path();
    let backend = testutil::backend();
    backend
        .run(
            &path,
            &GitOp::ResolveFile {
                path: "a.txt".into(),
                mode: "100644".into(),
                bytes: "내 쪽\n기능 쪽\n".as_bytes().to_vec(),
            },
        )
        .unwrap();
    assert!(
        backend
            .index_entries(&path)
            .unwrap()
            .iter()
            .all(|e| e.stage == 0)
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "내 쪽\n기능 쪽\n"
    );
    // 합치는 중이라 저장하면 부모가 둘인 버전이 된다.
    backend
        .run(
            &path,
            &GitOp::Commit {
                message: "합침".into(),
            },
        )
        .unwrap();
    let parents = testutil::git(dir, &["rev-list", "--parents", "-n", "1", "HEAD"]);
    assert_eq!(parents.split_whitespace().count(), 3);
}

#[test]
fn resolve_refuses_paths_outside_the_repo_and_odd_modes() {
    let (_repo, path) = conflicted();
    let backend = testutil::backend();
    for bad in ["../x", ".git/config", "/etc/x"] {
        let op = GitOp::ResolveFile {
            path: bad.into(),
            mode: "100644".into(),
            bytes: b"x".to_vec(),
        };
        assert!(backend.run(&path, &op).is_err(), "{bad}");
        assert!(
            backend
                .run(&path, &GitOp::ResolveDelete { path: bad.into() })
                .is_err()
        );
    }
    let op = GitOp::ResolveFile {
        path: "a.txt".into(),
        mode: "160000".into(),
        bytes: b"x".to_vec(),
    };
    assert!(backend.run(&path, &op).is_err());
}

#[test]
fn resolved_file_follows_the_line_ending_setting() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, ".gitattributes", "*.txt text eol=crlf\n");
    commit(dir, "a.txt", "원래\n");
    testutil::git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "a.txt", "기능\n");
    testutil::git(dir, &["switch", "-q", "main"]);
    commit(dir, "a.txt", "내 쪽\n");
    let path = RepoPath::new(dir);
    let backend = testutil::backend();
    let _ = backend.run(
        &path,
        &GitOp::Merge {
            rev: "기능".into(),
            message: "합침".into(),
        },
    );
    backend
        .run(
            &path,
            &GitOp::ResolveFile {
                path: "a.txt".into(),
                mode: "100644".into(),
                bytes: "골랐음\n".as_bytes().to_vec(),
            },
        )
        .unwrap();
    assert_eq!(
        std::fs::read(dir.join("a.txt")).unwrap(),
        "골랐음\r\n".as_bytes()
    );
    assert_eq!(testutil::git(dir, &["show", ":a.txt"]), "골랐음\n");
}

#[test]
fn resolve_delete_takes_the_deleting_side() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "원래\n");
    testutil::git(dir, &["switch", "-q", "-c", "기능"]);
    testutil::git(dir, &["rm", "-q", "a.txt"]);
    testutil::git(dir, &["commit", "-q", "-m", "지움"]);
    testutil::git(dir, &["switch", "-q", "main"]);
    commit(dir, "a.txt", "내 쪽\n");
    let path = RepoPath::new(dir);
    let backend = testutil::backend();
    let _ = backend.run(
        &path,
        &GitOp::Merge {
            rev: "기능".into(),
            message: "합침".into(),
        },
    );
    backend
        .run(
            &path,
            &GitOp::ResolveDelete {
                path: "a.txt".into(),
            },
        )
        .unwrap();
    assert!(!dir.join("a.txt").exists());
    assert!(backend.index_entries(&path).unwrap().is_empty());
}

#[test]
fn snapshot_keeps_conflicted_files_without_touching_the_index() {
    let (repo, path) = conflicted();
    let dir = repo.path();
    // 사용자가 외부 편집기로 충돌 파일을 고쳐 두었다.
    std::fs::write(dir.join("a.txt"), "직접 고침\n").unwrap();
    let backend = testutil::backend();
    let out = backend
        .run(
            &path,
            &GitOp::SnapshotFiles {
                paths: vec!["a.txt".into()],
                message: "충돌 파일".into(),
            },
        )
        .unwrap();
    let oid = out.oid.unwrap();
    assert_eq!(
        testutil::git(dir, &["show", &format!("{oid}:a.txt")]),
        "직접 고침\n"
    );
    let parents = testutil::git(dir, &["rev-list", "--parents", "-n", "1", oid.as_str()]);
    assert_eq!(parents.split_whitespace().count(), 3, "stash 모양: 부모 둘");
    // index의 충돌 항목은 그대로다.
    let stages: Vec<u8> = backend
        .index_entries(&path)
        .unwrap()
        .iter()
        .map(|e| e.stage)
        .collect();
    assert_eq!(stages.len(), 3, "{stages:?}");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "직접 고침\n"
    );
}
