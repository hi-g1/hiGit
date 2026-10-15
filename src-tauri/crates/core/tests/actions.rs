//! 저장에 넣기·빼기·저장하기 액션.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod common;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use common::{FakeGit, folder_state, repo_state};
use higit_core::view::{ActionInput, ActionResult, CommitInput, ErrorCode, ErrorView, PathsInput};
use higit_core::{
    Core, CoreError, ExecCtx, Executor, GitBackend, Outcome, Registry, UnavailableReason, actions,
};

fn executor() -> Executor {
    let mut registry = Registry::new();
    actions::register_all(&mut registry).unwrap();
    Executor::new(registry)
}

fn paths(p: &[&str]) -> PathsInput {
    PathsInput {
        paths: p.iter().map(|s| s.to_string()).collect(),
    }
}

fn commit(message: &str) -> ActionInput {
    ActionInput::Commit(CommitInput {
        message: message.into(),
    })
}

#[test]
fn stage_and_unstage_call_git_with_paths() {
    let exec = executor();
    let git = FakeGit::new();
    let state = repo_state("/repo");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };

    let out = exec
        .execute(
            &ctx,
            "stage",
            &ActionInput::Stage(paths(&["한글 파일.txt", "b.md"])),
            None,
        )
        .unwrap();
    assert_eq!(out, Outcome::Staged { count: 2 });
    let out = exec
        .execute(
            &ctx,
            "unstage",
            &ActionInput::Unstage(paths(&["b.md"])),
            None,
        )
        .unwrap();
    assert_eq!(out, Outcome::Unstaged { count: 1 });
    assert_eq!(
        git.calls(),
        [
            "Stage { paths: [\"한글 파일.txt\", \"b.md\"] } /repo",
            "Unstage { paths: [\"b.md\"] } /repo",
        ]
    );
}

#[test]
fn commit_returns_new_summary() {
    let exec = executor();
    let git = FakeGit::new();
    let state = repo_state("/repo");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };

    let out = exec
        .execute(&ctx, "commit", &commit("로그인 화면\n\n설명"), None)
        .unwrap();
    let Outcome::Committed { summary, .. } = out else {
        panic!("{out:?}")
    };
    assert_eq!(summary, "로그인 화면");
}

#[test]
fn inputs_are_checked_before_git() {
    let exec = executor();
    let git = FakeGit::new();
    let state = repo_state("/repo");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };
    let invalid = |r: higit_core::Result<Outcome>| matches!(r, Err(CoreError::InvalidInput(_)));

    assert!(invalid(exec.execute(
        &ctx,
        "stage",
        &ActionInput::Stage(paths(&[])),
        None
    )));
    assert!(invalid(exec.execute(
        &ctx,
        "stage",
        &ActionInput::Stage(paths(&["../밖.txt"])),
        None
    )));
    assert!(invalid(exec.execute(
        &ctx,
        "unstage",
        &ActionInput::Unstage(paths(&["C:/x"])),
        None
    )));
    assert!(invalid(exec.execute(
        &ctx,
        "commit",
        &commit("  \n "),
        None
    )));
    assert!(matches!(
        exec.execute(&ctx, "stage", &commit("메시지"), None),
        Err(CoreError::InputMismatch("stage"))
    ));
    assert!(git.calls().is_empty());
}

#[test]
fn actions_need_a_repo() {
    let exec = executor();
    let git = FakeGit::new();
    let state = folder_state("/그냥 폴더");
    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };
    for (id, input) in [
        ("stage", ActionInput::Stage(paths(&["a"]))),
        ("unstage", ActionInput::Unstage(paths(&["a"]))),
        ("commit", commit("저장")),
    ] {
        let err = exec.execute(&ctx, id, &input, None).unwrap_err();
        assert!(
            matches!(err, CoreError::Unavailable(UnavailableReason::NotARepo)),
            "{id}"
        );
    }
}

#[test]
fn same_repo_runs_one_action_at_a_time() {
    let dir = tempfile::tempdir().unwrap();
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = Some(higit_core::RepoPath::new(dir.path()));
    let core = Arc::new(Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap());
    core.open_project(dir.path(), higit_core::view::OpenHow::Picked)
        .unwrap();

    git.hold.store(true, Ordering::SeqCst);
    let first = {
        let core = core.clone();
        let path = dir.path().to_path_buf();
        std::thread::spawn(move || {
            core.execute(&path, &ActionInput::Stage(paths(&["a.txt"])), None)
        })
    };
    while !git.entered.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let err = core
        .execute(dir.path(), &ActionInput::Stage(paths(&["b.txt"])), None)
        .unwrap_err();
    assert_eq!(ErrorView::from(&err).code, ErrorCode::Busy);

    git.hold.store(false, Ordering::SeqCst);
    assert_eq!(
        first.join().unwrap().unwrap(),
        ActionResult::Staged { count: 1 }
    );
    // 끝나면 잠금이 풀린다.
    core.execute(dir.path(), &ActionInput::Stage(paths(&["b.txt"])), None)
        .unwrap();
}
