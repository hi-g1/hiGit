//! 백업 정리와 작업 내역을 실제 git으로 시험한다(D-079, D-085).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::OpenHow;
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
    std::fs::write(dir.join("a.txt"), "1\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    repo
}

fn open(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn backups(dir: &Path) -> Vec<String> {
    let out = git(dir, &["for-each-ref", "--format=%(refname)", "refs/higit/"]);
    out.lines().map(str::to_string).collect()
}

#[test]
fn opening_prunes_old_backups_a_few_at_a_time() {
    let repo = repo();
    let dir = repo.path();
    let head = git(dir, &["rev-parse", "HEAD"]);
    // 2000년 백업 25개, 지금 백업 하나, 이름이 다른 것 하나, 맡긴 ref 하나.
    for n in 0..25 {
        git(
            dir,
            &[
                "update-ref",
                &format!("refs/higit/backup/20000101T000000Z-{n}/undo-main"),
                &head,
            ],
        );
    }
    git(
        dir,
        &[
            "update-ref",
            "refs/higit/backup/29990101T000000Z-0/undo-main",
            &head,
        ],
    );
    git(
        dir,
        &["update-ref", "refs/higit/backup/이상한-이름/x", &head],
    );
    git(dir, &["update-ref", "refs/higit/parked/main", &head]);
    open(dir);
    assert_eq!(backups(dir).len(), 28 - 20);
    open(dir);
    let left = backups(dir);
    assert_eq!(left.len(), 3, "{left:?}");
    assert!(left.iter().any(|r| r.contains("29990101")));
    assert!(left.iter().any(|r| r.contains("이상한-이름")));
    assert!(left.contains(&"refs/higit/parked/main".to_string()));
}

mod timeline {
    use super::*;
    use higit_core::view::{ActionInput, CommitInput, PathsInput, RecoverySource, RestoreKind};

    fn save(core: &Core, dir: &Path, file: &str, text: &str) {
        std::fs::write(dir.join(file), text).unwrap();
        core.execute(
            dir,
            &ActionInput::Commit(CommitInput {
                message: format!("{file} 저장"),
                stage_all: true,
            }),
            None,
        )
        .unwrap();
    }

    #[test]
    fn five_sources_in_one_timeline() {
        let repo = repo();
        let dir = repo.path();
        let core = open(dir);
        save(&core, dir, "b.txt", "b\n");
        save(&core, dir, "c.txt", "c\n");
        core.undo(dir).unwrap();
        // 앱 밖에서 저장한 버전(reflog로만 보인다).
        std::fs::write(dir.join("d.txt"), "d\n").unwrap();
        git(dir, &["add", "d.txt"]);
        git(dir, &["commit", "-q", "-m", "터미널 저장"]);
        // 고친 것 버리기(백업은 꺼낼 것), 임시 보관.
        std::fs::write(dir.join("a.txt"), "버릴 것\n").unwrap();
        let input = ActionInput::Discard(PathsInput {
            paths: vec!["a.txt".into()],
        });
        let preview = core.preview(dir, &input).unwrap();
        core.execute(dir, &input, Some(preview.token)).unwrap();
        std::fs::write(dir.join("a.txt"), "보관할 것\n").unwrap();
        core.execute(dir, &ActionInput::Stash, None).unwrap();

        let items = core.recovery(dir).unwrap().items;
        let has = |source: RecoverySource| items.iter().any(|i| i.source == source);
        assert!(has(RecoverySource::Journal) && has(RecoverySource::Reflog));
        assert!(has(RecoverySource::Backup) && has(RecoverySource::Stash));
        // 최신 것부터.
        assert!(items.windows(2).all(|w| w[0].at >= w[1].at));
        // 되돌린 버전 저장은 표시된다.
        let undone: Vec<_> = items.iter().filter(|i| i.undone).collect();
        assert_eq!(undone.len(), 1);
        assert_eq!(undone[0].action.as_deref(), Some("commit"));
        // 앱 밖 저장은 reflog 줄로, 앱 저장과 같은 순간의 reflog 줄은 앱 줄에 합쳐진다.
        assert!(
            items
                .iter()
                .any(|i| i.source == RecoverySource::Reflog && i.summary == "터미널 저장")
        );
        assert!(
            !items
                .iter()
                .any(|i| i.source == RecoverySource::Reflog && i.summary == "b.txt 저장")
        );
        // 백업 종류는 모양으로: 버리기 백업은 꺼낼 것, 되돌리기 백업은 버전.
        let backup = |label: &str| {
            items
                .iter()
                .find(|i| i.source == RecoverySource::Backup && i.label.starts_with(label))
                .unwrap()
                .restore
        };
        assert_eq!(backup("discard-"), RestoreKind::Changes);
        assert_eq!(backup("undo-"), RestoreKind::Version);
        let stash = items
            .iter()
            .find(|i| i.source == RecoverySource::Stash)
            .unwrap();
        assert_eq!(stash.restore, RestoreKind::Changes);
        assert_eq!(stash.label, "higit stash main");
    }

    #[test]
    fn lines_from_another_key_show_without_restore() {
        let repo = repo();
        let dir = repo.path();
        {
            let core = open(dir);
            save(&core, dir, "b.txt", "b\n");
        }
        // 다른 Core(다른 키)로 연다.
        let core = open(dir);
        let item = core
            .recovery(dir)
            .unwrap()
            .items
            .into_iter()
            .find(|i| i.source == RecoverySource::Journal)
            .unwrap();
        assert_eq!(item.restore, RestoreKind::None);
    }
}

mod restore {
    use super::*;
    use higit_core::view::{
        ActionInput, ActionResult, CommitInput, ErrorCode, ErrorView, PathsInput, RestoreInput,
        TakeOutInput, TakeOutSource,
    };

    fn save(core: &Core, dir: &Path, file: &str, text: &str) {
        std::fs::write(dir.join(file), text).unwrap();
        core.execute(
            dir,
            &ActionInput::Commit(CommitInput {
                message: format!("{file} 저장"),
                stage_all: true,
            }),
            None,
        )
        .unwrap();
    }

    fn restore(core: &Core, dir: &Path, oid: &str) -> Result<ActionResult, higit_core::CoreError> {
        core.execute(
            dir,
            &ActionInput::Restore(RestoreInput { oid: oid.into() }),
            None,
        )
    }

    fn take_out(
        core: &Core,
        dir: &Path,
        refname: &str,
    ) -> Result<ActionResult, higit_core::CoreError> {
        core.execute(
            dir,
            &ActionInput::TakeOut(TakeOutInput {
                source: TakeOutSource::Ref {
                    refname: refname.into(),
                },
            }),
            None,
        )
    }

    fn code(e: higit_core::CoreError) -> ErrorCode {
        ErrorView::from(&e).code
    }

    #[test]
    fn going_back_keeps_edits_and_the_current_version() {
        let repo = repo();
        let dir = repo.path();
        let core = open(dir);
        let first = git(dir, &["rev-parse", "HEAD"]);
        save(&core, dir, "b.txt", "b\n");
        save(&core, dir, "c.txt", "c\n");
        let latest = git(dir, &["rev-parse", "HEAD"]);
        std::fs::write(dir.join("a.txt"), "고치는 중\n").unwrap();
        assert_eq!(restore(&core, dir, &first).unwrap(), ActionResult::Restored);
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), first);
        assert!(!dir.join("b.txt").exists());
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "고치는 중\n"
        );
        // 지금 버전은 백업 ref로 남아 작업 내역에서 다시 되살릴 수 있다.
        let backups = git(
            dir,
            &[
                "for-each-ref",
                "--format=%(refname) %(objectname)",
                "refs/higit/backup/",
            ],
        );
        assert!(
            backups.contains(&format!("restore-main {latest}")),
            "{backups}"
        );
        // Ctrl+Z로 원래 버전으로.
        core.undo(dir).unwrap();
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), latest);
    }

    #[test]
    fn only_listed_versions_are_accepted() {
        let repo = repo();
        let dir = repo.path();
        let core = open(dir);
        assert_eq!(
            code(restore(&core, dir, &"0".repeat(40)).unwrap_err()),
            ErrorCode::InvalidInput
        );
        assert_eq!(
            code(restore(&core, dir, "-x").unwrap_err()),
            ErrorCode::InvalidInput
        );
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(dir, &["switch", "-q", "--detach"]);
        assert_eq!(
            code(restore(&core, dir, &head).unwrap_err()),
            ErrorCode::InvalidInput
        );
    }

    #[test]
    fn discarded_changes_and_orphan_parked_files_come_back() {
        let repo = repo();
        let dir = repo.path();
        let core = open(dir);
        std::fs::write(dir.join("a.txt"), "버린 것\n").unwrap();
        let input = ActionInput::Discard(PathsInput {
            paths: vec!["a.txt".into()],
        });
        let preview = core.preview(dir, &input).unwrap();
        core.execute(dir, &input, Some(preview.token)).unwrap();
        let backup = git(
            dir,
            &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
        );
        assert_eq!(
            take_out(&core, dir, &backup).unwrap(),
            ActionResult::TookOut
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "버린 것\n"
        );
        // 버전 모양 백업은 꺼내지 않는다.
        git(dir, &["checkout", "-q", "--", "a.txt"]);
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &[
                "update-ref",
                "refs/higit/backup/20990101T000000Z-0/undo-main",
                &head,
            ],
        );
        assert_eq!(
            code(
                take_out(&core, dir, "refs/higit/backup/20990101T000000Z-0/undo-main").unwrap_err()
            ),
            ErrorCode::InvalidInput
        );
        assert_eq!(
            code(take_out(&core, dir, "refs/heads/main").unwrap_err()),
            ErrorCode::InvalidInput
        );
        // 지운 브랜치에 맡겨 둔 파일.
        std::fs::write(dir.join("a.txt"), "맡긴 것\n").unwrap();
        git(
            dir,
            &["stash", "push", "-q", "-m", "higit parked 지운-브랜치"],
        );
        let stash = git(dir, &["rev-parse", "refs/stash"]);
        git(dir, &["stash", "drop", "-q"]);
        let parked = "refs/higit/parked/지운-브랜치";
        git(dir, &["update-ref", parked, &stash]);
        take_out(&core, dir, parked).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "맡긴 것\n"
        );
        assert_eq!(
            git(
                dir,
                &["for-each-ref", "--format=%(refname)", "refs/higit/parked/"]
            ),
            ""
        );
    }
}

