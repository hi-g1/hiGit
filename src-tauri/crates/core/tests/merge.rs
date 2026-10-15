//! 합치기 제안·합치기·"main에 합치기"를 실제 git으로 시험한다(D-048, D-069, D-071).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ErrorCode, ErrorView, MergeHintKind, MergeInput, MergeIntoInput,
    OpenHow,
};
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

fn commit_file(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

/// main에 a.txt(다섯 줄), 거기서 갈라진 "기능" 브랜치로 갈아탄 저장소.
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
    commit_file(dir, "a.txt", "1\n2\n3\n4\n5\n");
    git(dir, &["switch", "-q", "-c", "기능"]);
    repo
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn merge(core: &Core, dir: &Path, branch: &str) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::Merge(MergeInput {
            branch: branch.into(),
            locale: "ko".into(),
        }),
        None,
    )
}

fn on_main(dir: &Path, file: &str, text: &str) {
    git(dir, &["switch", "-q", "main"]);
    commit_file(dir, file, text);
    git(dir, &["switch", "-q", "기능"]);
}

fn code(err: higit_core::CoreError) -> ErrorCode {
    ErrorView::from(&err).code
}

#[test]
fn hint_kinds() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    assert_eq!(
        core.queries().merge_hint(dir).unwrap().kind,
        MergeHintKind::None
    );
    commit_file(dir, "b.txt", "내 것\n");
    let hint = core.queries().merge_hint(dir).unwrap();
    assert_eq!(
        (hint.kind, hint.ahead, hint.behind),
        (MergeHintKind::Into, 1, 0)
    );
    assert_eq!(hint.branch.as_deref(), Some("main"));
    on_main(dir, "a.txt", "1\n2\n3\n4\n5\n6\n");
    let hint = core.queries().merge_hint(dir).unwrap();
    assert_eq!(
        (hint.kind, hint.ahead, hint.behind),
        (MergeHintKind::Bring, 1, 1)
    );
    assert!(hint.overlap.is_empty());
    // 같은 줄을 고치면 겹친 파일을 알려 준다.
    commit_file(dir, "a.txt", "1\n2\n3\n4\n5\n육\n");
    let hint = core.queries().merge_hint(dir).unwrap();
    assert_eq!(hint.overlap, ["a.txt"]);
    assert_eq!(hint.overlap_count, 1);
    // 기본 브랜치에서는 제안하지 않는다.
    git(dir, &["switch", "-q", "main"]);
    assert_eq!(
        core.queries().merge_hint(dir).unwrap().kind,
        MergeHintKind::None
    );
}

#[test]
fn bring_merges_and_undo_goes_back_with_edits_kept() {
    let repo = repo();
    let dir = repo.path();
    commit_file(dir, "b.txt", "내 것\n");
    on_main(dir, "c.txt", "main 것\n");
    let core = core(dir);
    let before = git(dir, &["rev-parse", "HEAD"]);
    std::fs::write(dir.join("b.txt"), "내 것 고침\n").unwrap();
    let r = merge(&core, dir, "main").unwrap();
    assert_eq!(
        r,
        ActionResult::Merged {
            branch: "main".into(),
            fast_forward: false
        }
    );
    assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "main 가져와 합침");
    assert!(dir.join("c.txt").exists());
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), before);
    assert!(!dir.join("c.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("b.txt")).unwrap(),
        "내 것 고침\n"
    );
}

#[test]
fn behind_only_fast_forwards() {
    let repo = repo();
    let dir = repo.path();
    on_main(dir, "c.txt", "main 것\n");
    let core = core(dir);
    let r = merge(&core, dir, "main").unwrap();
    assert_eq!(
        r,
        ActionResult::Merged {
            branch: "main".into(),
            fast_forward: true
        }
    );
    assert_eq!(
        git(dir, &["rev-parse", "HEAD"]),
        git(dir, &["rev-parse", "main"])
    );
}

#[test]
fn overlap_starts_the_merge_and_stops_for_the_conflict_screen() {
    let repo = repo();
    let dir = repo.path();
    commit_file(dir, "a.txt", "1\n둘\n3\n4\n5\n");
    on_main(dir, "a.txt", "1\n이\n3\n4\n5\n");
    let core = core(dir);
    let r = merge(&core, dir, "main").unwrap();
    assert_eq!(
        r,
        ActionResult::MergeConflicts {
            paths: vec!["a.txt".into()]
        }
    );
    assert!(dir.join(".git/MERGE_HEAD").exists());
    // 합치는 중에는 또 합치지 않고 되돌리기도 하지 않는다.
    assert_eq!(
        code(merge(&core, dir, "main").unwrap_err()),
        ErrorCode::OperationInProgress
    );
    assert_eq!(
        code(core.undo(dir).unwrap_err()),
        ErrorCode::OperationInProgress
    );
}

