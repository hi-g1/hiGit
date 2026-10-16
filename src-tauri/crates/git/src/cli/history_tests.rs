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

fn files(p: &Path, oid: &Oid) -> crate::CommitFiles {
    testutil::backend()
        .commit_files(&RepoPath::new(p), oid)
        .unwrap()
}

fn summary(f: &crate::CommitFiles) -> Vec<String> {
    let mut v: Vec<String> = f
        .files
        .iter()
        .map(|x| {
            format!(
                "{:?} {}{}",
                x.change,
                x.orig_path
                    .as_deref()
                    .map(|o| format!("{o} → "))
                    .unwrap_or_default(),
                x.path
            )
        })
        .collect();
    v.sort();
    v
}

#[test]
fn commit_files_list_what_a_version_changed() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("고칠 파일.txt"), "원래\n").unwrap();
    fs::write(p.join("지울 파일.txt"), "지움\n").unwrap();
    fs::write(
        p.join("옛 이름.txt"),
        "이름을 바꿀 파일의 내용이 충분히 길어야 이름 바꿈으로 잡힌다\n",
    )
    .unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", "처음"]);
    let root = oid(p, "HEAD");
    // 첫 버전은 빈 트리와 비교한다.
    assert_eq!(summary(&files(p, &root)).len(), 3);
    assert!(
        summary(&files(p, &root))
            .iter()
            .all(|s| s.starts_with("Added"))
    );

    fs::write(p.join("고칠 파일.txt"), "고침\n").unwrap();
    fs::remove_file(p.join("지울 파일.txt")).unwrap();
    fs::rename(p.join("옛 이름.txt"), p.join("새 이름.txt")).unwrap();
    fs::write(p.join("그림.bin"), [0u8, 1, 2, 3]).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", "여러 가지"]);
    let second = oid(p, "HEAD");
    assert_eq!(
        summary(&files(p, &second)),
        [
            "Added 그림.bin",
            "Deleted 지울 파일.txt",
            "Modified 고칠 파일.txt",
            "Renamed 옛 이름.txt → 새 이름.txt",
        ]
    );
    // 빈 버전은 바뀐 파일이 없다.
    git(p, &["commit", "-q", "--allow-empty", "-m", "빈"]);
    let empty = oid(p, "HEAD");
    assert_eq!(files(p, &empty).total, 0);
}

#[test]
fn a_merge_version_is_compared_with_its_first_parent() {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "처음");
    git(p, &["switch", "-q", "-c", "곁가지"]);
    commit(p, "곁.txt", "곁");
    git(p, &["switch", "-q", "main"]);
    commit(p, "main.txt", "메인");
    git(p, &["merge", "-q", "--no-edit", "곁가지"]);
    let merge = oid(p, "HEAD");
    assert_eq!(summary(&files(p, &merge)), ["Added 곁.txt"]);
}

#[test]
fn commit_files_stop_listing_at_five_hundred() {
    let repo = testutil::init_repo();
    let p = repo.path();
    for i in 0..501 {
        fs::write(p.join(format!("f{i:03}.txt")), "x").unwrap();
    }
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", "많음"]);
    let f = files(p, &oid(p, "HEAD"));
    assert_eq!(f.total, 501);
    assert_eq!(f.files.len(), 500);
}

#[test]
fn commit_diff_shows_one_file_of_a_version() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let root = commit(p, "a.txt", "하나\n");
    let second = commit(p, "a.txt", "하나\n둘\n");
    let b = testutil::backend();
    let target = |oid: &Oid| crate::DiffTarget {
        path: "a.txt".into(),
        orig_path: None,
        kind: crate::DiffKind::Commit { oid: oid.clone() },
        full_context: false,
    };
    let d = b.diff(&RepoPath::new(p), &target(&second)).unwrap();
    let added: Vec<&str> = d.hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind == crate::LineKind::Added)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(added, ["둘"]);
    // 첫 버전은 빈 파일과.
    let d = b.diff(&RepoPath::new(p), &target(&root)).unwrap();
    assert!(
        d.hunks[0]
            .lines
            .iter()
            .all(|l| l.kind == crate::LineKind::Added)
    );
}
