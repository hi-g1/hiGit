//! 복제 액션(D-115): 주소·대상 검사, 실패 정리, 취소 키, 끝난 뒤 열린 목록.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use common::FakeGit;
use higit_core::view::{ActionInput, ActionResult, CloneInput, ErrorCode, ErrorView, OpenHow};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(["-c", "user.name=민지", "-c", "user.email=m@example.com"])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// 버전 하나가 든 bare 원격.
fn remote() -> tempfile::TempDir {
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
    let work = tempfile::tempdir().unwrap();
    git(work.path(), &["init", "-q", "-b", "main"]);
    std::fs::write(work.path().join("a.txt"), "한 줄\n").unwrap();
    git(work.path(), &["add", "-A"]);
    git(work.path(), &["commit", "-q", "-m", "첫 버전"]);
    let url = remote.path().to_string_lossy().into_owned();
    git(work.path(), &["push", "-q", &url, "main"]);
    remote
}

fn real_core() -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {})).allow_local_remotes_for_tests();
    Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>)
        .unwrap()
        .allow_local_clone_for_tests()
}

fn input(url: &str, name: &str) -> ActionInput {
    ActionInput::Clone(CloneInput {
        url: url.into(),
        name: name.into(),
        accept_old_git: true,
    })
}

fn code(err: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(err).code
}

#[test]
fn clones_into_a_new_folder_and_opens_it() {
    let remote = remote();
    let parent = tempfile::tempdir().unwrap();
    let core = real_core();
    let url = remote.path().to_string_lossy().into_owned();
    let result = core
        .execute(parent.path(), &input(&url, "받은 저장소"), None)
        .unwrap();
    let ActionResult::RepoCreated { repo } = result else {
        panic!("{result:?}");
    };
    let target = parent.path().join("받은 저장소");
    assert!(target.join("a.txt").is_file());
    assert_eq!(
        Path::new(&repo.path),
        platform::normalize_repo_path(&target).unwrap()
    );
    // 열린 목록에 들어가 바로 조회할 수 있고, 화면은 이어서 project_open을 거친다.
    core.queries().status(&target).unwrap();
    let opened = core.open_project(&target, OpenHow::Picked).unwrap();
    assert!(matches!(
        opened,
        higit_core::view::ProjectOpenResult::Opened { .. }
    ));
    assert!(
        core.recent_projects()
            .iter()
            .any(|p| Path::new(&p.path) == Path::new(&repo.path))
    );
}

