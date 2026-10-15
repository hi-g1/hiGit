//! 겹침 검사를 실제 git으로 두 방식 모두 시험한다. 2.38 미만이면 merge-tree 쪽은 건너뛴다.

use std::path::Path;

use super::merge_check::Method;
use crate::testutil;
use crate::{GitBackend, GitVersion, MergeCheck, RepoPath};

fn commit(dir: &Path, file: &str, text: &[u8]) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", "-A"]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

/// main과 side가 갈라진 저장소를 만들고 `ours=main`, `theirs=side`로 두 방식의 결과를 돌려준다.
fn check(setup: impl Fn(&Path), main: impl Fn(&Path), side: impl Fn(&Path)) -> Vec<MergeCheck> {
    let repo = testutil::init_repo();
    let dir = repo.path();
    setup(dir);
    testutil::git(dir, &["branch", "side"]);
    main(dir);
    testutil::git(dir, &["switch", "-q", "side"]);
    side(dir);
    testutil::git(dir, &["switch", "-q", "main"]);
    let index_before = std::fs::read(dir.join(".git/index")).unwrap();

    let backend = testutil::backend();
    let path = RepoPath::new(dir);
    let mut methods = vec![Method::ReadTree];
    if backend.version().unwrap() >= GitVersion::new(2, 38, 0) {
        methods.push(Method::MergeTree);
    }
    let results = methods
        .into_iter()
        .map(|m| {
            backend
                .merge_check_with(m, &path, None, "main", "side")
                .unwrap()
        })
        .collect();
    assert_eq!(std::fs::read(dir.join(".git/index")).unwrap(), index_before);
    assert!(backend.status(&path).unwrap().entries.is_empty());
    results
}

fn lines(n: usize, changed: Option<(usize, &str)>) -> Vec<u8> {
    (0..n)
        .map(|i| match changed {
            Some((at, text)) if at == i => format!("{text}\n"),
            _ => format!("줄 {i}\n"),
        })
        .collect::<String>()
        .into_bytes()
}

#[test]
fn different_lines_of_the_same_file_merge() {
    let r = check(
        |d| commit(d, "a.txt", &lines(20, None)),
        |d| commit(d, "a.txt", &lines(20, Some((1, "main 고침")))),
        |d| commit(d, "a.txt", &lines(20, Some((18, "side 고침")))),
    );
    assert!(r.iter().all(|x| *x == MergeCheck::Clean), "{r:?}");
}

#[test]
fn same_line_overlaps() {
    let r = check(
        |d| commit(d, "a.txt", &lines(5, None)),
        |d| commit(d, "a.txt", &lines(5, Some((2, "main 고침")))),
        |d| commit(d, "a.txt", &lines(5, Some((2, "side 고침")))),
    );
    for x in &r {
        assert_eq!(
            *x,
            MergeCheck::Overlap {
                paths: vec!["a.txt".into()]
            }
        );
    }
}

#[test]
fn different_files_merge() {
    let r = check(
        |d| commit(d, "a.txt", b"a\n"),
        |d| commit(d, "b.txt", b"b\n"),
        |d| commit(d, "한글 c.txt", b"c\n"),
    );
    assert!(r.iter().all(|x| *x == MergeCheck::Clean), "{r:?}");
}

#[test]
fn delete_versus_modify_overlaps() {
    let r = check(
        |d| commit(d, "a.txt", b"a\n"),
        |d| {
            std::fs::remove_file(d.join("a.txt")).unwrap();
            testutil::git(d, &["commit", "-q", "-am", "지움"]);
        },
        |d| commit(d, "a.txt", "a 고침\n".as_bytes()),
    );
    assert!(
        r.iter().all(|x| matches!(x, MergeCheck::Overlap { .. })),
        "{r:?}"
    );
}

#[test]
fn binary_changed_on_both_sides_overlaps() {
    let r = check(
        |d| commit(d, "img.bin", &[0, 1, 2, 3]),
        |d| commit(d, "img.bin", &[0, 1, 2, 4]),
        |d| commit(d, "img.bin", &[0, 1, 2, 5]),
    );
    assert!(
        r.iter().all(|x| matches!(x, MergeCheck::Overlap { .. })),
        "{r:?}"
    );
}

#[test]
fn unrelated_histories_use_an_empty_base() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", b"a\n");
    testutil::git(dir, &["switch", "-q", "--orphan", "other"]);
    commit(dir, "b.txt", b"b\n");
    testutil::git(dir, &["switch", "-q", "main"]);
    let backend = testutil::backend();
    let path = RepoPath::new(dir);
    let got = backend
        .merge_check_with(Method::ReadTree, &path, None, "main", "other")
        .unwrap();
    assert_eq!(got, MergeCheck::Clean);
}

#[test]
fn public_entry_picks_a_method_for_this_git() {
    let r = check(
        |d| commit(d, "a.txt", &lines(5, None)),
        |d| commit(d, "a.txt", &lines(5, Some((2, "main")))),
        |d| commit(d, "a.txt", &lines(5, Some((2, "side")))),
    );
    assert!(!r.is_empty());
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", b"a\n");
    let path = RepoPath::new(dir);
    assert_eq!(
        testutil::backend()
            .merge_check(&path, None, "main", "main")
            .unwrap(),
        MergeCheck::Clean
    );
}
