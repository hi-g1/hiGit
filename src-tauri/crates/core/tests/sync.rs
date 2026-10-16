//! 원격 확인·가져오기·올리기·동기화를 로컬 bare 원격으로 시험한다(D-045, D-048).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionResult, AmendInput, AppSettingsPatch, BranchNameInput, CommitInput,
    ErrorCode, ErrorView, FetchInput, OidInput, OpenHow, SyncInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=민지", "-c", "user.email=m@example.com"])
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

struct Setup {
    _remote: tempfile::TempDir,
    url: String,
    mine: tempfile::TempDir,
    other: tempfile::TempDir,
}

fn setup() -> Setup {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let url = remote.path().to_string_lossy().into_owned();
    let mine = tempfile::tempdir().unwrap();
    git(mine.path(), &["init", "-q", "-b", "main"]);
    git(mine.path(), &["config", "core.autocrlf", "false"]);
    git(mine.path(), &["config", "user.name", "나"]);
    git(mine.path(), &["config", "user.email", "me@example.com"]);
    git(mine.path(), &["config", "commit.gpgsign", "false"]);
    commit_file(mine.path(), "a.txt", "줄 1\n줄 2\n줄 3\n줄 4\n줄 5\n");
    git(mine.path(), &["remote", "add", "origin", &url]);
    git(mine.path(), &["push", "-q", "-u", "origin", "main"]);
    let other = tempfile::tempdir().unwrap();
    git(other.path(), &["clone", "-q", &url, "."]);
    git(other.path(), &["config", "core.autocrlf", "false"]);
    Setup {
        _remote: remote,
        url,
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

fn sync(core: &Core, dir: &Path) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::Sync(SyncInput {
            locale: "ko".into(),
            merge_anyway: false,
        }),
        None,
    )
}

fn others_push(s: &Setup, file: &str, text: &str) {
    commit_file(s.other.path(), file, text);
    git(s.other.path(), &["push", "-q", "origin", "main"]);
}

#[test]
fn receive_only_fast_forwards() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    let core = core(s.mine.path());
    let r = sync(&core, s.mine.path()).unwrap();
    assert_eq!(
        r,
        ActionResult::Synced {
            received: 1,
            sent: 0,
            merged: false,
            overlap: vec![],
            merge_started: false
        }
    );
    assert!(s.mine.path().join("b.txt").exists());
}

#[test]
fn send_only_pushes() {
    let s = setup();
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert!(
        matches!(
            r,
            ActionResult::Synced {
                sent: 1,
                received: 0,
                ..
            }
        ),
        "{r:?}"
    );
    git(s.other.path(), &["pull", "-q", "--ff-only"]);
    assert!(s.other.path().join("c.txt").exists());
}

#[test]
fn both_sides_different_files_merge_and_push() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert!(
        matches!(
            r,
            ActionResult::Synced {
                received: 1,
                merged: true,
                sent: 2,
                ..
            }
        ),
        "{r:?}"
    );
    let subject = git(s.mine.path(), &["log", "-1", "--format=%s"]);
    assert_eq!(subject.trim(), "민지의 버전 1개 가져와 합침");
    // 원격에도 올라갔다.
    git(s.other.path(), &["pull", "-q", "--ff-only"]);
    assert!(s.other.path().join("c.txt").exists());
}

#[test]
fn overlapping_change_starts_the_merge_for_the_conflict_screen() {
    let s = setup();
    others_push(&s, "a.txt", "줄 1\n남이 고침\n줄 3\n줄 4\n줄 5\n");
    commit_file(
        s.mine.path(),
        "a.txt",
        "줄 1\n내가 고침\n줄 3\n줄 4\n줄 5\n",
    );
    let before = git(s.mine.path(), &["rev-parse", "HEAD"]);
    let core = core(s.mine.path());
    let r = sync(&core, s.mine.path()).unwrap();
    assert_eq!(
        r,
        ActionResult::Synced {
            received: 0,
            sent: 0,
            merged: false,
            overlap: vec!["a.txt".into()],
            merge_started: true,
        }
    );
    // 합치는 중으로 멈추고 올리지 않는다(D-069).
    assert_eq!(git(s.mine.path(), &["rev-parse", "HEAD"]), before);
    assert!(s.mine.path().join(".git/MERGE_HEAD").exists());
    // 합치는 중에는 다시 동기화하지 않는다.
    let err = sync(&core, s.mine.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::OperationInProgress);
}

#[test]
fn staged_files_block_a_merge_during_sync() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    std::fs::write(s.mine.path().join("d.txt"), "넣음\n").unwrap();
    git(s.mine.path(), &["add", "d.txt"]);
    let err = sync(&core(s.mine.path()), s.mine.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::StagedChanges);
}

