//! 덮어쓰며 올리기(`force-push`, D-128)를 로컬 bare 원격과 실제 `Core`로 시험한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, AmendInput, AppSettingsPatch, ErrorCode, ErrorView, ForcePushBlock,
    ForcePushInput, ForcePushPlanView, OpenHow,
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

fn commit(dir: &Path, file: &str, text: &str) -> String {
    std::fs::write(dir.join(file), text).unwrap();
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

/// 원격(main·기능), 내 저장소(기능에 있음, upstream origin/기능), 동료 저장소.
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
    commit(dir, "a.txt", "처음\n");
    git(dir, &["remote", "add", "origin", &url]);
    git(dir, &["push", "-q", "-u", "origin", "main"]);
    git(dir, &["switch", "-q", "-c", "기능"]);
    commit(dir, "f.txt", "기능\n");
    git(dir, &["push", "-q", "-u", "origin", "기능"]);
    let other = tempfile::tempdir().unwrap();
    git(other.path(), &["clone", "-q", &url, "."]);
    configure(other.path(), "동료", "mate@example.com");
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

fn allow(core: &Core, on: bool) {
    core.set_settings(&AppSettingsPatch {
        allow_published_rewrite: Some(on),
        ..Default::default()
    });
}

fn amend(core: &Core, dir: &Path, message: &str) {
    core.execute(
        dir,
        &ActionInput::Amend(AmendInput {
            expect_head: git(dir, &["rev-parse", "HEAD"]),
            message: Some(message.into()),
        }),
        None,
    )
    .unwrap();
}

/// 계획이 보인 값 그대로의 입력.
fn input_from(plan: &ForcePushPlanView) -> ActionInput {
    ActionInput::ForcePush(ForcePushInput {
        remote: plan.remote.clone().unwrap(),
        branch: plan.remote_branch.clone().unwrap(),
        expect_remote: plan.expect_remote.clone().unwrap(),
        expect_head: plan.expect_head.clone().unwrap(),
    })
}

fn force_push(core: &Core, dir: &Path, input: &ActionInput) -> higit_core::Result<ActionResult> {
    let token = core.preview(dir, input).unwrap().token;
    core.execute(dir, input, Some(token))
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn remote_tip(s: &Setup, branch: &str) -> String {
    git(
        s.remote.path(),
        &["rev-parse", &format!("refs/heads/{branch}")],
    )
}

#[test]
fn overwrites_after_rewriting_and_keeps_the_old_tip() {
    let s = setup();
    let dir = s.mine.path();
    let old = remote_tip(&s, "기능");
    let core = core(dir);
    allow(&core, true);
    amend(&core, dir, "고친 이름");
    let plan = core.force_push_plan(dir).unwrap();
    assert_eq!(plan.blocked, None);
    assert_eq!(
        (plan.ahead, plan.dropped, plan.dropped_by_others),
        (1, 1, 0)
    );
    assert!(plan.rewrote_published);
    assert_eq!(plan.sample.len(), 1);
    assert_eq!(plan.sample[0].summary, "f.txt");

    let result = force_push(&core, dir, &input_from(&plan)).unwrap();
    assert_eq!(result, ActionResult::ForcePushed { dropped: 1 });
    assert_eq!(remote_tip(&s, "기능"), git(dir, &["rev-parse", "HEAD"]));
    // 원격의 옛 끝은 백업 ref로 남는다.
    let backups = git(
        dir,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/higit/backup/",
        ],
    );
    assert!(backups.lines().any(|l| l == old), "{backups}");
    // 표식은 끝났고, 경계 줄이 남아 앞선 고치기는 되돌리지 않는다.
    assert!(!dir.join(".git/higit/rewrote.json").exists());
    assert!(core.undo(dir).is_err());
    // 이제 덮어쓸 것이 없다.
    let plan = core.force_push_plan(dir).unwrap();
    assert_eq!(plan.blocked, Some(ForcePushBlock::NothingToOverwrite));
}

#[test]
fn counts_a_teammate_version_that_would_disappear() {
    let s = setup();
    let dir = s.mine.path();
    // 동료가 기능에 버전을 올렸고 나는 받아 왔다.
    git(s.other.path(), &["switch", "-q", "기능"]);
    commit(s.other.path(), "mate.txt", "동료\n");
    git(s.other.path(), &["push", "-q", "origin", "기능"]);
    git(dir, &["fetch", "-q", "origin"]);
    // 내 쪽은 받지 않은 채 고쳤다(갈라짐).
    let core = core(dir);
    allow(&core, true);
    amend(&core, dir, "고친 이름");
    let plan = core.force_push_plan(dir).unwrap();
    assert_eq!(plan.blocked, None);
    assert_eq!((plan.dropped, plan.dropped_by_others), (2, 1));
    assert_eq!(plan.sample[0].author, "동료");
}

#[test]
fn needs_the_setting_a_matching_token_and_no_default_branch() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    allow(&core, true);
    amend(&core, dir, "고친 이름");
    let plan = core.force_push_plan(dir).unwrap();
    let input = input_from(&plan);
    // 토큰 없이, 다른 입력의 토큰으로는 실행하지 않는다.
    assert_eq!(
        code(&core.execute(dir, &input, None).unwrap_err()),
        ErrorCode::PreviewRequired
    );
    let other = ActionInput::ForcePush(ForcePushInput {
        remote: "origin".into(),
        branch: "기능".into(),
        expect_remote: "0".repeat(40),
        expect_head: plan.expect_head.clone().unwrap(),
    });
    let token = core.preview(dir, &other).unwrap().token;
    assert_eq!(
        code(&core.execute(dir, &input, Some(token)).unwrap_err()),
        ErrorCode::PreviewMismatch
    );
    // 설정이 꺼지면 계획도 실행도 막힌다.
    allow(&core, false);
    assert_eq!(
        core.force_push_plan(dir).unwrap().blocked,
        Some(ForcePushBlock::Disabled)
    );
    assert_eq!(
        code(&force_push(&core, dir, &input).unwrap_err()),
        ErrorCode::ForcePushDisabled
    );
    // 서버가 기능을 기본 브랜치라고 하면 막는다(로컬에는 그 기록이 없어도).
    allow(&core, true);
    git(
        s.remote.path(),
        &["symbolic-ref", "HEAD", "refs/heads/기능"],
    );
    assert_eq!(
        code(&force_push(&core, dir, &input).unwrap_err()),
        ErrorCode::ForcePushProtected
    );
    git(
        s.remote.path(),
        &["symbolic-ref", "HEAD", "refs/heads/main"],
    );
    // 원격 끝은 그대로다.
    assert_eq!(remote_tip(&s, "기능"), plan.expect_remote.unwrap());
}

