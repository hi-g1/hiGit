#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use common::{FakeGit, commit};
use git::{BranchInfo, Change, EntryKind, GitVersion, RepoPath, Status, StatusEntry};
use higit_core::view::{ActionInput, ActionResult, ChangeKind, ErrorCode, ErrorView, LogQuery};
use higit_core::view::{CommitInput, OpenHow, PathsInput, ProjectOpenResult};
use higit_core::{Core, CoreError, GitBackend};

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

    let dir = tempfile::tempdir().unwrap();
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    let view = core.queries().status(dir.path()).unwrap();
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
    git.status.lock().unwrap().branch.oid = Some(commit(5).oid);
    let dir = tempfile::tempdir().unwrap();
    core.open_project(dir.path(), OpenHow::Picked).unwrap();

    let first = core
        .queries()
        .log(
            dir.path(),
            LogQuery {
                skip: 0,
                limit: 2,
                ..Default::default()
            },
        )
        .unwrap();
    let summaries: Vec<_> = first.commits.iter().map(|c| c.summary.as_str()).collect();
    assert_eq!(summaries, ["저장 5", "저장 4"]);
    assert!(first.has_more);

    let last = core
        .queries()
        .log(
            dir.path(),
            LogQuery {
                skip: 4,
                limit: 2,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(last.commits.len(), 1);
    assert!(!last.has_more);

    let exact = core
        .queries()
        .log(
            dir.path(),
            LogQuery {
                skip: 0,
                limit: 5,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!exact.has_more);
}

#[test]
fn queries_reject_missing_folder() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = core_with(FakeGit::new());
    let err = core.queries().status(&dir.path().join("없음")).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::FolderNotFound);
}

#[test]
fn open_project_outside_repo_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = core_with(FakeGit::new());
    let result = core.open_project(dir.path(), OpenHow::Picked).unwrap();
    let ProjectOpenResult::NotARepo { folder } = result else {
        panic!("{result:?}")
    };
    assert!(!folder.is_empty());
}

#[test]
fn open_project_inside_repo_reports_repo() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("내 프로젝트");
    std::fs::create_dir(&repo_dir).unwrap();
    let git = FakeGit::new();
    *git.repo.lock().unwrap() = Some(RepoPath::new(&repo_dir));
    let (core, _) = core_with(git);

    let result = core.open_project(&repo_dir, OpenHow::Picked).unwrap();
    let ProjectOpenResult::Opened { repo } = result else {
        panic!("{result:?}")
    };
    assert_eq!(repo.name, "내 프로젝트");
}

#[test]
fn open_project_missing_folder() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = core_with(FakeGit::new());
    let result = core
        .open_project(&dir.path().join("없음"), OpenHow::Picked)
        .unwrap();
    assert_eq!(result, ProjectOpenResult::Missing);
}

#[test]
fn init_through_core() {
    let dir = tempfile::tempdir().unwrap();
    let (core, git) = core_with(FakeGit::new());

    let metas = core.action_metas();
    let ids: Vec<_> = metas.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(&ids[..4], ["init", "stage", "unstage", "commit"]);
    assert_eq!(ids.len(), higit_core::actions::all_metas().len());

    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    let result = core.execute(dir.path(), &ActionInput::Init, None).unwrap();
    let ActionResult::RepoCreated { repo } = result else {
        panic!("{result:?}")
    };
    assert!(!repo.path.is_empty());
    assert_eq!(git.calls().len(), 1);

    // 이미 저장소가 된 폴더에서는 다시 할 수 없다.
    let err = core
        .execute(dir.path(), &ActionInput::Init, None)
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::AlreadyRepo);
}