#[test]
fn push_names_the_branch_and_ignores_repo_push_settings() {
    let s = setup();
    // 남이 올린 다른 브랜치. 내 쪽에는 같은 이름의 오래된 브랜치가 있다.
    git(s.mine.path(), &["branch", "old"]);
    git(s.other.path(), &["switch", "-q", "-c", "old"]);
    commit_file(s.other.path(), "o.txt", "남의 브랜치\n");
    git(s.other.path(), &["push", "-q", "origin", "old"]);
    let theirs = git(s.other.path(), &["rev-parse", "old"]);
    // 받은 저장소의 설정이 모든 브랜치를 강제로 올리게 해 둔다.
    git(
        s.mine.path(),
        &["config", "remote.origin.push", "+refs/heads/*:refs/heads/*"],
    );
    git(s.mine.path(), &["config", "push.default", "matching"]);
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    sync(&core(s.mine.path()), s.mine.path()).unwrap();
    let remote_old = git(Path::new(&s.url), &["rev-parse", "old"]);
    assert_eq!(remote_old, theirs, "다른 브랜치를 덮어쓰지 않는다");
    let remote_main = git(Path::new(&s.url), &["rev-parse", "main"]);
    assert_eq!(remote_main, git(s.mine.path(), &["rev-parse", "main"]));
}

#[test]
fn first_upload_sets_upstream() {
    let s = setup();
    git(s.mine.path(), &["switch", "-q", "-c", "기능"]);
    commit_file(s.mine.path(), "d.txt", "새 브랜치\n");
    let r = sync(&core(s.mine.path()), s.mine.path()).unwrap();
    assert!(
        matches!(r, ActionResult::Synced { sent, .. } if sent >= 1),
        "{r:?}"
    );
    let up = git(s.mine.path(), &["rev-parse", "--abbrev-ref", "@{upstream}"]);
    assert_eq!(up.trim(), "origin/기능");
}

#[test]
fn no_remote_is_reported() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "main"]);
    commit_file(repo.path(), "a.txt", "a\n");
    let err = sync(&core(repo.path()), repo.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::NoRemote);
}

