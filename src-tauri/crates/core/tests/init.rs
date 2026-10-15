//! 실제 git으로 관리 시작(init)을 실행한다.

use std::path::PathBuf;
use std::sync::Arc;

use git::CliBackend;
use higit_core::view::ActionInput;
use higit_core::{ExecCtx, Executor, GitBackend, Outcome, Registry, RepoState, actions};

#[test]
fn init_turns_folder_into_repo() {
    let dir = tempfile::tempdir().unwrap();
    let folder: PathBuf = dir.path().join("새 프로젝트");
    std::fs::create_dir(&folder).unwrap();

    let git = CliBackend::new("git", Arc::new(|_| {}));
    let mut registry = Registry::new();
    actions::register_all(&mut registry).unwrap();
    let exec = Executor::new(registry);

    let state = RepoState {
        folder: folder.clone(),
        repo: git.discover(&folder).unwrap(),
    };
    assert_eq!(state.repo, None);

    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };
    let Outcome::RepoCreated(repo) = exec
        .execute(&ctx, "init", &ActionInput::Init, None)
        .unwrap();
    assert_eq!(
        std::fs::canonicalize(repo.as_path()).unwrap(),
        std::fs::canonicalize(&folder).unwrap()
    );
}
