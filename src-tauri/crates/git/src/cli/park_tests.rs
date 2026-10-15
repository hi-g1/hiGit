//! 맡겨 두기·꺼내기를 실제 git으로 시험한다.

use std::path::Path;

use crate::testutil;
use crate::{FailureKind, GitBackend, GitOp, Oid, RepoPath, StatusEntry};

fn commit(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    testutil::git(dir, &["add", file]);
    testutil::git(dir, &["commit", "-q", "-m", file]);
}

fn entries(path: &RepoPath) -> Vec<StatusEntry> {
    let mut e = testutil::backend().status(path).unwrap().entries;
    e.sort_by(|a, b| a.path.cmp(&b.path));
    e
}

fn stash_list(dir: &Path) -> String {
    testutil::git(dir, &["stash", "list", "--format=%H %s"])
}

fn park(path: &RepoPath, message: &str) -> Option<Oid> {
    let backend = testutil::backend();
    let oid = backend
        .run(
            path,
            &GitOp::Park {
                message: message.into(),
            },
        )
        .unwrap()
        .oid;
    if let Some(oid) = &oid {
        backend
            .run(path, &GitOp::StashDropIf { oid: oid.clone() })
            .unwrap();
    }
    oid
}

/// 수정, 지움, 넣은 파일, 새 파일(한글·공백 이름)이 섞인 저장소.
fn mixed_repo() -> tempfile::TempDir {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n");
    commit(dir, "지울 파일.txt", "x\n");
    commit(dir, "b.txt", "b\n");
    std::fs::write(dir.join("a.txt"), "a 고침\n").unwrap();
    std::fs::remove_file(dir.join("지울 파일.txt")).unwrap();
    std::fs::write(dir.join("b.txt"), "b 넣음\n").unwrap();
    testutil::git(dir, &["add", "b.txt"]);
    std::fs::write(dir.join("새 파일 한글.txt"), "새\n").unwrap();
    repo
}

#[test]
fn park_and_unpark_round_trip_without_touching_user_stash_list() {
    let repo = mixed_repo();
    let dir = repo.path();
    let path = RepoPath::new(dir);
    let before = entries(&path);
    let list_before = stash_list(dir);

    let oid = park(&path, "higit parked main").expect("맡길 것이 있다");
    assert!(entries(&path).is_empty(), "맡긴 뒤 작업 폴더는 깨끗하다");
    assert_eq!(stash_list(dir), list_before, "사용자 stash 목록은 그대로");

    testutil::backend()
        .run(&path, &GitOp::Unpark { oid })
        .unwrap();
    assert_eq!(entries(&path), before);
}

#[test]
fn nothing_to_park_keeps_existing_user_stash() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n");
    // 사용자가 직접 만든 stash.
    std::fs::write(dir.join("a.txt"), "사용자 것\n").unwrap();
    testutil::git(dir, &["stash", "push", "-q", "-m", "사용자 stash"]);
    // 무시된 파일만 있다.
    std::fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
    testutil::git(dir, &["add", ".gitignore"]);
    testutil::git(dir, &["commit", "-q", "-m", "ignore"]);
    std::fs::write(dir.join("빌드.log"), "무시됨\n").unwrap();
    let list_before = stash_list(dir);
    assert!(list_before.contains("사용자 stash"));

    let path = RepoPath::new(dir);
    assert_eq!(park(&path, "higit parked main"), None);
    assert_eq!(stash_list(dir), list_before);
    assert!(dir.join("빌드.log").exists(), "무시된 파일은 맡기지 않는다");
}

#[test]
fn user_stash_survives_a_real_park() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n");
    std::fs::write(dir.join("a.txt"), "사용자 것\n").unwrap();
    testutil::git(dir, &["stash", "push", "-q", "-m", "사용자 stash"]);
    let list_before = stash_list(dir);
    std::fs::write(dir.join("a.txt"), "앱이 맡길 것\n").unwrap();
    let path = RepoPath::new(dir);
    assert!(park(&path, "higit parked main").is_some());
    assert_eq!(stash_list(dir), list_before);
}

