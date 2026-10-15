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
        .unwrap()
    else {
        panic!("init 결과가 아님")
    };
    assert_eq!(
        std::fs::canonicalize(repo.as_path()).unwrap(),
        std::fs::canonicalize(&folder).unwrap()
    );
}

#[test]
fn stage_and_commit_with_real_git() {
    use higit_core::view::{CommitInput, PathsInput};

    let dir = tempfile::tempdir().unwrap();
    let folder: PathBuf = dir.path().to_path_buf();
    let git = CliBackend::new("git", Arc::new(|_| {}));
    let mut registry = Registry::new();
    actions::register_all(&mut registry).unwrap();
    let exec = Executor::new(registry);

    let state = RepoState {
        folder: folder.clone(),
        repo: None,
    };
    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };
    let Outcome::RepoCreated(repo) = exec
        .execute(&ctx, "init", &ActionInput::Init, None)
        .unwrap()
    else {
        panic!()
    };
    for (key, value) in [("user.name", "테스트"), ("user.email", "t@example.com")] {
        std::process::Command::new("git")
            .args(["config", key, value])
            .current_dir(&folder)
            .status()
            .unwrap();
    }
    std::fs::write(folder.join("한글 파일.txt"), "내용\n").unwrap();

    let state = RepoState {
        folder: folder.clone(),
        repo: Some(repo),
    };
    let ctx = ExecCtx {
        git: &git,
        state: &state,
    };
    exec.execute(
        &ctx,
        "stage",
        &ActionInput::Stage(PathsInput {
            paths: vec!["한글 파일.txt".into()],
        }),
        None,
    )
    .unwrap();
    let out = exec
        .execute(
            &ctx,
            "commit",
            &ActionInput::Commit(CommitInput {
                message: "첫 저장".into(),
            }),
            None,
        )
        .unwrap();
    let Outcome::Committed { summary, short_oid } = out else {
        panic!("{out:?}")
    };
    assert_eq!(summary, "첫 저장");
    assert!(!short_oid.is_empty());
}
