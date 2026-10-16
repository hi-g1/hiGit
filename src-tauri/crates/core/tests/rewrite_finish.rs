//! 기록 정리가 겹쳐 멈춘 뒤 마치기·그만두기(계획 008 E15, D-100, D-102). 실제 git으로.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ConflictFinishInput, ErrorCode, ErrorView, FileChoice,
    FileResolution, HunkChoice, OpenHow, RewriteInput, RewriteKindInput, RewriteStepInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
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

fn commit(dir: &Path, file: &str, text: &str, message: &str) -> String {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

/// 처음(f=1, g) → 둘(f=2) → 셋(f=3). 셋을 처음 바로 위로 옮기면 두 번 겹친다. (폴더, 둘, 셋)
fn overlapping() -> (tempfile::TempDir, String, String) {
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
    std::fs::write(dir.join("g.txt"), "g\n").unwrap();
    commit(dir, "f.txt", "1\n", "처음");
    let b = commit(dir, "f.txt", "2\n", "둘");
    let c = commit(dir, "f.txt", "3\n", "셋");
    (repo, b, c)
}

fn core(dir: &Path, keys: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .with_key_dir_for_tests(keys);
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn start(core: &Core, dir: &Path, b: &str, c: &str) {
    let step = |oid: &str| RewriteStepInput {
        oid: oid.into(),
        kind: RewriteKindInput::Pick,
        message: None,
    };
    let r = core
        .execute(
            dir,
            &ActionInput::RebaseI(RewriteInput {
                steps: vec![step(c), step(b)],
            }),
            None,
        )
        .unwrap();
    assert!(
        matches!(r, ActionResult::OperationConflicts { .. }),
        "{r:?}"
    );
}

fn finish(core: &Core, dir: &Path, choice: HunkChoice) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::ConflictFinish(ConflictFinishInput {
            resolutions: vec![FileResolution {
                path: "f.txt".into(),
                choice: FileChoice::Hunks {
                    choices: vec![choice],
                },
            }],
            locale: "ko".into(),
        }),
        None,
    )
}

fn subjects(dir: &Path) -> Vec<String> {
    git(dir, &["log", "--reverse", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

#[test]
fn two_conflicts_in_a_row_across_a_restart_then_undo() {
    let (repo, b, c) = overlapping();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    {
        let core = core(dir, keys.path());
        start(&core, dir, &b, &c);
        // 내 쪽 = 옮기는 버전(셋).
        let file = core.queries().conflict_file(dir, "f.txt").unwrap();
        assert_eq!(
            (
                file.hunks[0].ours.as_slice(),
                file.hunks[0].theirs.as_slice()
            ),
            (&["3".to_string()][..], &["1".to_string()][..])
        );
        let r = finish(&core, dir, HunkChoice::Ours).unwrap();
        assert_eq!(
            r,
            ActionResult::ConflictContinued {
                paths: vec!["f.txt".into()]
            }
        );
    }
    // 껐다 켰다.
    let core = core(dir, keys.path());
    let file = core.queries().conflict_file(dir, "f.txt").unwrap();
    assert_eq!(file.hunks[0].ours, ["2"], "다음 옮기는 버전(둘)");
    assert_eq!(
        finish(&core, dir, HunkChoice::Ours).unwrap(),
        ActionResult::ConflictFinished
    );
    assert_eq!(subjects(dir), ["처음", "셋", "둘"]);
    assert_eq!(std::fs::read_to_string(dir.join("f.txt")).unwrap(), "2\n");
    assert_eq!(git(dir, &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert!(!dir.join(".git/higit/operation.json").exists());
    // Ctrl+Z가 정리 전 기록으로.
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), c);
}

#[test]
fn abort_goes_back_and_backs_up_other_changes() {
    let (repo, b, c) = overlapping();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    start(&core, dir, &b, &c);
    std::fs::write(dir.join("g.txt"), "그사이 고침\n").unwrap();
    assert_eq!(
        core.execute(dir, &ActionInput::ConflictAbort, None)
            .unwrap(),
        ActionResult::ConflictAborted
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), c);
    assert_eq!(git(dir, &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert!(!dir.join(".git/rebase-merge").exists());
    assert!(!dir.join(".git/higit/operation.json").exists());
    let backup = git(
        dir,
        &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
    );
    let backup = backup
        .lines()
        .find(|r| r.contains("conflict-files"))
        .unwrap()
        .to_string();
    assert_eq!(
        git(dir, &["show", &format!("{backup}:g.txt")]),
        "그사이 고침"
    );
}

#[test]
fn planted_commands_or_mismatched_state_are_refused() {
    let (repo, b, c) = overlapping();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    start(&core, dir, &b, &c);
    // 받은 저장소처럼 남은 할 일 목록에 명령을 심었다.
    let todo = dir.join(".git/rebase-merge/git-rebase-todo");
    let before = std::fs::read_to_string(&todo).unwrap();
    std::fs::write(&todo, format!("exec touch pwned\n{before}")).unwrap();
    assert_eq!(
        code(&finish(&core, dir, HunkChoice::Ours).unwrap_err()),
        ErrorCode::OperationInProgress
    );
    assert!(!dir.join("pwned").exists());
    // git을 부르지 않았다: 아직 겹친 그대로다.
    assert!(core.conflict_state(dir).unwrap().files.len() == 1);
    std::fs::write(&todo, before).unwrap();
    // 시작 위치가 표식과 다르면 앱의 작업이 아니다.
    let orig = dir.join(".git/rebase-merge/orig-head");
    std::fs::write(&orig, format!("{b}\n")).unwrap();
    assert_eq!(
        code(&finish(&core, dir, HunkChoice::Ours).unwrap_err()),
        ErrorCode::OperationInProgress
    );
    assert!(core.conflict_state(dir).unwrap().other_operation);
}

#[test]
fn terminal_rebase_without_a_marker_is_refused() {
    let (repo, b, _) = overlapping();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["switch", "-q", "-c", "옆", &b]);
    commit(dir, "f.txt", "옆\n", "옆");
    let _ = std::process::Command::new("git")
        .args(["rebase", "main"])
        .current_dir(dir)
        .output();
    assert!(dir.join(".git/rebase-merge").exists());
    let core = core(dir, keys.path());
    assert_eq!(
        code(&finish(&core, dir, HunkChoice::Ours).unwrap_err()),
        ErrorCode::OperationInProgress
    );
    assert!(dir.join(".git/rebase-merge").exists());
}

#[test]
fn resolving_to_the_rewritten_content_drops_that_version() {
    let (repo, b, c) = overlapping();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    start(&core, dir, &b, &c);
    // 합칠 쪽(정리된 기록 = 처음)을 고르면 셋은 비어서 빠진다. 둘은 처음 위에 그대로 놓인다.
    assert_eq!(
        finish(&core, dir, HunkChoice::Theirs).unwrap(),
        ActionResult::ConflictFinished
    );
    assert_eq!(subjects(dir), ["처음", "둘"]);
    assert_eq!(std::fs::read_to_string(dir.join("f.txt")).unwrap(), "2\n");
}
