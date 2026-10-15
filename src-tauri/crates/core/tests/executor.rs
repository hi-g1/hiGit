#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use common::{FakeAction, FakeGit, folder_state, repo_state};
use higit_core::view::ActionInput;
use higit_core::{
    Action, CoreError, ExecCtx, Executor, Outcome, PreviewToken, Registry, Risk, UnavailableReason,
    actions,
};

/// 가짜 액션을 등록하고, 테스트에서 실행 횟수를 볼 수 있게 Arc로 함께 돌려준다.
struct Shared(Arc<FakeAction>);

impl Action for Shared {
    fn meta(&self) -> &higit_core::ActionMeta {
        self.0.meta()
    }
    fn available(&self, s: &higit_core::RepoState) -> higit_core::Availability {
        self.0.available(s)
    }
    fn execute(&self, c: &ExecCtx<'_>, i: &ActionInput) -> higit_core::Result<Outcome> {
        self.0.execute(c, i)
    }
}

fn executor_with(
    risks: &[(&'static str, Risk)],
    ttl: Duration,
) -> (Executor, Vec<Arc<FakeAction>>) {
    let mut registry = Registry::new();
    let mut handles = Vec::new();
    for (id, risk) in risks {
        let action = Arc::new(FakeAction::new(id, *risk));
        registry.register(Box::new(Shared(action.clone()))).unwrap();
        handles.push(action);
    }
    (Executor::with_ttl(registry, ttl), handles)
}

const TTL: Duration = Duration::from_secs(60);

#[test]
fn duplicate_id_is_rejected() {
    let mut registry = Registry::new();
    registry
        .register(Box::new(FakeAction::new("a", Risk::Safe)))
        .unwrap();
    let err = registry
        .register(Box::new(FakeAction::new("a", Risk::Normal)))
        .unwrap_err();
    assert!(matches!(err, CoreError::DuplicateAction("a")));
}

#[test]
fn unknown_action_is_rejected() {
    let (exec, _) = executor_with(&[], TTL);
    let git = FakeGit::new();
    let state = folder_state("/p");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let err = exec
        .execute(&ctx, "없음", &ActionInput::Init, None)
        .unwrap_err();
    assert!(matches!(err, CoreError::UnknownAction(_)));
}

#[test]
fn safe_and_normal_run_without_preview() {
    let (exec, actions) = executor_with(&[("safe", Risk::Safe), ("normal", Risk::Normal)], TTL);
    let git = FakeGit::new();
    let state = folder_state("/p");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };

    exec.execute(&ctx, "safe", &ActionInput::Init, None)
        .unwrap();
    exec.execute(&ctx, "normal", &ActionInput::Init, None)
        .unwrap();
    assert_eq!(actions[0].executed.load(Ordering::SeqCst), 1);
    assert_eq!(actions[1].executed.load(Ordering::SeqCst), 1);
}

const HEAD: &str = "1111111111111111111111111111111111111111";

fn git_at_head() -> FakeGit {
    let git = FakeGit::new();
    git.status.lock().unwrap().branch.oid = Some(HEAD.into());
    git
}

fn backups(git: &FakeGit) -> Vec<String> {
    git.refs
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.name.starts_with("refs/higit/backup/"))
        .map(|r| r.oid.clone())
        .collect()
}

#[test]
fn caution_and_danger_back_up_head_before_running() {
    let (exec, actions) =
        executor_with(&[("caution", Risk::Caution), ("danger", Risk::Danger)], TTL);
    let git = git_at_head();
    let dir = tempfile::tempdir().unwrap();
    let state = repo_state(&dir.path().to_string_lossy());
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let input = ActionInput::Init;

    for (i, id) in ["caution", "danger"].into_iter().enumerate() {
        // 되돌릴 수 없으니 미리보기(확인) 없이는 거부
        let err = exec.execute(&ctx, id, &input, None).unwrap_err();
        assert!(matches!(err, CoreError::PreviewRequired), "{id}: {err:?}");
        assert_eq!(actions[i].executed.load(Ordering::SeqCst), 0, "{id}");

        // 확인을 거치면 HEAD를 백업한 뒤 실행한다(D-043)
        let (_, token) = exec.preview(&ctx, id, &input).unwrap();
        exec.execute(&ctx, id, &input, Some(&token)).unwrap();
        assert_eq!(actions[i].executed.load(Ordering::SeqCst), 1, "{id}");
        assert_eq!(backups(&git).len(), i + 1, "{id}");
    }
    assert!(backups(&git).iter().all(|oid| oid == HEAD));
    // 되돌릴 수 없는 일은 경계 줄을 남긴다(D-065).
    let journal = std::fs::read_to_string(dir.path().join(".git/higit/journal.jsonl")).unwrap();
    assert_eq!(
        journal.matches("\"type\":\"barrier\"").count(),
        2,
        "{journal}"
    );
}

#[test]
fn no_backup_means_no_run() {
    let (exec, actions) = executor_with(&[("caution", Risk::Caution)], TTL);
    let git = git_at_head();
    git.fail_update_ref.store(true, Ordering::SeqCst);
    let state = repo_state("/p");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let (_, token) = exec.preview(&ctx, "caution", &ActionInput::Init).unwrap();
    assert!(
        exec.execute(&ctx, "caution", &ActionInput::Init, Some(&token))
            .is_err()
    );
    assert_eq!(actions[0].executed.load(Ordering::SeqCst), 0);
}