#[test]
fn main_is_never_overwritten() {
    let s = setup();
    let dir = s.mine.path();
    git(dir, &["switch", "-q", "main"]);
    let core = core(dir);
    allow(&core, true);
    let plan = core.force_push_plan(dir).unwrap();
    assert_eq!(plan.blocked, Some(ForcePushBlock::DefaultBranch));
    let input = ActionInput::ForcePush(ForcePushInput {
        remote: "origin".into(),
        branch: "main".into(),
        expect_remote: git(dir, &["rev-parse", "origin/main"]),
        expect_head: git(dir, &["rev-parse", "HEAD"]),
    });
    assert_eq!(
        code(&force_push(&core, dir, &input).unwrap_err()),
        ErrorCode::ForcePushProtected
    );
}

#[test]
fn a_teammate_push_in_between_is_not_overwritten() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    allow(&core, true);
    amend(&core, dir, "고친 이름");
    let plan = core.force_push_plan(dir).unwrap();
    // 확인 창을 보는 사이 동료가 올렸다(내 추적 ref는 그대로).
    git(s.other.path(), &["switch", "-q", "기능"]);
    let theirs = commit(s.other.path(), "mate.txt", "동료\n");
    git(s.other.path(), &["push", "-q", "origin", "기능"]);
    assert_eq!(
        code(&force_push(&core, dir, &input_from(&plan)).unwrap_err()),
        ErrorCode::RemoteMoved
    );
    assert_eq!(remote_tip(&s, "기능"), theirs);
}

#[test]
fn nothing_to_overwrite_and_merging_are_refused() {
    let s = setup();
    let dir = s.mine.path();
    let core = core(dir);
    allow(&core, true);
    // 앞서 있기만 하면 그냥 올리면 된다.
    commit(dir, "b.txt", "더\n");
    let plan = core.force_push_plan(dir).unwrap();
    assert_eq!(plan.blocked, Some(ForcePushBlock::NothingToOverwrite));
    let input = ActionInput::ForcePush(ForcePushInput {
        remote: "origin".into(),
        branch: "기능".into(),
        expect_remote: plan.expect_remote.unwrap(),
        expect_head: plan.expect_head.unwrap(),
    });
    assert_eq!(
        code(&force_push(&core, dir, &input).unwrap_err()),
        ErrorCode::NothingToOverwrite
    );
    // 합치는 중.
    git(dir, &["switch", "-q", "-c", "옆", "HEAD~1"]);
    std::fs::write(dir.join("b.txt"), "옆\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "옆"]);
    git(dir, &["switch", "-q", "기능"]);
    let _ = std::process::Command::new("git")
        .args(["merge", "옆"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert_eq!(
        core.force_push_plan(dir).unwrap().blocked,
        Some(ForcePushBlock::OtherOperation)
    );
}
