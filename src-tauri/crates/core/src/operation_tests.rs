//! 앱이 시작한 복사·되돌리기·기록 정리가 남긴 충돌의 맥락과 두 쪽(계획 008 E9). 실제 git으로.

use std::path::Path;
use std::sync::Arc;

use git::{GitBackend, Oid, RepoPath, RewriteKind, RewriteStep};

use crate::Core;
use crate::operation::{OpKind, OpMarker};
use crate::view::{ConflictContext, ConflictFileKind, OpenHow, SideState};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn init() -> tempfile::TempDir {
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

fn commit(dir: &Path, file: &str, text: &str, message: &str) -> Oid {
    std::fs::write(dir.join(file), text).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    oid(dir, "HEAD")
}

fn oid(dir: &Path, rev: &str) -> Oid {
    Oid::parse(&git(dir, &["rev-parse", rev])).unwrap()
}

fn backend() -> git::CliBackend {
    git::CliBackend::new("git", Arc::new(|_| {}))
}

fn core(dir: &Path) -> Core {
    let core = Core::with_backend(Arc::new(backend()) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn mark(core: &Core, dir: &Path, op: &OpMarker) {
    let git_dir = backend().git_dir(&RepoPath::new(dir)).unwrap();
    crate::operation::write(&git_dir, core.key_for_tests(), op).unwrap();
}

fn op(kind: OpKind, head_before: &Oid, base: Option<&Oid>, oids: &[&Oid]) -> OpMarker {
    OpMarker {
        kind,
        branch: "main".into(),
        head_before: head_before.to_string(),
        base: base.map(|b| b.to_string()),
        oids: oids.iter().map(|o| o.to_string()).collect(),
        started_at: 1,
    }
}

/// main과 "기능"이 f.txt를 다르게 고친 뒤 main에서 기능의 버전을 복사하다 멈춘다. (폴더, 지금, 복사하는 버전)
fn picking() -> (tempfile::TempDir, Oid, Oid) {
    let repo = init();
    let dir = repo.path();
    commit(dir, "f.txt", "1\n", "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    let picked = commit(dir, "f.txt", "기능\n", "기능 고침");
    git(dir, &["switch", "-q", "main"]);
    let head = commit(dir, "f.txt", "내\n", "내 고침");
    git(dir, &["cherry-pick", picked.as_str()]);
    (repo, head, picked)
}

#[test]
fn cherry_pick_context_and_sides() {
    let (tmp, head, picked) = picking();
    let dir = tmp.path();
    let core = core(dir);
    // 표식이 없으면 다른 도구의 작업이다.
    let state = core.conflict_state(dir).unwrap();
    assert!(!state.active && state.other_operation);
    assert!(core.pending_work(dir).unwrap().other_operation.is_some());

    mark(&core, dir, &op(OpKind::CherryPick, &head, None, &[&picked]));
    let state = core.conflict_state(dir).unwrap();
    assert!(state.active && !state.other_operation);
    assert_eq!(state.context, ConflictContext::CherryPick);
    assert_eq!(state.ours.as_ref().unwrap().label, "main");
    let theirs = state.theirs.as_ref().unwrap();
    assert_eq!(
        (theirs.label.as_str(), theirs.summary.as_str()),
        ("기능", "기능 고침")
    );
    assert_eq!(state.files.len(), 1);
    assert_eq!(core.pending_work(dir).unwrap().other_operation, None);
    let file = core.queries().conflict_file(dir, "f.txt").unwrap();
    assert_eq!(file.kind, ConflictFileKind::Text);
    assert_eq!(file.hunks[0].ours, ["내"]);
    assert_eq!(file.hunks[0].theirs, ["기능"]);
    // 조회는 표식을 지우지 않는다.
    assert!(core.conflict_state(dir).unwrap().active);
}

#[test]
fn markers_that_do_not_match_are_foreign() {
    let (tmp, head, picked) = picking();
    let dir = tmp.path();
    let core = core(dir);
    // 고른 버전이 다르다.
    mark(&core, dir, &op(OpKind::CherryPick, &head, None, &[&head]));
    assert!(core.conflict_state(dir).unwrap().other_operation);
    // 종류가 다르다.
    mark(&core, dir, &op(OpKind::Revert, &head, None, &[&picked]));
    assert!(core.conflict_state(dir).unwrap().other_operation);
    // 다른 키로 서명했다(다른 앱 데이터, 받은 저장소).
    let git_dir = backend().git_dir(&RepoPath::new(dir)).unwrap();
    crate::operation::write(
        &git_dir,
        &crate::undo::JournalKey::ephemeral(),
        &op(OpKind::CherryPick, &head, None, &[&picked]),
    )
    .unwrap();
    assert!(core.conflict_state(dir).unwrap().other_operation);
    assert!(core.pending_work(dir).unwrap().other_operation.is_some());
}

#[test]
fn revert_context_and_sides() {
    let repo = init();
    let dir = repo.path();
    commit(dir, "f.txt", "1\n", "하나");
    let reverted = commit(dir, "f.txt", "2\n", "둘");
    let head = commit(dir, "f.txt", "3\n", "셋");
    git(dir, &["revert", "--no-edit", reverted.as_str()]);
    let core = core(dir);
    mark(&core, dir, &op(OpKind::Revert, &head, None, &[&reverted]));
    let state = core.conflict_state(dir).unwrap();
    assert!(state.active);
    assert_eq!(state.context, ConflictContext::Revert);
    assert_eq!(state.ours.as_ref().unwrap().label, "main");
    let theirs = state.theirs.as_ref().unwrap();
    assert_eq!(theirs.summary, "둘");
    assert_eq!(
        theirs.label, theirs.short_oid,
        "가리키는 브랜치가 없으면 짧은 해시"
    );
    let file = core.queries().conflict_file(dir, "f.txt").unwrap();
    assert_eq!(file.hunks[0].ours, ["3"]);
    assert_eq!(file.hunks[0].theirs, ["1"]);
}

/// 처음(1) 위에 셋(2→3)을 먼저 놓아 겹친 기록 정리. (폴더, 처음, 둘, 셋)
fn rewriting(files: impl Fn(&Path, &str)) -> (tempfile::TempDir, Oid, Oid, Oid) {
    let repo = init();
    let dir = repo.path();
    files(dir, "1");
    let base = commit(dir, "f.txt", "1\n", "처음");
    files(dir, "2");
    let b = commit(dir, "f.txt", "2\n", "둘");
    files(dir, "3");
    let c = commit(dir, "f.txt", "3\n", "셋");
    let steps = [&c, &b]
        .iter()
        .map(|o| RewriteStep {
            oid: (*o).clone(),
            kind: RewriteKind::Pick,
        })
        .collect();
    let result = backend().run(
        &RepoPath::new(dir),
        &git::GitOp::RewriteHistory {
            base: base.clone(),
            steps,
        },
    );
    assert!(result.is_err(), "겹쳐서 멈춰야 한다");
    (repo, base, b, c)
}

#[test]
fn rebase_context_flips_the_sides() {
    let (tmp, base, b, c) = rewriting(|_, _| {});
    let dir = tmp.path();
    let core = core(dir);
    assert!(core.conflict_state(dir).unwrap().other_operation);
    mark(&core, dir, &op(OpKind::Rebase, &c, Some(&base), &[&b, &c]));
    let state = core.conflict_state(dir).unwrap();
    assert!(state.active && !state.other_operation);
    assert_eq!(state.context, ConflictContext::Rebase);
    // 내 쪽 = 옮기는 버전(셋), 합칠 쪽 = 정리된 기록(지금 HEAD = 처음, 이름표는 브랜치).
    assert_eq!(state.ours.as_ref().unwrap().summary, "셋");
    let theirs = state.theirs.as_ref().unwrap();
    assert_eq!(
        (theirs.label.as_str(), theirs.summary.as_str()),
        ("main", "처음")
    );
    let file = core.queries().conflict_file(dir, "f.txt").unwrap();
    assert_eq!(file.hunks[0].ours, ["3"]);
    assert_eq!(file.hunks[0].theirs, ["1"]);
    assert_eq!(core.pending_work(dir).unwrap().other_operation, None);
    // 시작 위치가 다르면 앱의 작업이 아니다.
    mark(&core, dir, &op(OpKind::Rebase, &b, Some(&base), &[&b, &c]));
    assert!(core.conflict_state(dir).unwrap().other_operation);
}

#[test]
fn flipped_whole_file_sides() {
    // 옮기는 버전(셋)이 g.txt를 지우고 바탕(처음)에는 g.txt가 그대로다: 내 쪽이 지운 쪽.
    let (tmp, base, b, c) = rewriting(|dir, n| match n {
        "1" => std::fs::write(dir.join("g.txt"), "g\n").unwrap(),
        "2" => std::fs::write(dir.join("g.txt"), "g2\n").unwrap(),
        _ => std::fs::remove_file(dir.join("g.txt")).unwrap(),
    });
    let dir = tmp.path();
    let core = core(dir);
    mark(&core, dir, &op(OpKind::Rebase, &c, Some(&base), &[&b, &c]));
    let file = core.queries().conflict_file(dir, "g.txt").unwrap();
    assert_eq!(file.kind, ConflictFileKind::Whole);
    assert_eq!(file.ours_side, SideState::Deleted);
    assert_eq!(file.theirs_side, SideState::Text);
    assert_eq!(file.theirs_preview, ["g"]);
}

#[test]
fn flipped_mode_change_sides() {
    // 옮기는 버전만 실행 권한을 켰다: 모드가 달라 파일 단위로 고르고, 내 쪽 미리 보기는 옮기는 버전.
    let (tmp, base, b, c) = rewriting(|dir, n| {
        // 작업 폴더의 권한이 아니라 index의 모드를 쓰게 한다(Windows와 같게).
        git(dir, &["config", "core.fileMode", "false"]);
        std::fs::write(dir.join("m.sh"), format!("m{n}\n")).unwrap();
        if n == "3" {
            git(dir, &["add", "m.sh"]);
            git(dir, &["update-index", "--chmod=+x", "m.sh"]);
        }
    });
    let dir = tmp.path();
    let core = core(dir);
    mark(&core, dir, &op(OpKind::Rebase, &c, Some(&base), &[&b, &c]));
    let file = core.queries().conflict_file(dir, "m.sh").unwrap();
    assert_eq!(file.kind, ConflictFileKind::Whole);
    assert_eq!(file.ours_preview, ["m3"]);
    assert_eq!(file.theirs_preview, ["m1"]);
}

#[test]
fn leftover_marker_is_tidied_by_pending_work_only() {
    let repo = init();
    let dir = repo.path();
    let head = commit(dir, "f.txt", "1\n", "처음");
    let core = core(dir);
    mark(&core, dir, &op(OpKind::CherryPick, &head, None, &[&head]));
    let file = dir.join(".git/higit/operation.json");
    // 조회는 지우지 않는다.
    core.conflict_state(dir).unwrap();
    assert!(file.exists());
    // 멈춘 작업이 없으니 끝나지 않은 작업 조회가 정리한다.
    core.pending_work(dir).unwrap();
    assert!(!file.exists());
}
