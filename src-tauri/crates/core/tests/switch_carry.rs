//! 브랜치를 바꿀 때 고친 파일을 가져가기·두고 가기와 그 되돌리기, 대화상자 조회를 실제 git으로 시험한다(D-195).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, CarryBlock, DirtyChoice, ErrorCode, ErrorView, OpenHow, SwitchInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=테스트", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn commit_file(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

const LINES: &str = "1\n2\n3\n4\n5\n6\n7\n";

/// main에 a.txt(일곱 줄), b.txt가 있고, `other`는 main에서 갈라져 c.txt를 더한 저장소.
fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "core.autocrlf", "false"]);
    commit_file(dir, "a.txt", LINES);
    commit_file(dir, "b.txt", "b\n");
    git(dir, &["switch", "-q", "-c", "other"]);
    commit_file(dir, "c.txt", "c\n");
    git(dir, &["switch", "-q", "main"]);
    repo
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn switch(
    core: &Core,
    dir: &Path,
    branch: &str,
    remote: Option<&str>,
    dirty: Option<DirtyChoice>,
) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::Switch(SwitchInput {
            branch: branch.into(),
            remote: remote.map(str::to_string),
            dirty,
        }),
        None,
    )
}

fn carry(core: &Core, dir: &Path, branch: &str) -> higit_core::Result<ActionResult> {
    switch(core, dir, branch, None, Some(DirtyChoice::Carry))
}

fn branch(dir: &Path) -> String {
    git(dir, &["branch", "--show-current"]).trim().to_string()
}

fn status(dir: &Path) -> String {
    git(dir, &["status", "--porcelain", "--untracked-files=all"])
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
}

fn refs(dir: &Path, prefix: &str) -> String {
    git(dir, &["for-each-ref", "--format=%(refname)", prefix])
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

#[test]
fn carry_brings_edits_and_undo_takes_them_back() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "1 고침\n2\n3\n4\n5\n6\n7\n").unwrap();
    std::fs::write(dir.join("새 파일.txt"), "새\n").unwrap();
    std::fs::remove_file(dir.join("b.txt")).unwrap();
    let stash_before = git(dir, &["stash", "list"]);
    let core = core(dir);

    let r = carry(&core, dir, "other").unwrap();
    assert_eq!(
        r,
        ActionResult::Switched {
            branch: "other".into(),
            parked_on: None,
            unparked: false,
            unpark_blocked: false,
            files: 3,
            carried: true,
        }
    );
    assert_eq!(branch(dir), "other");
    assert_eq!(read(dir, "a.txt"), "1 고침\n2\n3\n4\n5\n6\n7\n");
    assert_eq!(read(dir, "새 파일.txt"), "새\n");
    assert!(!dir.join("b.txt").exists(), "지운 파일도 지운 채로 온다");
    assert!(dir.join("c.txt").exists());
    assert_eq!(refs(dir, "refs/higit/parked/"), "", "두고 가지 않았다");
    assert!(
        refs(dir, "refs/higit/backup/").contains("switch-carry-other"),
        "적용한 stash는 백업으로 남는다"
    );
    assert_eq!(git(dir, &["stash", "list"]), stash_before);

    // 되돌리면 고친 파일을 들고 main으로 돌아온다.
    let edited = status(dir);
    core.undo(dir).unwrap();
    assert_eq!(branch(dir), "main");
    assert_eq!(status(dir), edited);
    assert_eq!(read(dir, "a.txt"), "1 고침\n2\n3\n4\n5\n6\n7\n");
    assert!(!dir.join("c.txt").exists());
    assert_eq!(refs(dir, "refs/higit/parked/"), "");

    core.redo(dir).unwrap();
    assert_eq!(branch(dir), "other");
    assert_eq!(read(dir, "새 파일.txt"), "새\n");
    assert_eq!(refs(dir, "refs/higit/parked/"), "");
}

#[test]
fn park_choice_is_the_old_behavior() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "고침\n").unwrap();
    let core = core(dir);
    let r = switch(&core, dir, "other", None, Some(DirtyChoice::Park)).unwrap();
    assert!(
        matches!(&r, ActionResult::Switched { parked_on: Some(b), files: 1, carried: false, .. } if b == "main"),
        "{r:?}"
    );
    assert_eq!(status(dir), "");
    assert_ne!(refs(dir, "refs/higit/parked/"), "");
    core.undo(dir).unwrap();
    assert_eq!(read(dir, "a.txt"), "고침\n");
}

#[test]
fn without_edits_both_choices_just_switch() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    let r = carry(&core, dir, "other").unwrap();
    assert!(
        matches!(
            r,
            ActionResult::Switched {
                files: 0,
                carried: false,
                parked_on: None,
                ..
            }
        ),
        "{r:?}"
    );
    let r = switch(&core, dir, "main", None, Some(DirtyChoice::Park)).unwrap();
    assert!(
        matches!(r, ActionResult::Switched { files: 0, .. }),
        "{r:?}"
    );
    assert_eq!(refs(dir, "refs/higit/backup/"), "");
}

