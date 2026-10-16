//! 원격 브랜치 지우기·이름 바꾸기(D-130)를 로컬 bare 원격과 실제 `Core`로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, ErrorCode, ErrorView, OpenHow, RemoteBranchBlock, RemoteBranchInput,
    RemoteBranchRenameInput,
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

fn commit(dir: &Path, file: &str) -> String {
    std::fs::write(dir.join(file), file).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
    git(dir, &["rev-parse", "HEAD"])
}

fn configure(dir: &Path, name: &str, email: &str) {
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", name),
        ("user.email", email),
        ("commit.gpgsign", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
}

/// 원격(main·feature/login), 내 저장소(main, feature/login을 받아 둠), 동료 저장소.
struct Setup {
    remote: tempfile::TempDir,
    mine: tempfile::TempDir,
    other: tempfile::TempDir,
}

fn setup() -> Setup {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let url = remote.path().to_string_lossy().into_owned();
    let mine = tempfile::tempdir().unwrap();
    let dir = mine.path();
    git(dir, &["init", "-q", "-b", "main"]);
    configure(dir, "나", "me@example.com");
    commit(dir, "a.txt");
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "-u", "origin", "main"]);
    let other = tempfile::tempdir().unwrap();
    git(other.path(), &["clone", "-q", &url, "."]);
    configure(other.path(), "동료", "mate@example.com");
    git(other.path(), &["switch", "-q", "-c", "feature/login"]);
    commit(other.path(), "login.txt");
    git(other.path(), &["push", "-q", "origin", "feature/login"]);
    git(dir, &["fetch", "-q", "origin"]);
    Setup {
        remote,
        mine,
        other,
    }
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn confirmed(core: &Core, dir: &Path, input: &ActionInput) -> higit_core::Result<ActionResult> {
    let token = core.preview(dir, input).unwrap().token;
    core.execute(dir, input, Some(token))
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn remote_ref(s: &Setup, name: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "-q", "--verify", name])
        .current_dir(s.remote.path())
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
}

fn delete(branch: &str, tip: &str) -> ActionInput {
    ActionInput::RemoteBranchDelete(RemoteBranchInput {
        remote: "origin".into(),
        branch: branch.into(),
        expect_tip: tip.into(),
        confirm_branch: None,
    })
}

fn rename(from: &str, to: &str, tip: &str) -> ActionInput {
    ActionInput::RemoteBranchRename(RemoteBranchRenameInput {
        remote: "origin".into(),
        from: from.into(),
        to: to.into(),
        expect_tip: tip.into(),
        confirm_branch: None,
    })
}

#[test]
fn plan_counts_versions_only_on_that_branch() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    let plan = core
        .queries()
        .remote_branch_plan(dir, "origin", "feature/login")
        .unwrap();
    assert_eq!(plan.blocked, None);
    assert_eq!((plan.only_here, plan.by_others), (1, 1));
    assert!(!plan.is_upstream_of_current);
    assert_eq!(
        plan.tip.as_deref(),
        Some(git(dir, &["rev-parse", "origin/feature/login"]).as_str())
    );
    let main = core
        .queries()
        .remote_branch_plan(dir, "origin", "main")
        .unwrap();
    // 로컬에 원격 HEAD 기록이 없으면 이름만 기본 같은 것으로 보고 이름 입력을 받는다(서버가 실행 때 다시 봄).
    assert_eq!(main.blocked, None);
    assert!(main.needs_name_confirm);
    assert!(main.is_upstream_of_current);
    git(dir, &["remote", "set-head", "origin", "main"]);
    let recorded = core
        .queries()
        .remote_branch_plan(dir, "origin", "main")
        .unwrap();
    assert_eq!(recorded.blocked, Some(RemoteBranchBlock::Protected));
    let none = core
        .queries()
        .remote_branch_plan(dir, "origin", "없음")
        .unwrap();
    assert_eq!(none.blocked, Some(RemoteBranchBlock::NoTrackingRef));
    let other = core.queries().remote_branch_plan(dir, "up", "x").unwrap();
    assert_eq!(other.blocked, Some(RemoteBranchBlock::NoRemote));
}

