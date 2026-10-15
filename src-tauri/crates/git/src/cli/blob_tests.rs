//! index 항목과 블롭 읽기를 실제 git으로 시험한다.

use std::path::Path;

use crate::testutil;
use crate::{GitBackend, GitOp, Oid, RepoPath};

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", file]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

#[test]
fn index_entries_and_blob_heads() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "한글 파일.txt", "첫 줄\n둘째 줄\n");
    std::fs::write(dir.join("큰.bin"), vec![b'x'; 1024 * 1024 + 1]).unwrap();
    testutil::git(dir, &["add", "큰.bin"]);
    let backend = testutil::backend();
    let path = RepoPath::new(dir);
    let mut entries = backend.index_entries(&path).unwrap();
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e.stage == 0 && e.mode == "100644"));
    let oids: Vec<Oid> = entries.iter().map(|e| e.oid.clone()).collect();
    let heads = backend.read_blobs(&path, &oids, 4).unwrap();
    let text = heads.iter().find(|h| h.oid == entries[1].oid).unwrap();
    assert_eq!(text.head, "첫 줄\n".as_bytes()[..4]);
    assert_eq!(text.size, "첫 줄\n둘째 줄\n".len() as u64);
    // 1MB가 넘으면 크기만 알린다.
    let big = heads.iter().find(|h| h.oid == entries[0].oid).unwrap();
    assert_eq!(big.size, 1024 * 1024 + 1);
    assert!(big.head.is_empty());
    // 없는 해시는 빠진다.
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    assert!(
        backend
            .read_blobs(&path, &[missing], 10)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn conflict_shows_three_stages() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "원래\n");
    testutil::git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "a.txt", "기능 쪽\n");
    testutil::git(dir, &["switch", "-q", "main"]);
    commit(dir, "a.txt", "내 쪽\n");
    let backend = testutil::backend();
    let path = RepoPath::new(dir);
    let err = backend
        .run(
            &path,
            &GitOp::Merge {
                rev: "기능".into(),
                message: "합침".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), crate::FailureKind::MergeConflict);
    let mut stages: Vec<u8> = backend
        .index_entries(&path)
        .unwrap()
        .iter()
        .filter(|e| e.path == "a.txt")
        .map(|e| e.stage)
        .collect();
    stages.sort();
    assert_eq!(stages, [1, 2, 3]);
}