#[test]
fn overlapping_edits_block_before_switching() {
    let repo = repo();
    let dir = repo.path();
    git(dir, &["switch", "-q", "other"]);
    commit_file(dir, "a.txt", "1 다른 쪽\n2\n3\n4\n5\n6\n7\n");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(dir.join("a.txt"), "1 내 쪽\n2\n3\n4\n5\n6\n7\n").unwrap();
    std::fs::write(dir.join("b.txt"), "b 고침\n").unwrap();
    let before = status(dir);
    let core = core(dir);

    let plan = core.queries().switch_plan(dir, "other", None).unwrap();
    assert_eq!(plan.dirty_files, 2);
    assert!(!plan.can_carry);
    assert_eq!(plan.reason, Some(CarryBlock::Overlap));
    assert_eq!(plan.blocked, vec!["a.txt".to_string()]);
    assert_eq!(plan.blocked_total, 1);

    let err = carry(&core, dir, "other").unwrap_err();
    assert_eq!(code(&err), ErrorCode::CarryBlocked);
    assert_eq!(ErrorView::from(&err).detail.as_deref(), Some("a.txt"));
    assert_eq!(branch(dir), "main", "갈아타지 않았다");
    assert_eq!(status(dir), before);
    assert_eq!(read(dir, "a.txt"), "1 내 쪽\n2\n3\n4\n5\n6\n7\n");
    assert_eq!(
        refs(dir, "refs/higit/"),
        "",
        "백업도 맡긴 것도 만들지 않았다"
    );
    assert_eq!(git(dir, &["stash", "list"]), "");

    // 두고 가기는 그대로 된다.
    switch(&core, dir, "other", None, Some(DirtyChoice::Park)).unwrap();
    assert_eq!(branch(dir), "other");
}

#[test]
fn edits_on_other_lines_of_the_same_file_come_along() {
    let repo = repo();
    let dir = repo.path();
    git(dir, &["switch", "-q", "other"]);
    commit_file(dir, "a.txt", "1 다른 쪽\n2\n3\n4\n5\n6\n7\n");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(dir.join("a.txt"), "1\n2\n3\n4\n5\n6\n7 내 쪽\n").unwrap();
    let core = core(dir);
    assert!(
        core.queries()
            .switch_plan(dir, "other", None)
            .unwrap()
            .can_carry
    );
    carry(&core, dir, "other").unwrap();
    assert_eq!(read(dir, "a.txt"), "1 다른 쪽\n2\n3\n4\n5\n6\n7 내 쪽\n");
    core.undo(dir).unwrap();
    assert_eq!(branch(dir), "main");
    assert_eq!(read(dir, "a.txt"), "1\n2\n3\n4\n5\n6\n7 내 쪽\n");
}

#[test]
fn a_new_file_that_already_exists_there_is_put_back() {
    let repo = repo();
    let dir = repo.path();
    // 내용이 같아 겹침 검사는 깨끗하지만, 꺼낼 때 이미 있어 적용하지 못하는 경우.
    std::fs::write(dir.join("c.txt"), "c\n").unwrap();
    let core = core(dir);
    let err = carry(&core, dir, "other").unwrap_err();
    assert_eq!(code(&err), ErrorCode::CarryBlocked);
    assert_eq!(branch(dir), "main", "갈아타기를 되돌렸다");
    assert_eq!(read(dir, "c.txt"), "c\n");
    assert_eq!(status(dir), "?? c.txt\n");
    assert_eq!(refs(dir, "refs/higit/parked/"), "");
    assert_eq!(git(dir, &["stash", "list"]), "");

    // 내용이 다르면 겹침 검사에서 막힌다.
    std::fs::write(dir.join("c.txt"), "내 c\n").unwrap();
    let plan = core.queries().switch_plan(dir, "other", None).unwrap();
    assert_eq!(plan.reason, Some(CarryBlock::Overlap));
    assert_eq!(plan.blocked, vec!["c.txt".to_string()]);
}

#[test]
fn files_left_on_the_arriving_branch_turn_carry_off() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("a.txt"), "main에서 고침\n").unwrap();
    let core = core(dir);
    switch(&core, dir, "other", None, None).unwrap();
    std::fs::write(dir.join("b.txt"), "other에서 고침\n").unwrap();

    let plan = core.queries().switch_plan(dir, "main", None).unwrap();
    assert_eq!(plan.reason, Some(CarryBlock::ParkedThere));
    assert_eq!(plan.blocked, vec!["a.txt".to_string()]);
    let err = carry(&core, dir, "main").unwrap_err();
    assert_eq!(code(&err), ErrorCode::CarryBlocked);
    assert_eq!(branch(dir), "other");
    assert_eq!(read(dir, "b.txt"), "other에서 고침\n");
}