#[test]
fn action_input_tags_match_registry() {
    // 새 변형을 더하면 이 match가 컴파일되지 않아 아래 목록을 고치게 된다.
    fn every_variant(input: &ActionInput) {
        match input {
            ActionInput::Init
            | ActionInput::Clone(_)
            | ActionInput::PrCreate(_)
            | ActionInput::PrComment(_)
            | ActionInput::PrMerge(_)
            | ActionInput::PrClose(_)
            | ActionInput::PrReopen(_)
            | ActionInput::IdentitySet(_)
            | ActionInput::Stage(_)
            | ActionInput::Unstage(_)
            | ActionInput::Commit(_)
            | ActionInput::Ignore(_)
            | ActionInput::Merge(_)
            | ActionInput::MergeInto(_)
            | ActionInput::ConflictFinish(_)
            | ActionInput::ConflictAbort
            | ActionInput::Stash
            | ActionInput::TakeOut(_)
            | ActionInput::Discard(_)
            | ActionInput::Restore(_)
            | ActionInput::ClearLock
            | ActionInput::RecoverParked
            | ActionInput::Switch(_)
            | ActionInput::BranchCreate(_)
            | ActionInput::Fetch(_)
            | ActionInput::Pull
            | ActionInput::Push
            | ActionInput::Sync(_)
            | ActionInput::Amend(_)
            | ActionInput::Revert(_)
            | ActionInput::CherryPick(_)
            | ActionInput::Reset(_)
            | ActionInput::RebaseI(_)
            | ActionInput::Tag(_)
            | ActionInput::TagDelete(_)
            | ActionInput::BranchDelete(_)
            | ActionInput::BranchRename(_)
            | ActionInput::ForcePush(_)
            | ActionInput::RemoteBranchDelete(_)
            | ActionInput::RemoteBranchRename(_)
            | ActionInput::TagPush(_)
            | ActionInput::Publish(_) => {}
        }
    }
    let paths = PathsInput {
        paths: vec!["a.txt".into()],
    };
    let samples = [
        ActionInput::Init,
        ActionInput::Clone(higit_core::view::CloneInput {
            url: "https://github.com/octo/hello.git".into(),
            name: "hello".into(),
            accept_old_git: false,
        }),
        ActionInput::PrCreate(higit_core::view::PrCreateInput {
            title: "제목".into(),
            body: String::new(),
            base: "main".into(),
            draft: false,
        }),
        ActionInput::PrComment(higit_core::view::PrCommentInput {
            number: 1,
            body: "댓글".into(),
        }),
        ActionInput::PrMerge(higit_core::view::PrMergeInput {
            number: 1,
            method: higit_core::view::MergeMethodView::Merge,
            expect_head: "a".repeat(40),
        }),
        ActionInput::PrClose(higit_core::view::PrNumberInput { number: 1 }),
        ActionInput::PrReopen(higit_core::view::PrNumberInput { number: 1 }),
        ActionInput::IdentitySet(higit_core::view::IdentityInput {
            name: "이름".into(),
            email: "a@example.com".into(),
        }),
        ActionInput::Stage(paths.clone()),
        ActionInput::Unstage(paths.clone()),
        ActionInput::Ignore(paths),
        ActionInput::Merge(higit_core::view::MergeInput {
            branch: "main".into(),
            locale: "ko".into(),
        }),
        ActionInput::MergeInto(higit_core::view::MergeIntoInput {
            branch: "main".into(),
        }),
        ActionInput::ConflictFinish(higit_core::view::ConflictFinishInput {
            resolutions: vec![],
            locale: "ko".into(),
        }),
        ActionInput::ConflictAbort,
        ActionInput::Stash,
        ActionInput::TakeOut(higit_core::view::TakeOutInput {
            source: higit_core::view::TakeOutSource::Stash {
                oid: "a".repeat(40),
            },
        }),
        ActionInput::Discard(PathsInput {
            paths: vec!["a.txt".into()],
        }),
        ActionInput::Restore(higit_core::view::RestoreInput {
            oid: "a".repeat(40),
        }),
        ActionInput::ClearLock,
        ActionInput::RecoverParked,
        ActionInput::Commit(CommitInput {
            message: "저장".into(),
            stage_all: false,
        }),
        ActionInput::Switch(higit_core::view::SwitchInput {
            branch: "main".into(),
            remote: None,
        }),
        ActionInput::BranchCreate(higit_core::view::BranchCreateInput {
            name: "새것".into(),
            from: None,
        }),
        ActionInput::Fetch(higit_core::view::FetchInput { background: true }),
        ActionInput::Pull,
        ActionInput::Push,
        ActionInput::Sync(higit_core::view::SyncInput {
            locale: "ko".into(),
            merge_anyway: false,
        }),
        ActionInput::Amend(higit_core::view::AmendInput {
            expect_head: "a".repeat(40),
            message: None,
        }),
        ActionInput::Revert(higit_core::view::OidInput {
            oid: "a".repeat(40),
        }),
        ActionInput::CherryPick(higit_core::view::OidInput {
            oid: "a".repeat(40),
        }),
        ActionInput::Reset(higit_core::view::OidInput {
            oid: "a".repeat(40),
        }),
        ActionInput::RebaseI(higit_core::view::RewriteInput { steps: vec![] }),
        ActionInput::Tag(higit_core::view::TagInput {
            oid: "a".repeat(40),
            name: "v1".into(),
        }),
        ActionInput::TagDelete(higit_core::view::TagDeleteInput {
            name: "v1".into(),
            oid: "a".repeat(40),
        }),
        ActionInput::BranchDelete(higit_core::view::BranchNameInput { name: "옆".into() }),
        ActionInput::BranchRename(higit_core::view::BranchRenameInput {
            from: "옆".into(),
            to: "곁".into(),
        }),
        ActionInput::ForcePush(higit_core::view::ForcePushInput {
            remote: "origin".into(),
            branch: "기능".into(),
            expect_remote: "a".repeat(40),
            expect_head: "b".repeat(40),
        }),
        ActionInput::RemoteBranchDelete(higit_core::view::RemoteBranchInput {
            remote: "origin".into(),
            branch: "옆".into(),
            expect_tip: "a".repeat(40),
        }),
        ActionInput::RemoteBranchRename(higit_core::view::RemoteBranchRenameInput {
            remote: "origin".into(),
            from: "옆".into(),
            to: "곁".into(),
            expect_tip: "a".repeat(40),
        }),
        ActionInput::TagPush(higit_core::view::TagPushInput { name: "v1".into() }),
        ActionInput::Publish(higit_core::view::PublishInput {
            name: "hello".into(),
            description: None,
            visibility: higit_core::view::Visibility::Private,
            acknowledge_public: false,
        }),
    ];
    samples.iter().for_each(every_variant);

    for input in &samples {
        let json = serde_json::to_value(input).unwrap();
        assert_eq!(
            json["id"],
            input.action_id(),
            "serde 태그와 action_id가 다르다"
        );
    }
    let mut tags: Vec<_> = samples.iter().map(ActionInput::action_id).collect();
    let mut registered: Vec<_> = higit_core::actions::all_metas()
        .iter()
        .map(|m| m.id)
        .collect();
    tags.sort();
    registered.sort();
    assert_eq!(
        tags, registered,
        "등록된 액션과 입력 변형이 일대일이어야 한다"
    );
}

