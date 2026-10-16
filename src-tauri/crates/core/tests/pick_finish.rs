//! 앱이 시작한 복사·되돌리기가 남긴 충돌을 마치고 그만두기(계획 008 E12, D-101). 실제 git으로.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ConflictFinishInput, ErrorCode, ErrorView, FileChoice,
    FileResolution, HunkChoice, OidInput, OpenHow,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "core.quotepath=off"])
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

fn commit_all(dir: &Path, message: &str) -> String {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

const BASE: &str = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n";

/// 2·5·8·11번째 줄 앞에 `prefix`를 붙인다.
fn spots(prefix: &str) -> String {
    BASE.lines()
        .map(|l| match l {
            "2" | "5" | "8" | "11" => format!("{prefix}{l}\n"),
            _ => format!("{l}\n"),
        })
        .collect()
}

/// main과 "기능"이 a.txt 네 곳, 이진 b.bin, d.txt(기능은 지움, main은 고침)를 다르게 고쳤다.
/// (폴더, 지금, 복사할 버전)
fn diverged() -> (tempfile::TempDir, String, String) {
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
    std::fs::write(dir.join("a.txt"), BASE).unwrap();
    std::fs::write(dir.join("b.bin"), b"\0base").unwrap();
    std::fs::write(dir.join("d.txt"), "d\n").unwrap();
    commit_all(dir, "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    std::fs::write(dir.join("a.txt"), spots("기능")).unwrap();
    std::fs::write(dir.join("b.bin"), b"\0theirs").unwrap();
    std::fs::remove_file(dir.join("d.txt")).unwrap();
    let picked = commit_all(dir, "기능 고침");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(dir.join("a.txt"), spots("내")).unwrap();
    std::fs::write(dir.join("b.bin"), b"\0ours").unwrap();
    std::fs::write(dir.join("d.txt"), "d 내 고침\n").unwrap();
    let head = commit_all(dir, "내 고침");
    (repo, head, picked)
}

fn core(dir: &Path, keys: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .with_key_dir_for_tests(keys);
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn start(core: &Core, dir: &Path, picked: &str) {
    let r = core
        .execute(
            dir,
            &ActionInput::CherryPick(OidInput { oid: picked.into() }),
            None,
        )
        .unwrap();
    assert!(
        matches!(r, ActionResult::OperationConflicts { .. }),
        "{r:?}"
    );
}

fn finish(
    core: &Core,
    dir: &Path,
    resolutions: Vec<FileResolution>,
) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::ConflictFinish(ConflictFinishInput {
            resolutions,
            locale: "ko".into(),
        }),
        None,
    )
}

fn file(path: &str, choice: FileChoice) -> FileResolution {
    FileResolution {
        path: path.into(),
        choice,
    }
}

fn mixed() -> Vec<FileResolution> {
    vec![
        file(
            "a.txt",
            FileChoice::Hunks {
                choices: vec![
                    HunkChoice::Ours,
                    HunkChoice::Theirs,
                    HunkChoice::Both,
                    HunkChoice::Custom {
                        text: "직접11\n".into(),
                    },
                ],
            },
        ),
        file("b.bin", FileChoice::Ours),
        file("d.txt", FileChoice::Theirs),
    ]
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

#[test]
fn finishing_makes_the_copied_version() {
    let (repo, head, picked) = diverged();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    start(&core, dir, &picked);
    let state = core.conflict_state(dir).unwrap();
    assert_eq!(state.files.len(), 3);
    // 다 고르지 않으면 쓰지 않는다.
    let err = finish(&core, dir, vec![file("b.bin", FileChoice::Ours)]).unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);

    assert_eq!(
        finish(&core, dir, mixed()).unwrap(),
        ActionResult::ConflictFinished
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD^"]), head);
    assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "기능 고침");
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "1\n내2\n3\n4\n기능5\n6\n7\n내8\n기능8\n9\n10\n직접11\n12\n13\n14\n"
    );
    assert_eq!(std::fs::read(dir.join("b.bin")).unwrap(), b"\0ours");
    assert!(!dir.join("d.txt").exists());
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    assert!(!dir.join(".git/higit/operation.json").exists());
    assert!(!dir.join(".git/CHERRY_PICK_HEAD").exists());
    // Ctrl+Z가 마친 새 버전을 되돌린다.
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
}