#[test]
fn overlapping_unpark_leaves_worktree_clean() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "한 줄\n");
    std::fs::write(dir.join("a.txt"), "맡긴 고침\n").unwrap();
    let path = RepoPath::new(dir);
    let oid = park(&path, "higit parked main").unwrap();
    // 맡긴 사이 같은 줄을 다르게 저장했다.
    commit(dir, "a.txt", "다른 고침\n");

    let err = testutil::backend()
        .run(&path, &GitOp::Unpark { oid: oid.clone() })
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::UnparkConflict, "{}", err.detail());
    assert!(entries(&path).is_empty(), "충돌 표시를 남기지 않는다");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "다른 고침\n"
    );
    // 맡긴 것은 그대로 남아 있다.
    assert!(
        testutil::backend()
            .resolve_ref(&path, oid.as_str())
            .unwrap()
            .is_some()
    );
}

#[test]
fn unpark_refuses_when_new_file_already_exists() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n");
    std::fs::write(dir.join("b.txt"), "맡긴 새 파일\n").unwrap();
    let path = RepoPath::new(dir);
    let oid = park(&path, "higit parked main").unwrap();
    commit(dir, "b.txt", "저장된 b\n");

    let err = testutil::backend()
        .run(&path, &GitOp::Unpark { oid })
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::UnparkConflict, "{}", err.detail());
    assert_eq!(
        std::fs::read_to_string(dir.join("b.txt")).unwrap(),
        "저장된 b\n"
    );
    assert!(entries(&path).is_empty());
}

#[test]
fn unpark_refuses_on_dirty_worktree() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n");
    std::fs::write(dir.join("a.txt"), "맡긴 것\n").unwrap();
    let path = RepoPath::new(dir);
    let oid = park(&path, "higit parked main").unwrap();
    std::fs::write(dir.join("c.txt"), "지금 고친 것\n").unwrap();
    let err = testutil::backend()
        .run(&path, &GitOp::Unpark { oid })
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::WouldOverwrite);
    assert!(dir.join("c.txt").exists());
}

#[test]
fn drop_if_leaves_other_stash_alone() {
    let repo = testutil::init_repo();
    let dir = repo.path();
    commit(dir, "a.txt", "a\n");
    std::fs::write(dir.join("a.txt"), "맡길 것\n").unwrap();
    let path = RepoPath::new(dir);
    let backend = testutil::backend();
    let oid = backend
        .run(
            &path,
            &GitOp::Park {
                message: "higit".into(),
            },
        )
        .unwrap()
        .oid
        .unwrap();
    // 그사이 사용자가 stash를 하나 더 만들었다.
    std::fs::write(dir.join("a.txt"), "사용자 것\n").unwrap();
    testutil::git(dir, &["stash", "push", "-q", "-m", "사용자 stash"]);
    backend.run(&path, &GitOp::StashDropIf { oid }).unwrap();
    let list = stash_list(dir);
    assert!(list.contains("사용자 stash"));
    assert!(list.contains("higit"));
}

#[test]
fn crlf_files_survive_with_autocrlf() {
    // Windows 기본값(core.autocrlf=true)에서도 맡기고 꺼낸 파일이 바이트까지 같다.
    let repo = testutil::init_repo();
    let dir = repo.path();
    testutil::git(dir, &["config", "core.autocrlf", "true"]);
    commit(dir, "a.txt", "하나\r\n둘\r\n");
    std::fs::write(dir.join("a.txt"), "하나\r\n둘 고침\r\n").unwrap();
    std::fs::write(dir.join("새.txt"), "새\r\n").unwrap();
    let path = RepoPath::new(dir);
    let oid = park(&path, "higit parked main").unwrap();
    testutil::backend()
        .run(&path, &GitOp::Unpark { oid })
        .unwrap();
    assert_eq!(
        std::fs::read(dir.join("a.txt")).unwrap(),
        "하나\r\n둘 고침\r\n".as_bytes()
    );
    assert_eq!(
        std::fs::read(dir.join("새.txt")).unwrap(),
        "새\r\n".as_bytes()
    );
}