#[test]
fn background_fetch_does_not_block_saving() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    let core = core(s.mine.path());
    core.execute(
        s.mine.path(),
        &ActionInput::Fetch(FetchInput { background: true }),
        None,
    )
    .unwrap();
    let st = git(s.mine.path(), &["status", "-sb"]);
    assert!(st.contains("behind 1"), "{st}");
    // FETCH_HEAD를 쓰지 않는다(감시 알림이 간격마다 나지 않게).
    assert!(!s.mine.path().join(".git/FETCH_HEAD").exists());
    std::fs::write(s.mine.path().join("c.txt"), "c").unwrap();
    core.execute(
        s.mine.path(),
        &ActionInput::Commit(CommitInput {
            message: "저장".into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
    let _ = s.url;
}

fn save_in_app(core: &Core, dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join(file), text).unwrap();
    core.execute(
        dir,
        &ActionInput::Commit(CommitInput {
            message: file.into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
}

#[test]
fn undoing_a_save_after_sync_makes_a_reverting_version() {
    let s = setup();
    let core = core(s.mine.path());
    save_in_app(&core, s.mine.path(), "c.txt", "내 것\n");
    sync(&core, s.mine.path()).unwrap();
    // 올린 버전은 기록을 지우지 않고 되돌리는 새 버전으로(D-073). 원격은 다음 동기화 때 바뀐다.
    let r = core.undo(s.mine.path()).unwrap();
    assert!(r.reverted);
    assert!(!s.mine.path().join("c.txt").exists());
    let st = git(s.mine.path(), &["status", "-sb"]);
    assert!(st.contains("ahead 1"), "{st}");
}

#[test]
fn a_sync_that_stopped_does_not_block_undoing_the_save() {
    let s = setup();
    others_push(&s, "a.txt", "줄 1\n남이 고침\n줄 3\n줄 4\n줄 5\n");
    let core = core(s.mine.path());
    let before = git(s.mine.path(), &["rev-parse", "HEAD"]);
    save_in_app(
        &core,
        s.mine.path(),
        "a.txt",
        "줄 1\n내가 고침\n줄 3\n줄 4\n줄 5\n",
    );
    let r = sync(&core, s.mine.path()).unwrap();
    assert!(matches!(
        r,
        ActionResult::Synced {
            merge_started: true,
            ..
        }
    ));
    // 합치기를 그만두면, 시작만 한 동기화는 기록이 없으니 되돌리기는 그 앞의 버전 저장을 되돌린다.
    core.execute(s.mine.path(), &ActionInput::ConflictAbort, None)
        .unwrap();
    core.undo(s.mine.path()).unwrap();
    assert_eq!(git(s.mine.path(), &["rev-parse", "HEAD"]), before);
}

/// 서버가 거절하면(보호 브랜치·hook) 다시 시도하지 않고 `RemoteRefused`로 멈춘다. 원격이 앞섰을 때만
/// 한 번 더 받고 올린다(계획 010 R11). hook 실행 비트가 필요해 Unix에서만.
#[cfg(unix)]
#[test]
fn server_refusal_is_not_retried() {
    use std::os::unix::fs::PermissionsExt;
    let s = setup();
    let hook = s._remote.path().join("hooks").join("pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho run >> hook-runs\nexit 1\n").unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    let core = core(s.mine.path());
    let err = sync(&core, s.mine.path()).unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::RemoteRefused);
    let runs = std::fs::read_to_string(s._remote.path().join("hook-runs")).unwrap();
    assert_eq!(runs.lines().count(), 1, "{runs}");
}

/// 설정 "올린 버전도 바꾸기"(D-127).
fn allow_rewrite(core: &Core, on: bool) {
    core.set_settings(&AppSettingsPatch {
        allow_published_rewrite: Some(on),
        ..Default::default()
    });
}

/// 내 저장소에 `기능` 브랜치를 만들고 버전 둘을 올린다. (첫째, 둘째)
fn published_feature(s: &Setup) -> (String, String) {
    let dir = s.mine.path();
    git(dir, &["switch", "-q", "-c", "기능"]);
    commit_file(dir, "f1.txt", "기능 1\n");
    let first = git(dir, &["rev-parse", "HEAD"]).trim().to_string();
    commit_file(dir, "f2.txt", "기능 2\n");
    let second = git(dir, &["rev-parse", "HEAD"]).trim().to_string();
    git(dir, &["push", "-q", "-u", "origin", "기능"]);
    (first, second)
}

fn head(dir: &Path) -> String {
    git(dir, &["rev-parse", "HEAD"]).trim().to_string()
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn sync_anyway(core: &Core, dir: &Path) -> Result<ActionResult, higit_core::CoreError> {
    core.execute(
        dir,
        &ActionInput::Sync(SyncInput {
            locale: "ko".into(),
            merge_anyway: true,
        }),
        None,
    )
}

#[test]
fn rewritten_published_branch_does_not_take_old_versions_back() {
    let s = setup();
    let dir = s.mine.path();
    let (_, second) = published_feature(&s);
    let core = core(dir);
    allow_rewrite(&core, true);
    core.execute(
        dir,
        &ActionInput::Amend(AmendInput {
            expect_head: second,
            message: Some("고친 이름".into()),
        }),
        None,
    )
    .unwrap();
    let amended = head(dir);
    // 갈라졌지만 원격의 옛 버전을 합치지 않고 멈춘다.
    assert_eq!(
        code(&sync(&core, dir).unwrap_err()),
        ErrorCode::RewrotePublished
    );
    assert_eq!(head(dir), amended);
    // 설정을 꺼도 표식은 남아 막는다.
    allow_rewrite(&core, false);
    assert_eq!(
        code(&sync(&core, dir).unwrap_err()),
        ErrorCode::RewrotePublished
    );
    // "그래도 합치기"면 받아 합치고 올린다. 그 뒤로는 평범한 동기화.
    let r = sync_anyway(&core, dir).unwrap();
    assert!(
        matches!(r, ActionResult::Synced { merged: true, .. }),
        "{r:?}"
    );
    assert!(!dir.join(".git/higit/rewrote.json").exists());
    sync(&core, dir).unwrap();
}

#[test]
fn going_back_on_a_published_branch_stops_sync_and_pull() {
    let s = setup();
    let dir = s.mine.path();
    let (first, _) = published_feature(&s);
    let core = core(dir);
    allow_rewrite(&core, true);
    core.execute(
        dir,
        &ActionInput::Reset(OidInput { oid: first.clone() }),
        None,
    )
    .unwrap();
    // 원격만 앞선 모양이지만 앞으로 감기(옛 버전 되살리기)를 하지 않는다.
    assert_eq!(
        code(&sync(&core, dir).unwrap_err()),
        ErrorCode::RewrotePublished
    );
    assert_eq!(
        code(&core.execute(dir, &ActionInput::Pull, None).unwrap_err()),
        ErrorCode::RewrotePublished
    );
    assert_eq!(head(dir), first);
}

#[test]
fn deleting_the_branch_forgets_its_mark() {
    let s = setup();
    let dir = s.mine.path();
    let (_, second) = published_feature(&s);
    let core = core(dir);
    allow_rewrite(&core, true);
    core.execute(
        dir,
        &ActionInput::Amend(AmendInput {
            expect_head: second,
            message: Some("고친 이름".into()),
        }),
        None,
    )
    .unwrap();
    assert!(dir.join(".git/higit/rewrote.json").exists());
    git(dir, &["switch", "-q", "main"]);
    core.execute(
        dir,
        &ActionInput::BranchDelete(BranchNameInput {
            name: "기능".into(),
        }),
        None,
    )
    .unwrap();
    assert!(!dir.join(".git/higit/rewrote.json").exists());
}

#[test]
fn ordinary_divergence_still_merges_with_the_setting_on() {
    let s = setup();
    others_push(&s, "b.txt", "남의 것\n");
    commit_file(s.mine.path(), "c.txt", "내 것\n");
    let core = core(s.mine.path());
    allow_rewrite(&core, true);
    let r = sync(&core, s.mine.path()).unwrap();
    assert!(
        matches!(r, ActionResult::Synced { merged: true, .. }),
        "{r:?}"
    );
}
