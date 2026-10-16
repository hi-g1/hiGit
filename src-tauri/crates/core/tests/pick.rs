//! 되돌리기·이 버전만 복사하기의 시작을 실제 git과 실행기로 시험한다(계획 008 E11, D-101).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ConflictContext, ErrorCode, ErrorView, OidInput, OpenHow, PathsInput,
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
    repo
}

fn commit(dir: &Path, file: &str, text: &str, message: &str) -> String {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

/// 껐다 켜도 같은 서명 키(앱 데이터 폴더를 쓸 수 있는 보통의 경우).
fn core(dir: &Path, keys: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .with_key_dir_for_tests(keys);
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn revert(core: &Core, dir: &Path, oid: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::Revert(OidInput { oid: oid.into() }),
        None,
    )
}

fn pick(core: &Core, dir: &Path, oid: &str) -> higit_core::Result<ActionResult> {
    core.execute(
        dir,
        &ActionInput::CherryPick(OidInput { oid: oid.into() }),
        None,
    )
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn marker(dir: &Path) -> bool {
    dir.join(".git/higit/operation.json").exists()
}

fn read(dir: &Path, file: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(file)).ok()
}

#[test]
fn revert_makes_a_new_version_and_undo_takes_it_back() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    commit(dir, "a.txt", "a\n", "처음");
    let target = commit(dir, "b.txt", "b\n", "넣음");
    let head = commit(dir, "c.txt", "c\n", "또");
    let core = core(dir, keys.path());
    let result = revert(&core, dir, &target).unwrap();
    assert!(
        matches!(result, ActionResult::Reverted { .. }),
        "{result:?}"
    );
    assert_eq!(read(dir, "b.txt"), None);
    assert_eq!(git(dir, &["rev-parse", "HEAD^"]), head);
    assert!(!marker(dir));
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert_eq!(read(dir, "b.txt").as_deref(), Some("b\n"));
    core.redo(dir).unwrap();
    assert_eq!(read(dir, "b.txt"), None);
}

#[test]
fn revert_handles_pushed_root_and_merge_versions() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    let root = commit(dir, "a.txt", "a\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "b.txt", "b\n", "기능");
    git(dir, &["switch", "-q", "main"]);
    commit(dir, "c.txt", "c\n", "내 것");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    let merged = git(dir, &["rev-parse", "HEAD"]);
    // 올린 버전도 되돌리기는 기록을 바꾸지 않으니 된다.
    git(dir, &["update-ref", "refs/remotes/origin/main", &merged]);
    let core = core(dir, keys.path());
    revert(&core, dir, &merged).unwrap();
    assert_eq!(read(dir, "b.txt"), None, "첫 부모 쪽으로 되돌린다");
    assert_eq!(read(dir, "c.txt").as_deref(), Some("c\n"));
    revert(&core, dir, &root).unwrap();
    assert_eq!(read(dir, "a.txt"), None, "첫 버전도 되돌린다");
}

#[test]
fn revert_refuses_bad_states_and_hashes() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    commit(dir, "a.txt", "1\n", "처음");
    let target = commit(dir, "f.txt", "f\n", "넣음");
    git(dir, &["switch", "-q", "-c", "옆", "HEAD~1"]);
    let side = commit(dir, "s.txt", "s\n", "옆");
    git(dir, &["switch", "-q", "main"]);
    let core = core(dir, keys.path());
    // 지금 위치의 조상이 아니거나 없는 해시.
    for bad in [side.as_str(), &"0".repeat(40), "main", &target[..8]] {
        assert_eq!(
            code(&revert(&core, dir, bad).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad}"
        );
    }
    // 넣은 파일이 있으면.
    std::fs::write(dir.join("n.txt"), "n\n").unwrap();
    core.execute(
        dir,
        &ActionInput::Stage(PathsInput {
            paths: vec!["n.txt".into()],
        }),
        None,
    )
    .unwrap();
    assert_eq!(
        code(&revert(&core, dir, &target).unwrap_err()),
        ErrorCode::StagedChanges
    );
    git(dir, &["reset", "-q"]);
    // 고친 파일과 겹치면 아무것도 바꾸지 않는다.
    std::fs::write(dir.join("f.txt"), "내 고침\n").unwrap();
    assert_eq!(
        code(&revert(&core, dir, &target).unwrap_err()),
        ErrorCode::WouldOverwrite
    );
    assert_eq!(read(dir, "f.txt").as_deref(), Some("내 고침\n"));
    assert!(!dir.join(".git/REVERT_HEAD").exists() && !marker(dir));
    std::fs::write(dir.join("f.txt"), "f\n").unwrap();
    // 이미 취소된 변경은 빈 결과.
    commit(dir, "f.txt", "", "손으로 비움");
    std::fs::remove_file(dir.join("f.txt")).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "손으로 지움"]);
    let head = git(dir, &["rev-parse", "HEAD"]);
    assert_eq!(
        code(&revert(&core, dir, &target).unwrap_err()),
        ErrorCode::NoChange
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert!(!dir.join(".git/REVERT_HEAD").exists() && !marker(dir));
}