#[test]
fn delete_needs_a_token_and_the_seen_tip_and_keeps_a_backup() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    let input = delete("feature/login", &tip);
    assert_eq!(
        code(&core.execute(dir, &input, None).unwrap_err()),
        ErrorCode::PreviewRequired
    );
    // 확인 창이 본 값과 다르면(그사이 받아 옴) 지우지 않는다.
    assert_eq!(
        code(&confirmed(&core, dir, &delete("feature/login", &"0".repeat(40))).unwrap_err()),
        ErrorCode::RemoteMoved
    );
    let result = confirmed(&core, dir, &input).unwrap();
    assert_eq!(
        result,
        ActionResult::RemoteBranchDeleted {
            remote: "origin".into(),
            branch: "feature/login".into()
        }
    );
    assert_eq!(remote_ref(&s, "refs/heads/feature/login"), None);
    assert!(git(dir, &["for-each-ref", "refs/remotes/origin/feature/login"]).is_empty());
    let backups = git(
        dir,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/higit/backup/",
        ],
    );
    assert!(backups.lines().any(|l| l == tip), "{backups}");
}

#[test]
fn delete_succeeds_when_already_gone_and_refuses_when_moved_on_the_server() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    // 동료가 먼저 지웠다(내 추적 ref는 그대로).
    git(
        s.other.path(),
        &["push", "-q", "origin", "--delete", "feature/login"],
    );
    confirmed(&core, dir, &delete("feature/login", &tip)).unwrap();
    // 서버에서 움직인 브랜치는 지우지 않는다.
    git(s.other.path(), &["push", "-q", "origin", "feature/login"]);
    commit(s.other.path(), "more.txt");
    git(s.other.path(), &["push", "-q", "origin", "feature/login"]);
    let moved = git(s.other.path(), &["rev-parse", "HEAD"]);
    git(dir, &["fetch", "-q", "origin"]);
    git(
        dir,
        &["update-ref", "refs/remotes/origin/feature/login", &tip],
    );
    assert_eq!(
        code(&confirmed(&core, dir, &delete("feature/login", &tip)).unwrap_err()),
        ErrorCode::RemoteMoved
    );
    assert_eq!(remote_ref(&s, "refs/heads/feature/login"), Some(moved));
}

#[test]
fn default_branch_and_head_are_never_deleted() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    let main = git(dir, &["rev-parse", "origin/main"]);
    assert_eq!(
        code(&confirmed(&core, dir, &delete("main", &main)).unwrap_err()),
        ErrorCode::RemoteBranchProtected
    );
    assert_eq!(
        code(&confirmed(&core, dir, &delete("HEAD", &main)).unwrap_err()),
        ErrorCode::InvalidInput
    );
    // 서버가 기본 브랜치라고 하는 브랜치도 막는다.
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    git(
        s.remote.path(),
        &["symbolic-ref", "HEAD", "refs/heads/feature/login"],
    );
    assert_eq!(
        code(&confirmed(&core, dir, &delete("feature/login", &tip)).unwrap_err()),
        ErrorCode::RemoteBranchProtected
    );
    assert!(remote_ref(&s, "refs/heads/feature/login").is_some());
}

/// 이름만 기본 같은 원격 브랜치(서버 기본은 trunk인데 main)는 이름을 똑같이 입력하면 지운다(D-183).
#[test]
fn name_only_default_is_deleted_with_the_typed_name() {
    let s = setup();
    let dir = s.mine.path();
    git(
        s.other.path(),
        &["push", "-q", "origin", "main:refs/heads/trunk"],
    );
    git(
        s.remote.path(),
        &["symbolic-ref", "HEAD", "refs/heads/trunk"],
    );
    git(dir, &["fetch", "-q", "origin"]);
    let core = core(dir);
    let main = git(dir, &["rev-parse", "origin/main"]);
    let with = |confirm: Option<&str>| {
        ActionInput::RemoteBranchDelete(RemoteBranchInput {
            remote: "origin".into(),
            branch: "main".into(),
            expect_tip: main.clone(),
            confirm_branch: confirm.map(str::to_owned),
        })
    };
    for wrong in [None, Some("main "), Some("MAIN")] {
        assert_eq!(
            code(&confirmed(&core, dir, &with(wrong)).unwrap_err()),
            ErrorCode::NameConfirmMismatch,
            "{wrong:?}"
        );
    }
    assert!(remote_ref(&s, "refs/heads/main").is_some());
    confirmed(&core, dir, &with(Some("main"))).unwrap();
    assert!(remote_ref(&s, "refs/heads/main").is_none());
    // 서버 기본(trunk)은 이름을 입력해도 막는다.
    let trunk = git(dir, &["rev-parse", "origin/trunk"]);
    let input = ActionInput::RemoteBranchDelete(RemoteBranchInput {
        remote: "origin".into(),
        branch: "trunk".into(),
        expect_tip: trunk,
        confirm_branch: Some("trunk".into()),
    });
    assert_eq!(
        code(&confirmed(&core, dir, &input).unwrap_err()),
        ErrorCode::RemoteBranchProtected
    );
}