#[test]
fn carry_to_a_remote_branch_and_undo_removes_the_created_local() {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let repo = repo();
    let dir = repo.path();
    let url = remote.path().to_string_lossy().into_owned();
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "origin", "other:원격기능"]);
    git(dir, &["fetch", "-q", "origin"]);
    std::fs::write(dir.join("b.txt"), "b 고침\n").unwrap();
    let core = core(dir);

    let plan = core
        .queries()
        .switch_plan(dir, "원격기능", Some("origin"))
        .unwrap();
    assert!(plan.can_carry);
    assert_eq!(plan.dirty_files, 1);
    switch(
        &core,
        dir,
        "원격기능",
        Some("origin"),
        Some(DirtyChoice::Carry),
    )
    .unwrap();
    assert_eq!(branch(dir), "원격기능");
    assert_eq!(read(dir, "b.txt"), "b 고침\n");
    assert!(dir.join("c.txt").exists());

    core.undo(dir).unwrap();
    assert_eq!(branch(dir), "main");
    assert_eq!(read(dir, "b.txt"), "b 고침\n");
    assert!(!refs(dir, "refs/heads/").contains("원격기능"));
}

#[test]
fn undo_is_refused_when_the_way_back_now_overlaps() {
    let repo = repo();
    let dir = repo.path();
    std::fs::write(dir.join("b.txt"), "b 고침\n").unwrap();
    let core = core(dir);
    carry(&core, dir, "other").unwrap();
    // 앱 밖에서 main의 b.txt를 다르게 바꾼다.
    let main = git(dir, &["rev-parse", "main"]).trim().to_string();
    let tree = git(dir, &["rev-parse", "main^{tree}"]).trim().to_string();
    let blob = {
        let out = std::process::Command::new("git")
            .args(["hash-object", "-w", "--stdin"])
            .current_dir(dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        out.stdin.as_ref().unwrap().write_all(b"b main\n").unwrap();
        String::from_utf8(out.wait_with_output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_string()
    };
    let listing = git(dir, &["ls-tree", &tree]).replace(
        &git(dir, &["rev-parse", "main:b.txt"]).trim().to_string(),
        &blob,
    );
    let new_tree = {
        let out = std::process::Command::new("git")
            .args(["mktree"])
            .current_dir(dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        out.stdin
            .as_ref()
            .unwrap()
            .write_all(listing.as_bytes())
            .unwrap();
        String::from_utf8(out.wait_with_output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_string()
    };
    let commit = git(dir, &["commit-tree", &new_tree, "-p", &main, "-m", "밖"])
        .trim()
        .to_string();
    git(dir, &["update-ref", "refs/heads/main", &commit]);

    let err = core.undo(dir).unwrap_err();
    assert_eq!(code(&err), ErrorCode::UndoStale);
    assert_eq!(branch(dir), "other");
    assert_eq!(read(dir, "b.txt"), "b 고침\n");
}

#[test]
fn too_many_files_turn_carry_off_without_computing() {
    let repo = repo();
    let dir = repo.path();
    std::fs::create_dir(dir.join("많음")).unwrap();
    for i in 0..501 {
        std::fs::write(dir.join(format!("많음/{i}.txt")), "x").unwrap();
    }
    let core = core(dir);
    let plan = core.queries().switch_plan(dir, "other", None).unwrap();
    assert_eq!(plan.dirty_files, 501);
    assert_eq!(plan.reason, Some(CarryBlock::TooBig));
    assert!(plan.blocked.is_empty());
    assert_eq!(
        code(&carry(&core, dir, "other").unwrap_err()),
        ErrorCode::CarryBlocked
    );
    assert_eq!(branch(dir), "main");
}

#[test]
fn the_plan_changes_nothing() {
    let repo = repo();
    let dir = repo.path();
    git(dir, &["switch", "-q", "other"]);
    commit_file(dir, "a.txt", "1 다른 쪽\n2\n3\n4\n5\n6\n7\n");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(dir.join("a.txt"), "1 내 쪽\n2\n3\n4\n5\n6\n7\n").unwrap();
    git(dir, &["add", "a.txt"]);
    std::fs::write(dir.join("새.txt"), "새\n").unwrap();
    let snapshot = |dir: &Path| {
        (
            git(dir, &["for-each-ref"]),
            git(dir, &["ls-files", "-s"]),
            status(dir),
            read(dir, "a.txt"),
            git(dir, &["stash", "list"]),
        )
    };
    let before = snapshot(dir);
    let core = core(dir);
    let plan = core.queries().switch_plan(dir, "other", None).unwrap();
    assert_eq!(plan.reason, Some(CarryBlock::Overlap));
    assert_eq!(snapshot(dir), before);

    // 지금 브랜치·버전 없는 저장소는 묻지 않는다.
    assert_eq!(
        core.queries()
            .switch_plan(dir, "main", None)
            .unwrap()
            .dirty_files,
        0
    );
    let empty = tempfile::tempdir().unwrap();
    git(empty.path(), &["init", "-q", "-b", "main"]);
    std::fs::write(empty.path().join("x.txt"), "x").unwrap();
    let core2 = self::core(empty.path());
    assert_eq!(
        core2
            .queries()
            .switch_plan(empty.path(), "other", None)
            .unwrap()
            .dirty_files,
        0
    );
}