#[test]
fn diff_lines_get_line_numbers() {
    use git::{DiffLine, FileDiff, Hunk, LineKind};
    use higit_core::view::{DiffRequest, LineKindView};
    let line = |kind, text: &str| DiffLine {
        kind,
        text: text.into(),
        no_newline_at_end: false,
    };
    let (core, git) = core_with(FakeGit::new());
    *git.file_diff.lock().unwrap() = FileDiff {
        binary: false,
        truncated: false,
        hunks: vec![Hunk {
            old_start: 10,
            old_len: 3,
            new_start: 10,
            new_len: 3,
            section: String::new(),
            lines: vec![
                line(LineKind::Context, "가"),
                line(LineKind::Removed, "나"),
                line(LineKind::Added, "나나"),
                line(LineKind::Context, "다"),
            ],
        }],
    };
    let dir = tempfile::tempdir().unwrap();
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    let view = core
        .queries()
        .diff(
            dir.path(),
            &DiffRequest {
                path: "a.txt".into(),
                orig_path: None,
                untracked: false,
                commit: None,
                full_context: false,
            },
        )
        .unwrap();
    let numbers: Vec<_> = view.hunks[0]
        .lines
        .iter()
        .map(|l| (l.kind, l.old_line, l.new_line))
        .collect();
    assert_eq!(
        numbers,
        [
            (LineKindView::Context, Some(10), Some(10)),
            (LineKindView::Removed, Some(11), None),
            (LineKindView::Added, None, Some(11)),
            (LineKindView::Context, Some(12), Some(12)),
        ]
    );
}

