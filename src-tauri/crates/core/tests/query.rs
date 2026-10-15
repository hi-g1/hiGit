mod common;

use std::sync::Arc;

use common::{FakeGit, commit};
use git::{BranchInfo, Change, EntryKind, GitVersion, RepoPath, Status, StatusEntry};
use higit_core::view::{ActionResult, ChangeKind, ErrorCode, ErrorView, LogQuery};
use higit_core::{Core, CoreError, GitBackend};
use serde_json::json;

fn core_with(git: FakeGit) -> (Core, Arc<FakeGit>) {
    let git = Arc::new(git);
    let core = Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap();
    (core, git)
}

fn entry(path: &str, index: Change, worktree: Change, kind: EntryKind) -> StatusEntry {
    StatusEntry {
        path: path.into(),
        orig_path: None,
        index,
        worktree,
        kind,
    }
}

#[test]
fn old_git_is_rejected() {
    let git = Arc::new(FakeGit::with_version(GitVersion::new(2, 29, 9)));
    let err = Core::with_backend(git as Arc<dyn GitBackend>)
        .err()
        .unwrap();
    assert!(matches!(err, CoreError::GitTooOld { found } if found == GitVersion::new(2, 29, 9)));
    assert_eq!(ErrorView::from(&err).code, ErrorCode::GitTooOld);
}

#[test]
fn missing_git_binary_is_git_missing() {
    let git = git::CliBackend::new("/없는/git", Arc::new(|_| {}));
    let err = Core::with_backend(Arc::new(git)).err().unwrap();
    assert!(matches!(err, CoreError::GitMissing));
}

#[test]
fn status_is_mapped_for_screen() {
    let (core, git) = core_with(FakeGit::new());
    *git.status.lock().unwrap() = Status {
        branch: BranchInfo {
            oid: Some("abc".into()),
            head: Some("main".into()),
            ..Default::default()
        },
        entries: vec![
            entry(
                "수정.txt",
                Change::Unmodified,
                Change::Modified,
                EntryKind::Ordinary,
            ),
            entry(
                "넣음.txt",
                Change::Added,
                Change::Unmodified,
                EntryKind::Ordinary,
            ),
            entry(
                "지움.txt",
                Change::Unmodified,
                Change::Deleted,
                EntryKind::Ordinary,
            ),
            entry(
                "겹침.txt",
                Change::Unmerged,
                Change::Unmerged,
                EntryKind::Unmerged,
            ),
            entry(
                "새 파일.txt",
                Change::Unmodified,
                Change::Untracked,
                EntryKind::Untracked,
            ),
            StatusEntry {
                orig_path: Some("옛 이름.txt".into()),
                ..entry(
                    "새 이름.txt",
                    Change::Renamed,
                    Change::Unmodified,
                    EntryKind::Renamed,
                )
            },
        ],
    };

    let view = core.queries().status("/repo".as_ref()).unwrap();
    assert_eq!(view.branch.name.as_deref(), Some("main"));
    assert!(view.branch.has_commits);
    let kinds: Vec<_> = view
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.change, f.staged))
        .collect();
    assert_eq!(
        kinds,
        [
            ("수정.txt", ChangeKind::Modified, false),
            ("넣음.txt", ChangeKind::Added, true),
            ("지움.txt", ChangeKind::Deleted, false),
            ("겹침.txt", ChangeKind::Conflicted, true),
            ("새 파일.txt", ChangeKind::Untracked, false),
            ("새 이름.txt", ChangeKind::Renamed, true),
        ]
    );
    assert_eq!(view.files[5].orig_path.as_deref(), Some("옛 이름.txt"));
}

#[test]
fn log_pages_know_if_more_exist() {
    let (core, git) = core_with(FakeGit::new());
    *git.commits.lock().unwrap() = (1..=5).rev().map(commit).collect();

    let first = core
        .queries()
        .log("/repo".as_ref(), LogQuery { skip: 0, limit: 2 })
        .unwrap();
    let summaries: Vec<_> = first.commits.iter().map(|c| c.summary.as_str()).collect();
    assert_eq!(summaries, ["저장 5", "저장 4"]);
    assert!(first.has_more);

    let last = core
        .queries()
        .log("/repo".as_ref(), LogQuery { skip: 4, limit: 2 })
        .unwrap();
    assert_eq!(last.commits.len(), 1);
    assert!(!last.has_more);

    let exact = core
        .queries()
        .log("/repo".as_ref(), LogQuery { skip: 0, limit: 5 })
        .unwrap();
    assert!(!exact.has_more);
}

#[test]
fn open_folder_outside_repo_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = core_with(FakeGit::new());
    let info = core.open_folder(dir.path()).unwrap();
    assert!(info.repo.is_none());
    assert!(!info.folder.is_empty());
}

#[test]
fn open_folder_inside_repo_reports_repo() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("내 프로젝트");
    std::fs::create_dir(&repo_dir).unwrap();
    let git = FakeGit::new();
    *git.repo.lock().unwrap() = Some(RepoPath::new(&repo_dir));
    let (core, _) = core_with(git);

    let info = core.open_folder(&repo_dir).unwrap();
    assert_eq!(info.repo.unwrap().name, "내 프로젝트");
}

#[test]
fn missing_folder_is_folder_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = core_with(FakeGit::new());
    let err = core.open_folder(&dir.path().join("없음")).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::FolderNotFound);
    assert!(ErrorView::from(&err).detail.is_some());
}

#[test]
fn init_through_core() {
    let dir = tempfile::tempdir().unwrap();
    let (core, git) = core_with(FakeGit::new());

    let metas = core.action_metas();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "init");

    let result = core.execute(dir.path(), "init", &json!({}), None).unwrap();
    let ActionResult::RepoCreated { repo } = result;
    assert!(!repo.path.is_empty());
    assert_eq!(git.calls().len(), 1);

    // 이미 저장소가 된 폴더에서는 다시 할 수 없다.
    let err = core
        .execute(dir.path(), "init", &json!({}), None)
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::AlreadyRepo);
}