#[test]
fn many_backups_cost_at_most_one_log_per_shown_item() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let repo = repo();
    let dir = repo.path();
    let head = git(dir, &["rev-parse", "HEAD"]);
    let script: String = (0..500)
        .map(|i| format!("create refs/higit/backup/x-{i}/t {head}\n"))
        .collect();
    let mut child = std::process::Command::new("git")
        .args(["update-ref", "--stdin"])
        .current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    let logs = Arc::new(AtomicUsize::new(0));
    let counter = logs.clone();
    let backend = git::CliBackend::new(
        "git",
        Arc::new(move |cmd: &mut std::process::Command| {
            if cmd.get_args().any(|a| a == "log") {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }),
    );
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    logs.store(0, Ordering::SeqCst);
    let items = core.recovery(dir).unwrap().items;
    assert_eq!(items.len(), 200);
    let used = logs.load(Ordering::SeqCst);
    assert!(used <= 205, "git log {used}번");
}

#[test]
fn take_out_refuses_paths_into_the_git_folder() {
    use higit_core::view::{ActionInput, ErrorCode, ErrorView, TakeOutInput, TakeOutSource};
    let repo = repo();
    let dir = repo.path();
    // 받은 저장소에 `.GIT/config`를 담은 맡긴 ref가 있다.
    let blob = {
        let mut child = std::process::Command::new("git")
            .args(["hash-object", "-w", "--stdin"])
            .current_dir(dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child.stdin.take().unwrap().write_all(b"[core]\n").unwrap();
        String::from_utf8(child.wait_with_output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_string()
    };
    let mktree = |input: String| {
        let mut child = std::process::Command::new("git")
            .args(["mktree"])
            .current_dir(dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        String::from_utf8(child.wait_with_output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_string()
    };
    let sub = mktree(format!("100644 blob {blob}\tconfig\n"));
    let a = git(dir, &["rev-parse", "HEAD:a.txt"]);
    let tree = mktree(format!("100644 blob {a}\ta.txt\n040000 tree {sub}\t.GIT\n"));
    let evil = git(
        dir,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@e",
            "commit-tree",
            &tree,
            "-p",
            "HEAD",
            "-m",
            "WIP on main",
        ],
    );
    git(dir, &["update-ref", "refs/higit/parked/main", &evil]);
    let core = open(dir);
    let err = core
        .execute(
            dir,
            &ActionInput::TakeOut(TakeOutInput {
                source: TakeOutSource::Ref {
                    refname: "refs/higit/parked/main".into(),
                },
            }),
            None,
        )
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
    // Windows는 `.GIT`과 `.git`이 같은 폴더다: 설정 파일이 덮어써지지 않았는지 본다.
    let config = std::fs::read_to_string(dir.join(".git/config")).unwrap();
    assert!(config.contains("repositoryformatversion"), "{config}");
    assert_eq!(git(dir, &["rev-parse", "refs/higit/parked/main"]), evil);
    let err = core
        .execute(
            dir,
            &ActionInput::TakeOut(TakeOutInput {
                source: TakeOutSource::Parked {
                    branch: "main".into(),
                },
            }),
            None,
        )
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
}
