//! 기록 정리(`rebase-i`)를 실제 git과 실행기로 시험한다(계획 008 E14b, D-100).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ConflictContext, ErrorCode, ErrorView, OpenHow, RewriteInput,
    RewriteKindInput, RewriteStepInput,
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

/// 처음 + 각자 다른 파일을 더하는 둘·셋·넷·다섯. (폴더, [처음, 둘, 셋, 넷, 다섯])
fn five() -> (tempfile::TempDir, Vec<String>) {
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
    let oids = ["처음", "둘", "셋", "넷", "다섯"]
        .iter()
        .enumerate()
        .map(|(i, m)| commit(dir, &format!("{i}.txt"), m, m))
        .collect();
    (repo, oids)
}

fn core(dir: &Path, keys: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .with_key_dir_for_tests(keys);
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn step(oid: &str, kind: RewriteKindInput, message: Option<&str>) -> RewriteStepInput {
    RewriteStepInput {
        oid: oid.into(),
        kind,
        message: message.map(str::to_string),
    }
}

fn rewrite(
    core: &Core,
    dir: &Path,
    steps: Vec<RewriteStepInput>,
) -> higit_core::Result<ActionResult> {
    core.execute(dir, &ActionInput::RebaseI(RewriteInput { steps }), None)
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

use RewriteKindInput::{Drop, Pick, Squash};

#[test]
fn reorder_join_rename_and_drop_together() {
    let (repo, o) = five();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let old_head = git(dir, &["rev-parse", "HEAD"]);
    let author_time = git(dir, &["log", "-1", "--format=%at", &o[2]]);
    let core = core(dir, keys.path());
    // 둘·셋·넷을 고른다: 넷을 맨 아래로, 셋을 둘에 묶고 이름을 고치고(묶음 맨 아래 = 둘), 다섯은
    // 구간 위라 그대로 이어진다.
    let r = rewrite(
        &core,
        dir,
        vec![
            step(&o[3], Pick, None),
            step(&o[1], Pick, Some("둘과 셋\n\n본문")),
            step(&o[2], Squash, None),
        ],
    )
    .unwrap();
    assert_eq!(
        r,
        ActionResult::Rewritten {
            kept: 2,
            dropped: 0,
            joined: 1
        }
    );
    assert_eq!(subjects(dir), ["처음", "넷", "둘과 셋", "다섯"]);
    assert_eq!(
        git(dir, &["log", "-1", "--format=%B", "HEAD~1"]),
        "둘과 셋\n\n본문"
    );
    let files = git(dir, &["show", "--name-only", "--format=", "HEAD~1"]);
    assert!(
        files.contains("1.txt") && files.contains("2.txt"),
        "{files}"
    );
    // 작성 시각은 그대로(묶음은 맨 아래 버전의 것).
    assert_eq!(
        git(dir, &["log", "-1", "--format=%at", "HEAD~1"]),
        git(dir, &["log", "-1", "--format=%at", &o[1]])
    );
    let _ = author_time;
    assert_eq!(git(dir, &["status", "--porcelain"]), "");
    assert!(!dir.join(".git/higit/operation.json").exists());
    // 옛 끝은 백업 ref에 있다.
    let backups = git(
        dir,
        &[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/higit/backup/",
        ],
    );
    assert!(
        backups.contains(&old_head) && backups.contains("rebase-i"),
        "{backups}"
    );
    // Ctrl+Z가 원래 기록(같은 해시)으로.
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), old_head);
    core.redo(dir).unwrap();
    assert_eq!(subjects(dir), ["처음", "넷", "둘과 셋", "다섯"]);

    // 빼기.
    let now: Vec<String> = git(dir, &["log", "--reverse", "--format=%H"])
        .lines()
        .map(str::to_string)
        .collect();
    let r = rewrite(
        &core,
        dir,
        vec![step(&now[1], Drop, None), step(&now[2], Pick, None)],
    )
    .unwrap();
    assert_eq!(
        r,
        ActionResult::Rewritten {
            kept: 1,
            dropped: 1,
            joined: 0
        }
    );
    assert_eq!(subjects(dir), ["처음", "둘과 셋", "다섯"]);
    assert!(!dir.join("3.txt").exists());
}

#[test]
fn refuses_bad_input_and_unchanged_plans() {
    let (repo, o) = five();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let core = core(dir, keys.path());
    let head = git(dir, &["rev-parse", "HEAD"]);
    let invalid = |steps| code(&rewrite(&core, dir, steps).unwrap_err());
    // 바뀐 것 없음(처음 메시지 그대로 보내도).
    assert_eq!(
        invalid(vec![step(&o[2], Pick, Some("셋")), step(&o[3], Pick, None)]),
        ErrorCode::NoChange
    );
    // 첫 단계가 묶기, 빼기 뒤 묶기, 묶기에 메시지, 같은 버전 두 번, 구간과 다른 집합, 첫 버전.
    for steps in [
        vec![step(&o[2], Squash, None), step(&o[3], Pick, None)],
        vec![step(&o[2], Drop, None), step(&o[3], Squash, None)],
        vec![step(&o[2], Pick, None), step(&o[3], Squash, Some("x"))],
        vec![step(&o[2], Pick, None), step(&o[2], Pick, None)],
        vec![step(&o[2], Pick, None), step(&o[4], Pick, None)],
        vec![step(&o[1], Pick, Some("x")), step(&o[0], Pick, None)],
        vec![step(&o[3], Pick, Some(" "))],
        vec![step(&o[3], Pick, Some(&"가".repeat(6000)))],
    ] {
        assert_eq!(invalid(steps), ErrorCode::InvalidInput);
    }
    // 올린 버전, 고친 파일.
    git(dir, &["update-ref", "refs/remotes/origin/main", &o[3]]);
    assert_eq!(
        invalid(vec![step(&o[3], Pick, Some("고침"))]),
        ErrorCode::PublishedHistory
    );
    git(dir, &["update-ref", "-d", "refs/remotes/origin/main"]);
    std::fs::write(dir.join("1.txt"), "고침").unwrap();
    assert_eq!(
        invalid(vec![step(&o[3], Pick, Some("고침"))]),
        ErrorCode::HasChanges
    );
    git(dir, &["checkout", "--", "1.txt"]);
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert!(!dir.join(".git/higit/operation.json").exists());
}

