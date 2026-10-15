//! 쓰기 작업(GitOp) 테스트. 실제 git으로 임시 저장소에서 확인한다.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::testutil::{self, git};
use crate::{
    Change, CliBackend, EntryKind, FailureKind, GitBackend, GitOp, LogQuery, RepoPath, Status,
};

fn repo_path(p: &Path) -> RepoPath {
    RepoPath::new(p)
}

fn status(p: &Path) -> Status {
    testutil::backend().status(&repo_path(p)).unwrap()
}

fn stage(p: &Path, paths: &[&str]) {
    let paths = paths.iter().map(|s| s.to_string()).collect();
    testutil::backend()
        .run(&repo_path(p), &GitOp::Stage { paths })
        .unwrap();
}

fn unstage(p: &Path, paths: &[&str]) {
    let paths = paths.iter().map(|s| s.to_string()).collect();
    testutil::backend()
        .run(&repo_path(p), &GitOp::Unstage { paths })
        .unwrap();
}

fn commit(p: &Path, message: &str) -> crate::Result<crate::OpOutput> {
    testutil::backend().run(
        &repo_path(p),
        &GitOp::Commit {
            message: message.into(),
        },
    )
}

fn index_of(s: &Status, path: &str) -> Change {
    s.entries
        .iter()
        .find(|e| e.path == path)
        .map(|e| e.index)
        .unwrap_or_else(|| panic!("{path} 없음: {:?}", s.entries))
}

#[test]
fn stage_and_unstage_special_names() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("기존.txt"), "a\n").unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", "처음"]);

    let names = [
        "한글 파일.txt",
        "공백 있는 이름.md",
        "괄호[1].txt",
        "기존.txt",
    ];
    for name in names {
        fs::write(p.join(name), "바뀜\n").unwrap();
    }
    fs::write(p.join("괄호1.txt"), "넣으면 안 됨\n").unwrap();
    stage(p, &names);

    let s = status(p);
    assert_eq!(index_of(&s, "한글 파일.txt"), Change::Added);
    assert_eq!(index_of(&s, "괄호[1].txt"), Change::Added);
    assert_eq!(index_of(&s, "기존.txt"), Change::Modified);
    // `[1]`이 패턴으로 해석되지 않았다(그랬다면 괄호1.txt도 들어갔다).
    let other = s.entries.iter().find(|e| e.path == "괄호1.txt").unwrap();
    assert_eq!(other.kind, EntryKind::Untracked);

    unstage(p, &["한글 파일.txt", "기존.txt"]);
    let s = status(p);
    let new = s
        .entries
        .iter()
        .find(|e| e.path == "한글 파일.txt")
        .unwrap();
    assert_eq!(new.kind, EntryKind::Untracked);
    assert_eq!(index_of(&s, "기존.txt"), Change::Unmodified);
    // 작업 폴더 내용은 그대로
    assert_eq!(fs::read_to_string(p.join("기존.txt")).unwrap(), "바뀜\n");
}

#[test]
fn stage_deleted_file() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("지울.txt"), "a\n").unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", "처음"]);
    fs::remove_file(p.join("지울.txt")).unwrap();

    stage(p, &["지울.txt"]);
    assert_eq!(index_of(&status(p), "지울.txt"), Change::Deleted);
}

#[test]
fn stage_many_files() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let names: Vec<String> = (0..1000).map(|i| format!("폴더/파일 {i:04}.txt")).collect();
    fs::create_dir(p.join("폴더")).unwrap();
    for name in &names {
        fs::write(p.join(name), "x").unwrap();
    }
    testutil::backend()
        .run(
            &repo_path(p),
            &GitOp::Stage {
                paths: names.clone(),
            },
        )
        .unwrap();
    let s = status(p);
    assert_eq!(
        s.entries
            .iter()
            .filter(|e| e.index == Change::Added)
            .count(),
        1000
    );
}

#[test]
fn unstage_in_repo_without_commits() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("첫 파일.txt"), "a\n").unwrap();
    stage(p, &["첫 파일.txt"]);
    assert_eq!(index_of(&status(p), "첫 파일.txt"), Change::Added);

    unstage(p, &["첫 파일.txt"]);
    let s = status(p);
    assert_eq!(s.entries[0].kind, EntryKind::Untracked);
    assert!(p.join("첫 파일.txt").exists());
}

#[test]
fn commit_saves_korean_multiline_message() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("a.txt"), "a\n").unwrap();
    stage(p, &["a.txt"]);
    commit(p, "로그인 화면 \"추가\"\n\n자세한 설명 줄").unwrap();

    let log = testutil::backend()
        .log(&repo_path(p), &LogQuery::default())
        .unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].summary, "로그인 화면 \"추가\"");
    let body = git(p, &["log", "-1", "--format=%B"]);
    assert!(body.contains("자세한 설명 줄"), "{body}");
}

#[test]
fn commit_without_staged_changes() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("a.txt"), "a\n").unwrap();
    let err = commit(p, "빈 저장").unwrap_err();
    assert_eq!(err.kind(), FailureKind::NothingToCommit, "{}", err.detail());
}

#[test]
fn leftover_index_lock_is_locked() {
    let repo = testutil::init_repo();
    let p = repo.path();
    fs::write(p.join("a.txt"), "a\n").unwrap();
    fs::write(p.join(".git/index.lock"), "").unwrap();
    let err = testutil::backend()
        .run(
            &repo_path(p),
            &GitOp::Stage {
                paths: vec!["a.txt".into()],
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::Locked, "{}", err.detail());
}

#[test]
fn commit_without_identity() {
    let repo = testutil::init_repo();
    let p = repo.path();
    git(p, &["config", "--unset", "user.name"]);
    git(p, &["config", "--unset", "user.email"]);
    // 이름·이메일을 추측하지 않게 한다(사용자 PC의 호스트 이름 등).
    git(p, &["config", "user.useConfigOnly", "true"]);
    fs::write(p.join("a.txt"), "a\n").unwrap();
    stage(p, &["a.txt"]);

    let backend = CliBackend::new(
        "git",
        Arc::new(|cmd: &mut std::process::Command| {
            testutil::isolate(cmd);
            for var in [
                "GIT_AUTHOR_NAME",
                "GIT_AUTHOR_EMAIL",
                "GIT_COMMITTER_NAME",
                "GIT_COMMITTER_EMAIL",
                "EMAIL",
            ] {
                cmd.env_remove(var);
            }
        }),
    );
    let err = backend
        .run(
            &repo_path(p),
            &GitOp::Commit {
                message: "저장".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::IdentityMissing, "{}", err.detail());
}
