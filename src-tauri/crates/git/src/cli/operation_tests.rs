//! 마지막 버전 고치기·복사·되돌리기의 시작·이어가기·건너뛰기·그만두기(D-101). 실제 git으로.

use std::fs;
use std::path::Path;

use crate::testutil::{self, git};
use crate::{GitBackend, GitOp, Oid, OperationKind, RepoPath};

fn commit(p: &Path, file: &str, text: &str) -> Oid {
    fs::write(p.join(file), text).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", text]);
    oid(p, "HEAD")
}

fn oid(p: &Path, rev: &str) -> Oid {
    Oid::parse(git(p, &["rev-parse", rev]).trim()).unwrap()
}

fn run(p: &Path, op: &GitOp) -> crate::Result<crate::OpOutput> {
    testutil::backend().run(&RepoPath::new(p), op)
}

/// 충돌 항목의 stage들.
fn stages(p: &Path) -> Vec<u8> {
    let mut s: Vec<u8> = testutil::backend()
        .index_entries(&RepoPath::new(p))
        .unwrap()
        .iter()
        .map(|e| e.stage)
        .filter(|s| *s > 0)
        .collect();
    s.sort_unstable();
    s
}

#[test]
fn amend_changes_the_message_and_takes_staged_files() {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "처음");
    let before = commit(p, "b.txt", "고칠 버전");
    fs::write(p.join("c.txt"), "더함").unwrap();
    git(p, &["add", "c.txt"]);
    // 저장소 설정이 # 줄을 지우게 해도 그대로 남는다.
    git(p, &["config", "commit.cleanup", "strip"]);
    run(
        p,
        &GitOp::Amend {
            message: Some("고친 이름\n\n# 샵 줄도 남음".into()),
        },
    )
    .unwrap();
    let after = oid(p, "HEAD");
    assert_ne!(after, before);
    assert_eq!(
        git(p, &["log", "-1", "--format=%B"]).trim_end(),
        "고친 이름\n\n# 샵 줄도 남음"
    );
    assert!(git(p, &["show", "--name-only", "--format=", "HEAD"]).contains("c.txt"));
    // 메시지 없이: 내용만.
    fs::write(p.join("d.txt"), "또").unwrap();
    git(p, &["add", "d.txt"]);
    run(p, &GitOp::Amend { message: None }).unwrap();
    assert!(git(p, &["log", "-1", "--format=%B"]).starts_with("고친 이름"));
}

#[test]
fn cherry_pick_copies_and_stops_on_conflict_without_cleaning() {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "원래\n");
    git(p, &["switch", "-q", "-c", "곁"]);
    let clean = commit(p, "b.txt", "곁 파일\n");
    let clash = commit(p, "a.txt", "곁에서 고침\n");
    git(p, &["switch", "-q", "main"]);
    commit(p, "a.txt", "메인에서 고침\n");
    // 겹치지 않으면 새 버전.
    run(p, &GitOp::CherryPickStart { oid: clean }).unwrap();
    assert!(p.join("b.txt").exists());
    // 겹치면 멈추고 상태를 남긴다(stage 2 = 지금, 3 = 복사할 버전).
    assert!(run(p, &GitOp::CherryPickStart { oid: clash }).is_err());
    assert!(p.join(".git/CHERRY_PICK_HEAD").exists());
    assert_eq!(stages(p), [1, 2, 3]);
    // 풀지 않고 이어가면 실패하고 상태는 그대로.
    assert!(
        run(
            p,
            &GitOp::OperationContinue {
                kind: OperationKind::CherryPick
            }
        )
        .is_err()
    );
    assert!(p.join(".git/CHERRY_PICK_HEAD").exists());
    // 풀고 이어가면 새 버전.
    fs::write(p.join("a.txt"), "합친 모습\n").unwrap();
    git(p, &["add", "a.txt"]);
    run(
        p,
        &GitOp::OperationContinue {
            kind: OperationKind::CherryPick,
        },
    )
    .unwrap();
    assert!(!p.join(".git/CHERRY_PICK_HEAD").exists());
    assert_eq!(fs::read_to_string(p.join("a.txt")).unwrap(), "합친 모습\n");
}

#[test]
fn an_empty_copy_is_skipped_and_abort_goes_back() {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "원래\n");
    git(p, &["switch", "-q", "-c", "곁"]);
    let same = commit(p, "a.txt", "같은 내용\n");
    git(p, &["switch", "-q", "main"]);
    commit(p, "a.txt", "같은 내용\n");
    let head = oid(p, "HEAD");
    // 이미 같은 내용이라 비어 버리면 멈춘 상태가 남는다(충돌 없음). 건너뛰면 정리된다.
    assert!(run(p, &GitOp::CherryPickStart { oid: same.clone() }).is_err());
    assert!(p.join(".git/CHERRY_PICK_HEAD").exists());
    assert!(stages(p).is_empty());
    run(
        p,
        &GitOp::OperationSkip {
            kind: OperationKind::CherryPick,
        },
    )
    .unwrap();
    assert!(!p.join(".git/CHERRY_PICK_HEAD").exists());
    assert_eq!(oid(p, "HEAD"), head);
}

#[test]
fn revert_start_stops_on_conflict_and_abort_restores() {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "하나\n");
    let target = commit(p, "a.txt", "둘\n");
    commit(p, "a.txt", "셋\n");
    let head = oid(p, "HEAD");
    assert!(
        run(
            p,
            &GitOp::RevertStart {
                oid: target,
                mainline: None
            }
        )
        .is_err()
    );
    assert!(p.join(".git/REVERT_HEAD").exists());
    run(
        p,
        &GitOp::OperationAbort {
            kind: OperationKind::Revert,
        },
    )
    .unwrap();
    assert!(!p.join(".git/REVERT_HEAD").exists());
    assert_eq!(oid(p, "HEAD"), head);
    assert_eq!(fs::read_to_string(p.join("a.txt")).unwrap(), "셋\n");
    // 겹치지 않는 되돌리기는 바로 새 버전.
    let other = commit(p, "b.txt", "따로\n");
    run(
        p,
        &GitOp::RevertStart {
            oid: other,
            mainline: None,
        },
    )
    .unwrap();
    assert!(!p.join("b.txt").exists());
}