#[test]
fn overlapping_changes_stop_for_the_conflict_screen() {
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
    commit(dir, "f.txt", "1\n", "처음");
    let b = commit(dir, "f.txt", "2\n", "둘");
    let c = commit(dir, "f.txt", "3\n", "셋");
    let keys = tempfile::tempdir().unwrap();
    let core = core(dir, keys.path());
    let r = rewrite(&core, dir, vec![step(&c, Pick, None), step(&b, Pick, None)]).unwrap();
    assert_eq!(
        r,
        ActionResult::OperationConflicts {
            context: ConflictContext::Rebase,
            paths: vec!["f.txt".into()]
        }
    );
    assert!(dir.join(".git/higit/operation.json").exists());
    let state = core.conflict_state(dir).unwrap();
    assert!(state.active && state.context == ConflictContext::Rebase);
    assert_eq!(
        code(&rewrite(&core, dir, vec![step(&c, Pick, None)]).unwrap_err()),
        ErrorCode::OperationInProgress
    );

    // 서명 키가 이번 실행만의 것이면 바로 그만둔다.
    let other = tempfile::tempdir().unwrap();
    let o2 = other.path();
    git(o2, &["init", "-q", "-b", "main"]);
    for (k, v) in [("user.name", "테스트"), ("user.email", "t@example.com")] {
        git(o2, &["config", k, v]);
    }
    commit(o2, "f.txt", "1\n", "처음");
    let b = commit(o2, "f.txt", "2\n", "둘");
    let c = commit(o2, "f.txt", "3\n", "셋");
    let temporary = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    temporary.open_project(o2, OpenHow::Picked).unwrap();
    assert!(
        rewrite(
            &temporary,
            o2,
            vec![step(&c, Pick, None), step(&b, Pick, None)]
        )
        .is_err()
    );
    assert_eq!(git(o2, &["rev-parse", "HEAD"]), c);
    assert!(!o2.join(".git/rebase-merge").exists());
    assert!(!o2.join(".git/higit/operation.json").exists());
}

#[test]
fn hook_refusing_the_new_message_aborts() {
    let (repo, o) = five();
    let keys = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let head = git(dir, &["rev-parse", "HEAD"]);
    let hook = dir.join(".git/hooks/commit-msg");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&hook)
        .status();
    let core = core(dir, keys.path());
    assert!(rewrite(&core, dir, vec![step(&o[3], Pick, Some("고친 넷"))]).is_err());
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert!(!dir.join(".git/rebase-merge").exists());
    assert!(!dir.join(".git/higit/operation.json").exists());
    assert_eq!(git(dir, &["symbolic-ref", "HEAD"]), "refs/heads/main");
}

#[test]
fn setting_allows_published_versions_but_not_changed_files() {
    let (repo, o) = five();
    let dir = repo.path();
    let keys = tempfile::tempdir().unwrap();
    git(dir, &["switch", "-q", "-c", "기능"]);
    git(
        dir,
        &["remote", "add", "origin", "https://example.com/r.git"],
    );
    git(dir, &["update-ref", "refs/remotes/origin/기능", &o[4]]);
    git(dir, &["branch", "--set-upstream-to=origin/기능"]);
    let core = core(dir, keys.path());
    let steps = || {
        vec![
            step(&o[3], Pick, None),
            step(&o[4], Pick, Some("다섯 고침")),
        ]
    };
    assert_eq!(
        code(&rewrite(&core, dir, steps()).unwrap_err()),
        ErrorCode::PublishedHistory
    );
    core.set_settings(&higit_core::view::AppSettingsPatch {
        allow_published_rewrite: Some(true),
        ..Default::default()
    });
    // 켜도 고친 파일은 막는다.
    std::fs::write(dir.join("1.txt"), "고침").unwrap();
    assert_eq!(
        code(&rewrite(&core, dir, steps()).unwrap_err()),
        ErrorCode::HasChanges
    );
    git(dir, &["checkout", "--", "1.txt"]);
    rewrite(&core, dir, steps()).unwrap();
    assert_eq!(subjects(dir).last().map(String::as_str), Some("다섯 고침"));
    // 카드 지도 조회도 같은 규칙.
    let plan = core
        .rewrite_plan(dir, vec![o[1].clone(), o[2].clone()])
        .unwrap();
    assert_eq!(plan.blocked, None);
}