#[test]
fn git_failures_map_to_error_codes() {
    let failed = |stderr: &str| {
        CoreError::Git(git::GitError::Failed {
            args: vec![],
            code: Some(1),
            stdout: String::new(),
            stderr: stderr.into(),
        })
    };
    let cases = [
        (
            "hint: Updates were rejected because",
            ErrorCode::PushRejected,
        ),
        (
            "Host key verification failed.",
            ErrorCode::SshHostUnverified,
        ),
        (
            "git@github.com: Permission denied (publickey).",
            ErrorCode::SshAuthFailed,
        ),
        ("fatal: 처음 보는 오류", ErrorCode::GitFailed),
    ];
    for (stderr, code) in cases {
        let view = ErrorView::from(&failed(stderr));
        assert_eq!(view.code, code, "{stderr}");
        // 이메일 꼴(`git@github.com`)은 가려진다(D-040).
        let want = stderr.replace("git@github.com", "***@github.com");
        assert_eq!(view.detail.as_deref(), Some(want.as_str()));
    }
}

#[test]
fn diff_rejects_paths_outside_repo() {
    use higit_core::view::DiffRequest;
    let base = tempfile::tempdir().unwrap();
    let repo = base.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    std::fs::write(base.path().join("비밀.txt"), "저장소 밖 내용\n").unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap();
    let core = Core::with_backend(Arc::new(git::CliBackend::new("git", Arc::new(|_| {})))).unwrap();
    core.open_project(&repo, OpenHow::Picked).unwrap();

    for (path, orig) in [("../비밀.txt", None), ("a.txt", Some("../비밀.txt"))] {
        let err = core
            .queries()
            .diff(
                &repo,
                &DiffRequest {
                    path: path.into(),
                    orig_path: orig.map(str::to_string),
                    untracked: true,
                    commit: None,
                    full_context: false,
                },
            )
            .unwrap_err();
        assert_eq!(
            ErrorView::from(&err).code,
            ErrorCode::InvalidInput,
            "{path}"
        );
    }
}

#[test]
fn diff_shows_outside_link_target_without_reading_it() {
    use higit_core::view::DiffRequest;
    let base = tempfile::tempdir().unwrap();
    let repo = base.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    let secret = base.path().join("비밀.txt");
    std::fs::write(&secret, "저장소 밖 내용\n").unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap();
    // 저장소 안의 링크가 밖의 파일을 가리킨다. 내용은 읽지 않고 대상만 보여준다(D-052).
    if let Err(e) = platform::symlink_file(&secret, &repo.join("링크.txt")) {
        eprintln!("이 환경에서는 심볼릭 링크를 만들 수 없어 시험을 건너뜁니다: {e}");
        return;
    }
    let core = Core::with_backend(Arc::new(git::CliBackend::new("git", Arc::new(|_| {})))).unwrap();
    core.open_project(&repo, OpenHow::Picked).unwrap();
    for untracked in [true, false] {
        let view = core
            .queries()
            .diff(
                &repo,
                &DiffRequest {
                    path: "링크.txt".into(),
                    orig_path: None,
                    untracked,
                    commit: None,
                    full_context: false,
                },
            )
            .unwrap();
        assert!(view.hunks.is_empty());
        assert!(view.link.as_deref().unwrap().ends_with("비밀.txt"));
    }
}

