//! 올리지 않은 버전 수·브랜치에서 닿음·전체 메시지(D-097, D-099). 실제 git으로 확인한다.

use std::fs;
use std::path::Path;

use super::history::parse_messages;
use crate::testutil::{self, git};
use crate::{GitBackend, Oid, RepoPath};

fn commit(p: &Path, file: &str, message: &str) -> Oid {
    fs::write(p.join(file), message).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", message]);
    oid(p, "HEAD")
}

fn oid(p: &Path, rev: &str) -> Oid {
    Oid::parse(git(p, &["rev-parse", rev]).trim()).unwrap()
}

fn unpublished(p: &Path, tip: &Oid, base: Option<&Oid>) -> Option<u32> {
    testutil::backend()
        .count_unpublished(&RepoPath::new(p), tip, base)
        .unwrap()
}

#[test]
fn unpublished_counts_versions_no_remote_has() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    commit(p, "b.txt", "둘째");
    let head = oid(p, "HEAD");
    // 원격이 없으면 전부.
    assert_eq!(unpublished(p, &head, None), Some(2));
    assert_eq!(unpublished(p, &head, Some(&first)), Some(1));
    // 원격 추적 브랜치가 첫 버전을 가리키면 하나만 안 올림. 다른 원격 브랜치에 있어도 올린 것이다.
    git(
        p,
        &["update-ref", "refs/remotes/origin/main", first.as_str()],
    );
    assert_eq!(unpublished(p, &head, None), Some(1));
    git(
        p,
        &["update-ref", "refs/remotes/upstream/다른", head.as_str()],
    );
    assert_eq!(unpublished(p, &head, None), Some(0));
}

#[test]
fn too_many_remote_tips_means_unknown() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    let mut packed = String::from("# pack-refs with: peeled fully-peeled sorted \n");
    for i in 0..1001 {
        packed.push_str(&format!("{first} refs/remotes/origin/b{i:04}\n"));
    }
    fs::write(p.join(".git/packed-refs"), packed).unwrap();
    assert_eq!(unpublished(p, &first, None), None);
}

#[test]
fn reachable_from_local_branches_only() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    let second = commit(p, "b.txt", "둘째");
    git(p, &["reset", "-q", "--hard", first.as_str()]);
    // 둘째는 이제 백업 ref만 가리킨다.
    git(p, &["update-ref", "refs/higit/backup/x", second.as_str()]);
    let b = testutil::backend();
    let repo_path = RepoPath::new(p);
    assert!(b.reachable_from_branches(&repo_path, &first).unwrap());
    assert!(!b.reachable_from_branches(&repo_path, &second).unwrap());
}

#[test]
fn full_messages_keep_lines_as_written() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("a.txt"), "1").unwrap();
    git(p, &["add", "-A"]);
    git(
        p,
        &[
            "commit",
            "-q",
            "--cleanup=verbatim",
            "-m",
            "제목 줄\n\n# 샵으로 시작\n세 번째 \"따옴표\"",
        ],
    );
    let one = oid(p, "HEAD");
    let two = commit(p, "b.txt", "한 줄");
    let messages = testutil::backend()
        .commit_messages(&RepoPath::new(p), &[two.clone(), one.clone()])
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].oid, two);
    assert_eq!(messages[0].message, "한 줄");
    assert_eq!(
        messages[1].message,
        "제목 줄\n\n# 샵으로 시작\n세 번째 \"따옴표\""
    );
}

#[test]
fn message_parser_drops_a_cut_record() {
    let a = "a".repeat(40);
    let b = "b".repeat(40);
    let text = format!("\x1e{a}\0첫\n\n\x1e{b}\0둘");
    assert_eq!(parse_messages(&text, false).len(), 2);
    let cut = parse_messages(&text, true);
    assert_eq!(cut.len(), 1);
    assert_eq!(cut[0].message, "첫");
}