#[test]
fn revert_conflict_leaves_a_marker_for_the_conflict_screen() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    commit(dir, "f.txt", "1\n", "하나");
    let target = commit(dir, "f.txt", "2\n", "둘");
    commit(dir, "f.txt", "3\n", "셋");
    let core = core(dir, keys.path());
    let result = revert(&core, dir, &target).unwrap();
    let ActionResult::OperationConflicts { context, paths } = result else {
        panic!("{result:?}");
    };
    assert_eq!(
        (context, paths),
        (ConflictContext::Revert, vec!["f.txt".to_string()])
    );
    assert!(marker(dir));
    let state = core.conflict_state(dir).unwrap();
    assert!(state.active && state.context == ConflictContext::Revert);
    // 껐다 켜도(같은 키) 앱의 작업으로 읽는다.
    let again = core_again(dir, keys.path());
    assert!(again.conflict_state(dir).unwrap().active);
}

fn core_again(dir: &Path, keys: &Path) -> Core {
    core(dir, keys)
}

#[test]
fn cherry_pick_copies_and_undo_takes_it_back() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    commit(dir, "a.txt", "a\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    let picked = commit(dir, "b.txt", "b\n", "기능 버전");
    git(dir, &["switch", "-q", "main"]);
    let head = commit(dir, "c.txt", "c\n", "내 것");
    let core = core(dir, keys.path());
    let result = pick(&core, dir, &picked).unwrap();
    let ActionResult::Picked { summary, .. } = result else {
        panic!("{result:?}");
    };
    assert_eq!(summary, "기능 버전");
    assert_ne!(git(dir, &["rev-parse", "HEAD"]), picked);
    assert_eq!(git(dir, &["rev-parse", "HEAD^"]), head);
    assert_eq!(read(dir, "b.txt").as_deref(), Some("b\n"));
    assert!(!marker(dir));
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert_eq!(read(dir, "b.txt"), None);
}

#[test]
fn cherry_pick_refuses_merges_ancestors_unreachable_and_empty() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    let root = commit(dir, "a.txt", "원래\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    let same = commit(dir, "a.txt", "같은 내용\n", "기능 고침");
    commit(dir, "x.txt", "x\n", "기능 더");
    git(dir, &["switch", "-q", "-c", "합친", "main"]);
    commit(dir, "y.txt", "y\n", "합친 쪽");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    let merged = git(dir, &["rev-parse", "HEAD"]);
    git(dir, &["switch", "-q", "-c", "버릴", "main"]);
    let dropped = commit(dir, "z.txt", "z\n", "버릴 것");
    git(dir, &["switch", "-q", "main"]);
    git(dir, &["branch", "-q", "-D", "버릴"]);
    commit(dir, "a.txt", "같은 내용\n", "내 고침");
    let head = git(dir, &["rev-parse", "HEAD"]);
    let core = core(dir, keys.path());
    for bad in [&merged, &root, &dropped] {
        assert_eq!(
            code(&pick(&core, dir, bad).unwrap_err()),
            ErrorCode::InvalidInput,
            "{bad}"
        );
    }
    assert_eq!(
        code(&pick(&core, dir, &same).unwrap_err()),
        ErrorCode::NoChange
    );
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert!(!dir.join(".git/CHERRY_PICK_HEAD").exists() && !marker(dir));
}

#[test]
fn cherry_pick_conflict_and_temporary_key() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    commit(dir, "f.txt", "1\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    let picked = commit(dir, "f.txt", "기능\n", "기능 고침");
    git(dir, &["switch", "-q", "main"]);
    let head = commit(dir, "f.txt", "내\n", "내 고침");
    // 키가 이번 실행만의 것이면 바로 그만두고 시작 전으로 돌린다.
    let temporary = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    temporary.open_project(dir, OpenHow::Picked).unwrap();
    assert!(pick(&temporary, dir, &picked).is_err());
    assert!(!dir.join(".git/CHERRY_PICK_HEAD").exists() && !marker(dir));
    assert_eq!(git(dir, &["rev-parse", "HEAD"]), head);
    assert_eq!(read(dir, "f.txt").as_deref(), Some("내\n"));
    // 보통은 표식을 남기고 충돌 해결 화면으로.
    let core = core(dir, keys.path());
    let result = pick(&core, dir, &picked).unwrap();
    assert!(
        matches!(&result, ActionResult::OperationConflicts { context: ConflictContext::CherryPick, paths } if paths == &["f.txt"]),
        "{result:?}"
    );
    let state = core.conflict_state(dir).unwrap();
    assert!(state.active && state.context == ConflictContext::CherryPick);
    // 멈춘 동안 다른 작업은 거절한다.
    assert_eq!(
        code(&pick(&core, dir, &picked).unwrap_err()),
        ErrorCode::OperationInProgress
    );
}

#[test]
fn identity_is_checked_before_starting() {
    let (repo, keys) = (repo(), tempfile::tempdir().unwrap());
    let dir = repo.path();
    commit(dir, "f.txt", "1\n", "처음");
    let target = commit(dir, "g.txt", "g\n", "둘");
    git(dir, &["config", "--unset", "user.name"]);
    git(dir, &["config", "--unset", "user.email"]);
    git(dir, &["config", "user.useConfigOnly", "true"]);
    let core = core(dir, keys.path());
    // 전역 설정의 이름이 있는 개발 PC에서는 시작한다. 없을 때만 확인한다.
    let has_global = std::process::Command::new("git")
        .args(["var", "GIT_COMMITTER_IDENT"])
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success();
    if !has_global {
        assert_eq!(
            code(&revert(&core, dir, &target).unwrap_err()),
            ErrorCode::IdentityMissing
        );
        assert!(!marker(dir));
    }
}
