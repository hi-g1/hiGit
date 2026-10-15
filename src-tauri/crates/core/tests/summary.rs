//! 시작 화면의 프로젝트 상태 요약(D-031).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod common;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use common::{FakeGit, commit};
use git::{BranchInfo, Change, EntryKind, RepoPath, Status, StatusEntry};
use higit_core::recent::RecentStore;
use higit_core::view::{ErrorCode, ErrorView};
use higit_core::{Core, GitBackend};

fn real(p: &Path) -> std::path::PathBuf {
    platform::normalize_repo_path(p).unwrap()
}

fn entry(path: &str, kind: EntryKind) -> StatusEntry {
    StatusEntry {
        path: path.into(),
        orig_path: None,
        index: Change::Unmerged,
        worktree: Change::Modified,
        kind,
    }
}

/// 최근 목록에 `listed` 경로들이 있는 Core. 상태 요약은 최근 목록에 있는 경로를 받는다.
fn core_listing(git: Arc<dyn GitBackend>, listed: &[&Path]) -> Core {
    let recent = RecentStore::in_memory();
    for p in listed {
        recent.touch(&real(p));
    }
    Core::with_backend_and_recent(git, recent).unwrap()
}

fn core_for(repo: &Path) -> (Core, Arc<FakeGit>) {
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = Some(RepoPath::new(real(repo)));
    let inner = repo.join("안쪽");
    let _ = std::fs::create_dir(&inner);
    (
        core_listing(git.clone() as Arc<dyn GitBackend>, &[repo, &inner]),
        git,
    )
}

fn real_git_core(listed: &Path) -> Core {
    core_listing(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))),
        &[listed],
    )
}

#[test]
fn counts_changed_and_conflicted_files() {
    let dir = tempfile::tempdir().unwrap();
    let (core, git) = core_for(dir.path());
    *git.status.lock().unwrap() = Status {
        branch: BranchInfo {
            oid: Some("abc".into()),
            head: Some("main".into()),
            ..Default::default()
        },
        entries: vec![
            entry("a.txt", EntryKind::Ordinary),
            entry("b.txt", EntryKind::Unmerged),
            entry("c.txt", EntryKind::Untracked),
        ],
    };
    *git.commits.lock().unwrap() = vec![commit(7)];

    let s = core.queries().project_summary(dir.path()).unwrap();
    assert_eq!(s.changed_files, 3);
    assert_eq!(s.conflicted_files, 1);
    assert_eq!(s.branch.name.as_deref(), Some("main"));
    assert_eq!(s.last_commit.unwrap().summary, "저장 7");
}

#[test]
fn repo_without_versions_does_not_read_log() {
    let dir = tempfile::tempdir().unwrap();
    let (core, git) = core_for(dir.path());
    let s = core.queries().project_summary(dir.path()).unwrap();
    assert_eq!(s.last_commit, None);
    assert!(!s.branch.has_commits);
    assert_eq!(git.log_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn missing_folder_and_outer_repo() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = core_for(dir.path());
    let err = core
        .queries()
        .project_summary(&dir.path().join("없음"))
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::FolderNotFound);

    let inner = dir.path().join("안쪽");
    let err = core.queries().project_summary(&inner).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotARepo);
}

fn git_in(dir: &Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=테스트",
            "-c",
            "user.email=t@e",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

#[test]
fn real_git_summary_while_another_task_holds_index_lock() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git_in(p, &["init", "-q"]);
    std::fs::write(p.join("a.txt"), "1").unwrap();
    std::fs::write(p.join("b.txt"), "1").unwrap();
    git_in(p, &["add", "-A"]);
    git_in(p, &["commit", "-q", "-m", "첫 버전"]);
    std::fs::write(p.join("a.txt"), "2").unwrap();
    std::fs::write(p.join("b.txt"), "2").unwrap();

    // 다른 작업(터미널의 git 등)이 index를 잡고 있는 상태를 흉내 낸다.
    let lock = p.join(".git/index.lock");
    std::fs::write(&lock, "").unwrap();
    let index_before = std::fs::metadata(p.join(".git/index"))
        .unwrap()
        .modified()
        .unwrap();

    let core = real_git_core(p);
    let s = core.queries().project_summary(p).unwrap();
    assert_eq!(s.changed_files, 2);
    assert_eq!(s.last_commit.unwrap().summary, "첫 버전");

    // 상태 요약은 잠금을 건드리지 않고 index도 쓰지 않는다(GIT_OPTIONAL_LOCKS=0).
    assert!(lock.exists());
    let index_after = std::fs::metadata(p.join(".git/index"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(index_before, index_after);
}

#[test]
fn real_git_summary_does_not_rewrite_index() {
    // 파일 시각만 바뀌면 보통의 git status는 index를 다시 쓴다(index.lock을 잡는다).
    // 상태 요약은 GIT_OPTIONAL_LOCKS=0이라 index를 쓰지 않아야 한다.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git_in(p, &["init", "-q"]);
    std::fs::write(p.join("a.txt"), "1").unwrap();
    git_in(p, &["add", "-A"]);
    git_in(p, &["commit", "-q", "-m", "첫 버전"]);
    std::thread::sleep(std::time::Duration::from_millis(1200));
    std::fs::write(p.join("a.txt"), "1").unwrap(); // 내용은 같고 시각만 바뀜
    std::thread::sleep(std::time::Duration::from_millis(1200));
    let before = std::fs::metadata(p.join(".git/index"))
        .unwrap()
        .modified()
        .unwrap();

    let core = real_git_core(p);
    core.queries().project_summary(p).unwrap();

    let after = std::fs::metadata(p.join(".git/index"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(before, after, "상태 요약이 index를 다시 썼다");
}

#[test]
fn unlisted_and_unopened_paths_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = Some(RepoPath::new(real(dir.path())));
    let core = core_listing(git as Arc<dyn GitBackend>, &[]);
    let err = core.queries().project_summary(dir.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
}

#[test]
fn route_preview_reads_listed_projects_with_a_limit() {
    let dir = tempfile::tempdir().unwrap();
    let (core, git) = core_for(dir.path());
    *git.status.lock().unwrap() = Status {
        branch: BranchInfo {
            oid: Some(commit(30).oid),
            head: Some("main".into()),
            ..Default::default()
        },
        entries: Vec::new(),
    };
    *git.commits.lock().unwrap() = (1..=30).rev().map(commit).collect();
    // 열지 않았지만 최근 목록에 있다.
    let view = core.queries().route_preview(dir.path(), 12).unwrap();
    assert_eq!(view.current_commits.len(), 12);
    assert_eq!(view.current_more, 1);
    assert_eq!(view.current_commits.last().unwrap().summary, "저장 30");
    // 상한 20.
    let view = core.queries().route_preview(dir.path(), 500).unwrap();
    assert_eq!(view.current_commits.len(), 20);
}

#[test]
fn route_preview_runs_no_git_reads_in_an_untrusted_repo() {
    let dir = tempfile::tempdir().unwrap();
    let (core, git) = core_for(dir.path());
    *git.local_config.lock().unwrap() = vec![("core.fsmonitor".into(), "evil".into())];
    *git.commits.lock().unwrap() = vec![commit(1)];
    let err = core.queries().route_preview(dir.path(), 12).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NeedsTrust);
    assert_eq!(git.log_calls.load(Ordering::SeqCst), 0);

    let other = tempfile::tempdir().unwrap();
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = Some(RepoPath::new(real(other.path())));
    let core = core_listing(git as Arc<dyn GitBackend>, &[]);
    let err = core.queries().route_preview(other.path(), 12).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
}