#[test]
fn caution_without_any_version_runs_without_backup() {
    let (exec, actions) = executor_with(&[("caution", Risk::Caution)], TTL);
    let git = FakeGit::new();
    let state = repo_state("/p");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let (_, token) = exec.preview(&ctx, "caution", &ActionInput::Init).unwrap();
    exec.execute(&ctx, "caution", &ActionInput::Init, Some(&token))
        .unwrap();
    assert_eq!(actions[0].executed.load(Ordering::SeqCst), 1);
    assert!(backups(&git).is_empty());
}

#[test]
fn preview_token_must_match() {
    let (exec, _) = executor_with(&[("caution", Risk::Caution), ("other", Risk::Caution)], TTL);
    let git = FakeGit::new();
    let state = folder_state("/p");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let input = ActionInput::Init;
    let mismatch = |r: higit_core::Result<Outcome>| matches!(r, Err(CoreError::PreviewMismatch));

    // 없는 토큰
    let fake = PreviewToken("p999".into());
    assert!(mismatch(exec.execute(&ctx, "caution", &input, Some(&fake))));

    // 입력이 다름
    let message = |m: &str| {
        ActionInput::Commit(higit_core::view::CommitInput {
            message: m.into(),
            stage_all: false,
        })
    };
    let (_, token) = exec.preview(&ctx, "caution", &message("가")).unwrap();
    assert!(mismatch(exec.execute(
        &ctx,
        "caution",
        &message("나"),
        Some(&token)
    )));

    // 다른 액션의 토큰
    let (_, token) = exec.preview(&ctx, "other", &input).unwrap();
    assert!(mismatch(exec.execute(
        &ctx,
        "caution",
        &input,
        Some(&token)
    )));

    // 다른 폴더에서 받은 토큰
    let other_state = folder_state("/다른 폴더");
    let other_ctx = ExecCtx {
        git: &git,
        state: &other_state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let (_, token) = exec.preview(&other_ctx, "caution", &input).unwrap();
    assert!(mismatch(exec.execute(
        &ctx,
        "caution",
        &input,
        Some(&token)
    )));

    // 한 번 쓴 토큰은 다시 못 쓴다
    let (_, token) = exec.preview(&ctx, "caution", &input).unwrap();
    let _ = exec.execute(&ctx, "caution", &input, Some(&token));
    assert!(mismatch(exec.execute(
        &ctx,
        "caution",
        &input,
        Some(&token)
    )));
}

#[test]
fn expired_token_is_rejected() {
    let (exec, _) = executor_with(&[("caution", Risk::Caution)], Duration::ZERO);
    let git = FakeGit::new();
    let state = folder_state("/p");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let (_, token) = exec.preview(&ctx, "caution", &ActionInput::Init).unwrap();
    let err = exec
        .execute(&ctx, "caution", &ActionInput::Init, Some(&token))
        .unwrap_err();
    assert!(matches!(err, CoreError::PreviewMismatch));
}

#[test]
fn init_runs_only_outside_repo() {
    let mut registry = Registry::new();
    actions::register_all(&mut registry).unwrap();
    let exec = Executor::new(registry);
    let git = FakeGit::new();

    let in_repo = repo_state("/이미 저장소");
    let ctx = ExecCtx {
        git: &git,
        state: &in_repo,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let err = exec
        .execute(&ctx, "init", &ActionInput::Init, None)
        .unwrap_err();
    assert!(matches!(
        err,
        CoreError::Unavailable(UnavailableReason::AlreadyRepo)
    ));
    assert!(git.calls().is_empty());

    let folder = folder_state("/새 폴더");
    let ctx = ExecCtx {
        git: &git,
        state: &folder,
        key: &higit_core::undo::JournalKey::ephemeral(),
    };
    let outcome = exec
        .execute(&ctx, "init", &ActionInput::Init, None)
        .unwrap();
    assert_eq!(git.calls(), ["Init /새 폴더"]);
    assert_eq!(
        outcome,
        Outcome::RepoCreated(higit_core::RepoPath::new("/새 폴더"))
    );
}

#[test]
fn registered_actions_match_spec_risk() {
    let mut registry = Registry::new();
    actions::register_all(&mut registry).unwrap();
    let metas: Vec<_> = registry
        .metas()
        .map(|m| (m.id, m.glossary_id, m.risk))
        .collect();
    assert_eq!(
        metas,
        [
            ("init", "init", Risk::Safe),
            ("stage", "stage", Risk::Safe),
            ("unstage", "unstage", Risk::Safe),
            ("commit", "commit", Risk::Safe),
            ("ignore", "ignore", Risk::Safe),
            ("merge", "merge", Risk::Normal),
            ("merge-into", "merge", Risk::Normal),
            ("conflict-finish", "conflict", Risk::Normal),
            ("conflict-abort", "merge", Risk::Normal),
            ("stash", "stash", Risk::Safe),
            ("take-out", "stash", Risk::Safe),
            ("discard", "discard", Risk::Caution),
            ("restore", "reflog", Risk::Safe),
            ("clear-lock", "lock", Risk::Caution),
            ("recover-parked", "park", Risk::Safe),
            ("switch", "switch", Risk::Normal),
            ("branch-create", "branch", Risk::Safe),
            ("fetch", "fetch", Risk::Safe),
            ("pull", "pull", Risk::Normal),
            ("push", "push", Risk::Normal),
            ("sync", "sync", Risk::Normal),
        ]
    );
}