#[test]
fn diff_rejects_files_under_outside_folder_link() {
    use higit_core::view::DiffRequest;
    let base = tempfile::tempdir().unwrap();
    let repo = base.path().join("repo");
    let outside = base.path().join("밖 폴더");
    std::fs::create_dir(&repo).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("비밀.txt"), "밖\n").unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap();
    if let Err(e) = platform::symlink_dir(&outside, &repo.join("링크폴더")) {
        eprintln!("이 환경에서는 폴더 링크를 만들 수 없어 시험을 건너뜁니다: {e}");
        return;
    }
    let core = Core::with_backend(Arc::new(git::CliBackend::new("git", Arc::new(|_| {})))).unwrap();
    core.open_project(&repo, OpenHow::Picked).unwrap();
    for path in ["링크폴더/비밀.txt", "링크폴더"] {
        let result = core.queries().diff(
            &repo,
            &DiffRequest {
                path: path.into(),
                orig_path: None,
                untracked: true,
                commit: None,
                full_context: false,
            },
        );
        match (path, result) {
            // 폴더 링크 자체는 대상만 보여준다.
            ("링크폴더", Ok(view)) => assert!(view.link.is_some()),
            (_, Err(err)) => assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput),
            (p, Ok(view)) => panic!("{p}가 막히지 않았다: {view:?}"),
        }
    }
}

#[test]
fn diff_rejects_absolute_and_drive_paths() {
    use higit_core::view::DiffRequest;
    let (core, _git) = core_with(FakeGit::new());
    let dir = tempfile::tempdir().unwrap();
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    for path in ["/etc/passwd", "C:/x", "c:\\x", "../a"] {
        let err = core
            .queries()
            .diff(
                dir.path(),
                &DiffRequest {
                    path: path.into(),
                    orig_path: None,
                    untracked: true,
                    commit: None,
                    full_context: false,
                },
            )
            .unwrap_err();
        assert_eq!(
            ErrorView::from(&err).code,
            ErrorCode::InvalidInput,
            "{path}"
        );
    }
}

#[test]
fn diff_allows_symlink_inside_repo() {
    use higit_core::view::DiffRequest;
    let base = tempfile::tempdir().unwrap();
    let repo = base.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    std::fs::write(repo.join("원본.txt"), "안쪽 내용\n").unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap();
    if let Err(e) = platform::symlink_file(&repo.join("원본.txt"), &repo.join("링크.txt")) {
        eprintln!("이 환경에서는 심볼릭 링크를 만들 수 없어 시험을 건너뜁니다: {e}");
        return;
    }
    let core = Core::with_backend(Arc::new(git::CliBackend::new("git", Arc::new(|_| {})))).unwrap();
    core.open_project(&repo, OpenHow::Picked).unwrap();
    let request = DiffRequest {
        path: "링크.txt".into(),
        orig_path: None,
        untracked: true,
        commit: None,
        full_context: false,
    };
    let view = core.queries().diff(&repo, &request).unwrap();
    assert!(view.link.is_some(), "안쪽 링크도 대상만 보여준다");
}