#[test]
fn staged_files_block_merging() {
    let repo = repo();
    let dir = repo.path();
    on_main(dir, "c.txt", "main 것\n");
    std::fs::write(dir.join("d.txt"), "넣음\n").unwrap();
    git(dir, &["add", "d.txt"]);
    let core = core(dir);
    assert_eq!(
        code(merge(&core, dir, "main").unwrap_err()),
        ErrorCode::StagedChanges
    );
    assert_eq!(
        code(merge(&core, dir, "-x").unwrap_err()),
        ErrorCode::InvalidInput
    );
    assert_eq!(
        code(merge(&core, dir, "없음").unwrap_err()),
        ErrorCode::InvalidInput
    );
}

#[test]
fn a_tag_with_the_same_name_does_not_win() {
    let repo = repo();
    let dir = repo.path();
    // 처음 버전에 "main"이라는 태그를 단다. 짧은 이름이면 태그가 먼저 풀린다.
    git(dir, &["tag", "main", "HEAD"]);
    on_main(dir, "c.txt", "main 것\n");
    let core = core(dir);
    merge(&core, dir, "main").unwrap();
    assert!(dir.join("c.txt").exists(), "브랜치 main을 합쳐야 한다");
}

#[test]
fn a_hook_refusing_the_merge_leaves_a_finish_only_merge() {
    let repo = repo();
    let dir = repo.path();
    commit_file(dir, "b.txt", "내 것\n");
    on_main(dir, "c.txt", "main 것\n");
    let core = core(dir);
    let hook = dir.join(".git/hooks/pre-merge-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&hook)
        .status();
    let r = merge(&core, dir, "main").unwrap();
    assert_eq!(r, ActionResult::MergeConflicts { paths: vec![] });
    assert!(dir.join(".git/MERGE_HEAD").exists());
}

fn merge_into(core: &Core, dir: &Path) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::MergeInto(MergeIntoInput {
            branch: "main".into(),
        }),
        None,
    )
}

#[test]
fn merge_into_moves_main_forward_and_undo_moves_it_back() {
    let repo = repo();
    let dir = repo.path();
    let old_main = git(dir, &["rev-parse", "main"]);
    commit_file(dir, "b.txt", "내 것\n");
    std::fs::write(dir.join("a.txt"), "고치는 중\n").unwrap();
    let core = core(dir);
    merge_into(&core, dir).unwrap();
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        git(dir, &["rev-parse", "HEAD"])
    );
    assert_eq!(
        git(dir, &["branch", "--show-current"]),
        "기능",
        "갈아타지 않는다"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "고치는 중\n"
    );
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "main"]), old_main);
    core.redo(dir).unwrap();
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        git(dir, &["rev-parse", "HEAD"])
    );
}

#[test]
fn merge_into_refuses_main_checked_out_in_the_main_folder() {
    let repo = repo();
    let dir = repo.path();
    // 주 작업 폴더는 main, 앱은 연결된 worktree(기능 브랜치)를 연다.
    git(dir, &["switch", "-q", "main"]);
    let parent = tempfile::tempdir().unwrap();
    let wt = parent.path().join("작업 트리");
    git(
        dir,
        &["worktree", "add", "-q", &wt.to_string_lossy(), "기능"],
    );
    commit_file(&wt, "b.txt", "내 것\n");
    let old_main = git(dir, &["rev-parse", "main"]);
    let core = core(&wt);
    let err = merge_into(&core, &wt).unwrap_err();
    assert_eq!(code(err), ErrorCode::InvalidInput);
    assert_eq!(git(dir, &["rev-parse", "main"]), old_main);
}

#[test]
fn merge_into_needs_main_not_ahead() {
    let repo = repo();
    let dir = repo.path();
    commit_file(dir, "b.txt", "내 것\n");
    on_main(dir, "c.txt", "main 것\n");
    let core = core(dir);
    assert_eq!(
        code(merge_into(&core, dir).unwrap_err()),
        ErrorCode::NotFastForward
    );
}
