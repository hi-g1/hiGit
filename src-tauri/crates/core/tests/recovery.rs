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