#[test]
fn abort_goes_back_and_keeps_a_backup() {
    let (repo, head, picked) = diverged();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    start(&core, dir, &picked);
    let r = core
        .execute(dir, &ActionInput::ConflictAbort, None)
        .unwrap();
    assert_eq!(r, ActionResult::ConflictAborted);
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        spots("내")
    );
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    assert!(!dir.join(".git/higit/operation.json").exists());
    let backups = git(
        dir,
        &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
    );
    assert!(backups.contains("conflict-files-main"), "{backups}");
}

#[test]
fn continues_after_a_restart_and_refuses_without_a_marker() {
    let (repo, _, picked) = diverged();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    {
        let core = core(dir, keys.path());
        start(&core, dir, &picked);
    }
    // 앱을 껐다 켰다(같은 앱 데이터의 키).
    let again = core(dir, keys.path());
    assert_eq!(
        finish(&again, dir, mixed()).unwrap(),
        ActionResult::ConflictFinished
    );

    // 터미널에서 시작한 복사는 앱이 끝내지 않는다.
    let (repo, _, picked) = diverged();
    let dir = repo.path();
    let _ = std::process::Command::new("git")
        .args(["cherry-pick", &picked])
        .current_dir(dir)
        .output();
    let core = core(dir, keys.path());
    assert_eq!(
        code(&finish(&core, dir, mixed()).unwrap_err()),
        ErrorCode::OperationInProgress
    );
    assert_eq!(
        code(
            &core
                .execute(dir, &ActionInput::ConflictAbort, None)
                .unwrap_err()
        ),
        ErrorCode::OperationInProgress
    );
    assert!(dir.join(".git/CHERRY_PICK_HEAD").exists());
}

#[test]
fn resolving_to_the_same_content_skips_the_copy() {
    let (repo, head, picked) = diverged();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    start(&core, dir, &picked);
    let mine = vec![
        file(
            "a.txt",
            FileChoice::Hunks {
                choices: vec![HunkChoice::Ours; 4],
            },
        ),
        file("b.bin", FileChoice::Ours),
        file("d.txt", FileChoice::Ours),
    ];
    assert_eq!(
        code(&finish(&core, dir, mine).unwrap_err()),
        ErrorCode::NoChange
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert!(!dir.join(".git/CHERRY_PICK_HEAD").exists());
    assert!(!dir.join(".git/higit/operation.json").exists());
    assert!(!core.conflict_state(dir).unwrap().active);
}

#[test]
fn revert_conflict_finishes_too() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
    ] {
        git(dir, &["config", k, v]);
    }
    std::fs::write(dir.join("f.txt"), "1\n").unwrap();
    commit_all(dir, "하나");
    std::fs::write(dir.join("f.txt"), "2\n").unwrap();
    let target = commit_all(dir, "둘");
    std::fs::write(dir.join("f.txt"), "3\n").unwrap();
    let head = commit_all(dir, "셋");
    let keys = tempfile::tempdir().unwrap();
    let core = core(dir, keys.path());
    let r = core
        .execute(dir, &ActionInput::Revert(OidInput { oid: target }), None)
        .unwrap();
    assert!(
        matches!(r, ActionResult::OperationConflicts { .. }),
        "{r:?}"
    );
    let r = finish(
        &core,
        dir,
        vec![file(
            "f.txt",
            FileChoice::Hunks {
                choices: vec![HunkChoice::Theirs],
            },
        )],
    )
    .unwrap();
    assert_eq!(r, ActionResult::ConflictFinished);
    assert_eq!(git(dir, &["rev-parse", "HEAD^"]), head);
    assert_eq!(std::fs::read_to_string(dir.join("f.txt")).unwrap(), "1\n");
    assert!(!dir.join(".git/REVERT_HEAD").exists());
}