#[test]
fn rename_moves_the_branch_and_relinks_local_upstreams() {
    let s = setup();
    let dir = s.mine.path();
    git(dir, &["switch", "-q", "--track", "origin/feature/login"]);
    let core = core(dir);
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    let result = confirmed(
        &core,
        dir,
        &rename("feature/login", "feature/sign-in", &tip),
    )
    .unwrap();
    assert_eq!(
        result,
        ActionResult::RemoteBranchRenamed {
            remote: "origin".into(),
            from: "feature/login".into(),
            to: "feature/sign-in".into()
        }
    );
    assert_eq!(remote_ref(&s, "refs/heads/feature/sign-in"), Some(tip));
    assert_eq!(remote_ref(&s, "refs/heads/feature/login"), None);
    assert_eq!(
        git(
            dir,
            &["rev-parse", "--abbrev-ref", "feature/login@{upstream}"]
        ),
        "origin/feature/sign-in"
    );
}

#[test]
fn rename_never_overwrites_an_existing_name() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    // 서버에만 있는 같은 이름(내 추적 ref에는 없음)도 거절하고 덮어쓰지 않는다.
    git(
        s.other.path(),
        &["push", "-q", "origin", "main:refs/heads/taken"],
    );
    let taken = remote_ref(&s, "refs/heads/taken");
    assert_eq!(
        code(&confirmed(&core, dir, &rename("feature/login", "taken", &tip)).unwrap_err()),
        ErrorCode::RemoteNameTaken
    );
    assert_eq!(remote_ref(&s, "refs/heads/taken"), taken);
    assert!(remote_ref(&s, "refs/heads/feature/login").is_some());
}

/// 옛 이름 지우기만 서버 hook이 거절하면 새 이름은 남고 "옛 이름이 남았어요"다. hook 실행 비트가 필요해 Unix만.
#[cfg(unix)]
#[test]
fn a_failed_old_name_delete_is_reported_as_partial() {
    use std::os::unix::fs::PermissionsExt;
    let s = setup();
    let dir = s.mine.path();
    let hook = s.remote.path().join("hooks").join("pre-receive");
    std::fs::write(
        &hook,
        "#!/bin/sh\nwhile read old new ref; do\n  case $new in 0000000000000000000000000000000000000000) exit 1;; esac\ndone\nexit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    let core = core(dir);
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    let err = confirmed(
        &core,
        dir,
        &rename("feature/login", "feature/sign-in", &tip),
    )
    .unwrap_err();
    assert_eq!(code(&err), ErrorCode::RemoteRenamePartial);
    assert!(ErrorView::from(&err).detail.is_some());
    assert_eq!(
        remote_ref(&s, "refs/heads/feature/sign-in"),
        Some(tip.clone())
    );
    assert_eq!(remote_ref(&s, "refs/heads/feature/login"), Some(tip));
}

#[test]
fn remote_branch_work_leaves_no_journal_line() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    // 앞선 버전 저장은 되돌릴 수 있어야 한다.
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    core.execute(
        dir,
        &ActionInput::Commit(higit_core::view::CommitInput {
            message: "b".into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
    let tip = git(dir, &["rev-parse", "origin/feature/login"]);
    confirmed(&core, dir, &delete("feature/login", &tip)).unwrap();
    core.undo(dir).unwrap();
    assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "a.txt");
}