#[test]
fn unopened_paths_are_rejected() {
    let (core, _git) = core_with(FakeGit::new());
    let dir = tempfile::tempdir().unwrap();
    let err = core.queries().status(dir.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    let err = core
        .queries()
        .log(
            dir.path(),
            LogQuery {
                skip: 0,
                limit: 1,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    let err = core
        .execute(dir.path(), &ActionInput::Init, None)
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NotOpened);
    // 나머지 조회·액션·감시도 열지 않은 경로를 받지 않는다(D-037).
    let not_opened = |r: Result<(), higit_core::CoreError>| {
        assert_eq!(ErrorView::from(&r.unwrap_err()).code, ErrorCode::NotOpened);
    };
    let q = core.queries();
    let request = higit_core::view::DiffRequest {
        path: "a.txt".into(),
        orig_path: None,
        untracked: false,
        commit: None,
        full_context: false,
    };
    not_opened(q.diff(dir.path(), &request).map(drop));
    not_opened(q.branches(dir.path()).map(drop));
    not_opened(q.check_branch_name(dir.path(), "새것").map(drop));
    not_opened(q.home_map(dir.path()).map(drop));
    not_opened(core.preview(dir.path(), &ActionInput::Init).map(drop));
    not_opened(core.undo(dir.path()).map(drop));
    not_opened(core.redo(dir.path()).map(drop));
    not_opened(core.watch(dir.path(), Arc::new(|_| {})));
    not_opened(core.recovery(dir.path()).map(drop));
    not_opened(q.stashes(dir.path()).map(drop));
    not_opened(q.merge_hint(dir.path()).map(drop));
    not_opened(q.conflict_state(dir.path()).map(drop));
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    assert!(core.queries().status(dir.path()).is_ok());
}

#[test]
fn log_page_size_is_capped() {
    let (core, git) = core_with(FakeGit::new());
    *git.commits.lock().unwrap() = (1..=600).rev().map(commit).collect();
    git.status.lock().unwrap().branch.oid = Some(commit(600).oid);
    let dir = tempfile::tempdir().unwrap();
    core.open_project(dir.path(), OpenHow::Picked).unwrap();
    let page = core
        .queries()
        .log(
            dir.path(),
            LogQuery {
                skip: 0,
                limit: u32::MAX,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(page.commits.len(), higit_core::MAX_LOG_PAGE as usize);
    assert!(page.has_more);
}

#[test]
fn error_detail_hides_credentials() {
    let err = CoreError::Git(git::GitError::Failed {
        args: vec![],
        code: Some(128),
        stdout: String::new(),
        stderr: "fatal: unable to access 'https://kim:ghp_abcdefghijklmnop1234@github.com/a/b.git/': 403".into(),
    });
    let detail = ErrorView::from(&err).detail.unwrap();
    assert!(!detail.contains("kim"), "{detail}");
    assert!(!detail.contains("ghp_abc"), "{detail}");
    assert!(detail.contains("https://***@github.com"), "{detail}");
}

mod secrets {
    use std::path::Path;
    use std::sync::Arc;

    use higit_core::view::{OpenHow, SecretCheckRequest, SecretRule};
    use higit_core::{Core, GitBackend};

    fn git(dir: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .args(["-c", "core.autocrlf=false"])
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .current_dir(dir)
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    fn opened(dir: &Path) -> Core {
        let core = Core::with_backend(
            Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
        )
        .unwrap();
        core.open_project(dir, OpenHow::Picked).unwrap();
        core
    }

    #[test]
    fn finds_names_tokens_and_keys_without_the_secret_text() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        std::fs::write(dir.join(".env"), "PASSWORD=hunter2\n").unwrap();
        std::fs::write(dir.join(".env.example"), "PASSWORD=\n").unwrap();
        let token = format!("ghp_{}", "Z9".repeat(18));
        std::fs::write(
            dir.join("설정 파일.ts"),
            format!("// 설명\nconst t = \"{token}\";\n"),
        )
        .unwrap();
        std::fs::write(dir.join("평범.txt"), "그냥 글\n").unwrap();
        let core = opened(dir);

        // 넣은 파일이 없고 모두 넣고 저장이면 작업 폴더의 고친 파일을 본다.
        let view = core
            .queries()
            .secret_check(dir, &SecretCheckRequest { stage_all: true })
            .unwrap();
        let mut found: Vec<(String, SecretRule, Option<u32>)> = view
            .findings
            .iter()
            .map(|f| (f.path.clone(), f.rule, f.line))
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            found,
            [
                (".env".to_string(), SecretRule::EnvFile, None),
                ("설정 파일.ts".to_string(), SecretRule::GithubToken, Some(2)),
            ]
        );
        let json = serde_json::to_string(&view).unwrap();
        assert!(
            !json.contains(&token) && !json.contains("hunter2"),
            "{json}"
        );
        // 모두 넣지 않는 저장이면 넣은 파일이 없으니 볼 것이 없다.
        let view = core
            .queries()
            .secret_check(dir, &SecretCheckRequest { stage_all: false })
            .unwrap();
        assert!(view.findings.is_empty());
    }

    #[test]
    fn staged_content_is_what_gets_checked() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        let key = "-----BEGIN RSA PRIVATE KEY-----\nabc\n";
        std::fs::write(dir.join("a.txt"), key).unwrap();
        git(dir, &["add", "a.txt"]);
        // 작업 폴더에서는 지웠지만 넣은 내용에는 키가 있다.
        std::fs::write(dir.join("a.txt"), "깨끗함\n").unwrap();
        std::fs::write(dir.join(".env"), "x\n").unwrap();
        let core = opened(dir);
        let view = core
            .queries()
            .secret_check(dir, &SecretCheckRequest { stage_all: true })
            .unwrap();
        assert_eq!(view.findings.len(), 1, "{:?}", view.findings);
        assert_eq!(view.findings[0].path, "a.txt");
        assert_eq!(view.findings[0].rule, SecretRule::PrivateKey);
        assert_eq!(view.findings[0].line, Some(1));
    }

    fn rules(core: &Core, dir: &Path, stage_all: bool) -> Vec<(String, SecretRule)> {
        let mut found: Vec<(String, SecretRule)> = core
            .queries()
            .secret_check(dir, &SecretCheckRequest { stage_all })
            .unwrap()
            .findings
            .into_iter()
            .map(|f| (f.path, f.rule))
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found
    }

    #[test]
    fn new_archives_and_executables_are_noticed_once() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        // 이미 추적 중인 압축 파일을 고친 것은 알리지 않는다.
        std::fs::write(dir.join("fixture.zip"), b"PK\x03\x04old").unwrap();
        git(dir, &["add", "fixture.zip"]);
        git(dir, &["commit", "-q", "-m", "자료"]);
        std::fs::write(dir.join("fixture.zip"), b"PK\x03\x04new").unwrap();
        std::fs::write(dir.join("home.zip"), b"PK\x03\x04data").unwrap();
        std::fs::write(dir.join("tool"), b"\x7fELF\x02\x01\x01").unwrap();
        std::fs::write(dir.join("보고서.docx"), b"PK\x03\x04doc").unwrap();
        // 이름만 압축 파일인 글자 파일의 비밀은 놓치지 않는다(비밀 규칙이 이긴다).
        let token = format!("ghp_{}", "Q7".repeat(18));
        std::fs::write(dir.join("notes.zip"), format!("token = {token}\n")).unwrap();
        let core = opened(dir);
        assert_eq!(
            rules(&core, dir, true),
            [
                ("home.zip".to_string(), SecretRule::Archive),
                ("notes.zip".to_string(), SecretRule::GithubToken),
                ("tool".to_string(), SecretRule::Executable),
            ]
        );
        // 넣은 새 파일도 같다(인덱스 쪽).
        git(dir, &["add", "home.zip"]);
        assert_eq!(
            rules(&core, dir, false),
            [("home.zip".to_string(), SecretRule::Archive)]
        );
    }

    #[test]
    fn links_are_not_read() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        let outside = tempfile::tempdir().unwrap();
        let secret = outside.path().join("token.txt");
        std::fs::write(&secret, format!("ghp_{}", "a".repeat(40))).unwrap();
        if platform::symlink_file(&secret, &dir.join("링크.txt")).is_err() {
            return;
        }
        let core = opened(dir);
        let view = core
            .queries()
            .secret_check(dir, &SecretCheckRequest { stage_all: true })
            .unwrap();
        assert!(view.findings.is_empty(), "{:?}", view.findings);
    }
}

#[test]
fn app_status_carries_the_git_path_field() {
    let core = Core::with_backend(Arc::new(common::FakeGit::new()) as Arc<dyn GitBackend>).unwrap();
    // 시험용 Core는 git을 찾지 않으므로 비어 있다. 실제 경로는 `init`이 채운다.
    assert_eq!(core.app_status().git_path, "");
}
