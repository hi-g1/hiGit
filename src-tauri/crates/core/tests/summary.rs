//! 시작 화면의 프로젝트 상태 요약(D-031).
mod common;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use common::{FakeGit, commit};
use git::{BranchInfo, Change, EntryKind, RepoPath, Status, StatusEntry};
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

fn core_for(repo: &Path) -> (Core, Arc<FakeGit>) {
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = Some(RepoPath::new(real(repo)));
    (
        Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap(),
        git,
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
    std::fs::create_dir(&inner).unwrap();
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

    let core = Core::with_backend(Arc::new(git::CliBackend::new("git", Arc::new(|_| {})))).unwrap();
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

    let core = Core::with_backend(Arc::new(git::CliBackend::new("git", Arc::new(|_| {})))).unwrap();
    core.queries().project_summary(p).unwrap();

    let after = std::fs::metadata(p.join(".git/index"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(before, after, "상태 요약이 index를 다시 썼다");
}