#[test]
fn existing_empty_folder_is_fine_but_full_one_is_refused() {
    let remote = remote();
    let url = remote.path().to_string_lossy().into_owned();
    let parent = tempfile::tempdir().unwrap();
    let core = real_core();
    std::fs::create_dir(parent.path().join("empty")).unwrap();
    core.execute(parent.path(), &input(&url, "empty"), None)
        .unwrap();
    assert!(parent.path().join("empty").join("a.txt").is_file());

    std::fs::create_dir(parent.path().join("full")).unwrap();
    std::fs::write(parent.path().join("full").join("mine.txt"), "내 것").unwrap();
    let err = core
        .execute(parent.path(), &input(&url, "full"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::CloneTargetExists);
    assert!(parent.path().join("full").join("mine.txt").is_file());

    std::fs::write(parent.path().join("file"), "파일").unwrap();
    let err = core
        .execute(parent.path(), &input(&url, "file"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::CloneTargetExists);
}

#[test]
fn failed_clone_removes_only_what_it_created() {
    let parent = tempfile::tempdir().unwrap();
    let core = real_core();
    let missing = parent
        .path()
        .join("없는 원격")
        .to_string_lossy()
        .into_owned();
    let err = core
        .execute(parent.path(), &input(&missing, "new"), None)
        .unwrap_err();
    assert_ne!(code(&err), ErrorCode::Internal);
    assert!(!parent.path().join("new").exists());

    // 원래 있던 빈 폴더는 남긴다.
    std::fs::create_dir(parent.path().join("kept")).unwrap();
    core.execute(parent.path(), &input(&missing, "kept"), None)
        .unwrap_err();
    assert!(parent.path().join("kept").is_dir());
}

#[test]
fn inputs_are_checked_before_git_runs() {
    let git = Arc::new(FakeGit::new());
    let core = Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap();
    let parent = tempfile::tempdir().unwrap();
    let cases = [
        (input("file:///etc", "a"), ErrorCode::CloneUrlInvalid),
        (input("ext::sh -c x", "a"), ErrorCode::CloneUrlInvalid),
        (input("/local/path", "a"), ErrorCode::CloneUrlInvalid),
        (
            input("https://user:pw@github.com/a/b", "a"),
            ErrorCode::CloneUrlInvalid,
        ),
        (
            input("https://github.com/a/b", "../escape"),
            ErrorCode::CloneNameInvalid,
        ),
        (
            input("https://github.com/a/b", "CON"),
            ErrorCode::CloneNameInvalid,
        ),
        (
            input("https://github.com/a/b", ".git"),
            ErrorCode::CloneNameInvalid,
        ),
    ];
    for (input, want) in cases {
        let err = core.execute(parent.path(), &input, None).unwrap_err();
        assert_eq!(code(&err), want, "{input:?}");
    }
    // 부모가 저장소의 .git 안이면 거절.
    let inside = parent.path().join("repo").join(".git").join("x");
    std::fs::create_dir_all(&inside).unwrap();
    let err = core
        .execute(&inside, &input("https://github.com/a/b", "c"), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::InvalidInput);
    // 없는 부모
    let err = core
        .execute(
            &parent.path().join("없음"),
            &input("https://github.com/a/b", "c"),
            None,
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::FolderNotFound);
    assert!(git.calls().iter().all(|c| !c.starts_with("Clone")));
}

#[cfg(unix)]
#[test]
fn linked_parent_is_refused() {
    let git = Arc::new(FakeGit::new());
    let core = Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("real")).unwrap();
    std::os::unix::fs::symlink(dir.path().join("real"), dir.path().join("link")).unwrap();
    let err = core
        .execute(
            &dir.path().join("link"),
            &input("https://github.com/a/b", "c"),
            None,
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::FolderNotFound);
    assert!(git.calls().iter().all(|c| !c.starts_with("Clone")));
}

#[test]
fn outdated_git_needs_consent() {
    // FakeGit은 2.45.0(보안 수정 기준 2.45.4보다 낮음), 배포판 패키지가 아님.
    let git = Arc::new(FakeGit::new());
    let core = Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap();
    let parent = tempfile::tempdir().unwrap();
    let mut clone = CloneInput {
        url: "https://github.com/octo/hello.git".into(),
        name: "hello".into(),
        accept_old_git: false,
    };
    let err = core
        .execute(parent.path(), &ActionInput::Clone(clone.clone()), None)
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::GitOutdated);
    clone.accept_old_git = true;
    // FakeGit은 폴더를 만들지 않으므로 결과 경로만 확인한다.
    let result = core
        .execute(parent.path(), &ActionInput::Clone(clone), None)
        .unwrap();
    assert!(matches!(result, ActionResult::RepoCreated { .. }));
    let calls = git.calls();
    let clone_call = calls.iter().find(|c| c.starts_with("Clone")).unwrap();
    assert!(clone_call.contains("auth: UserDefault"), "{clone_call}");
}

#[test]
fn cancel_matches_the_target_folder_exactly() {
    let git = Arc::new(FakeGit::new());
    let core = Arc::new(Core::with_backend(git.clone() as Arc<dyn GitBackend>).unwrap());
    let parent = tempfile::tempdir().unwrap();
    let target = parent.path().join("hello");
    git.hold.store(true, Ordering::SeqCst);
    let runner = {
        let core = core.clone();
        let parent = parent.path().to_path_buf();
        std::thread::spawn(move || {
            core.execute(
                &parent,
                &input("https://github.com/octo/hello", "hello"),
                None,
            )
        })
    };
    while !git.entered.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    // 같은 대상에 두 번째 복제는 Busy.
    let err = core
        .execute(
            parent.path(),
            &input("https://github.com/octo/hello", "hello"),
            None,
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::Busy);
    // 부모 폴더(열린 폴더도 아님)로는 복제를 취소하지 않는다.
    let _ = core.cancel(parent.path());
    assert!(git.calls().iter().all(|c| !c.starts_with("cancel")));
    // 대상 폴더(아직 없어도)로 취소한다.
    core.cancel(&target).unwrap();
    let canonical = platform::normalize_repo_path(parent.path())
        .unwrap()
        .join("hello");
    assert!(
        git.calls()
            .iter()
            .any(|c| c == &format!("cancel {}", canonical.display())),
        "{:?}",
        git.calls()
    );
    git.hold.store(false, Ordering::SeqCst);
    runner.join().unwrap().unwrap();
}

#[test]
fn defaults_point_to_home_folder() {
    let git = Arc::new(FakeGit::new());
    let core = Core::with_backend(git as Arc<dyn GitBackend>).unwrap();
    let defaults = core.queries().clone_defaults();
    assert_eq!(
        defaults.parent,
        platform::home_dir().map(|h| h.to_string_lossy().into_owned())
    );
}
