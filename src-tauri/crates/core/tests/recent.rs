//! 최근 프로젝트 목록과 프로젝트 열기(D-032).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::FakeGit;
use higit_core::recent::{MAX_RECENT, RecentStore};
use higit_core::view::{ActionInput, OpenHow, ProjectOpenResult};
use higit_core::{Core, GitBackend, RepoPath};

fn real(p: &Path) -> PathBuf {
    platform::normalize_repo_path(p).unwrap()
}

/// discover가 `repo`를 돌려주는 가짜 git과, `data` 폴더에 목록을 두는 Core.
fn core(repo: Option<&Path>, data: &Path) -> (Core, Arc<FakeGit>) {
    let git = Arc::new(FakeGit::new());
    *git.repo.lock().unwrap() = repo.map(|p| RepoPath::new(real(p)));
    let core =
        Core::with_backend_and_recent(git.clone() as Arc<dyn GitBackend>, RecentStore::open(data))
            .unwrap();
    (core, git)
}

#[test]
fn opening_adds_to_front_once() {
    let data = tempfile::tempdir().unwrap();
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let (core, git) = core(Some(a.path()), data.path());

    core.open_project(a.path(), OpenHow::Picked).unwrap();
    *git.repo.lock().unwrap() = Some(RepoPath::new(real(b.path())));
    core.open_project(b.path(), OpenHow::Picked).unwrap();
    *git.repo.lock().unwrap() = Some(RepoPath::new(real(a.path())));
    core.open_project(a.path(), OpenHow::Recent).unwrap();

    let list = core.recent_projects();
    let paths: Vec<_> = list.iter().map(|p| PathBuf::from(&p.path)).collect();
    assert_eq!(paths, [real(a.path()), real(b.path())]);

    // 다시 켜도(새 Core) 파일에서 읽는다.
    let (again, _) = core_with_store(data.path());
    assert_eq!(again.recent_projects().len(), 2);
}

fn core_with_store(data: &Path) -> (Core, Arc<FakeGit>) {
    core(None, data)
}

#[test]
fn keeps_at_most_twenty() {
    let data = tempfile::tempdir().unwrap();
    let store = RecentStore::open(data.path());
    let dirs: Vec<_> = (0..MAX_RECENT + 1)
        .map(|_| tempfile::tempdir().unwrap())
        .collect();
    for d in &dirs {
        store.touch(&real(d.path()));
    }
    let list = store.list();
    assert_eq!(list.len(), MAX_RECENT);
    assert_eq!(
        PathBuf::from(&list[0].path),
        real(dirs.last().unwrap().path())
    );
}

#[test]
fn broken_or_unknown_file_starts_empty() {
    for body in ["{ 깨진", r#"{"version": 99, "projects": []}"#] {
        let data = tempfile::tempdir().unwrap();
        std::fs::write(data.path().join("recent.json"), body).unwrap();
        let store = RecentStore::open(data.path());
        assert!(store.list().is_empty());
        assert!(data.path().join("recent.json.bad").exists(), "{body}");
    }
}

#[test]
fn missing_and_not_a_repo() {
    let data = tempfile::tempdir().unwrap();
    let plain = tempfile::tempdir().unwrap();
    let (core, _) = core(None, data.path());

    let gone = plain.path().join("없는 폴더");
    assert_eq!(
        core.open_project(&gone, OpenHow::Recent).unwrap(),
        ProjectOpenResult::Missing
    );
    let r = core.open_project(plain.path(), OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::NotARepo { .. }), "{r:?}");
    assert!(core.recent_projects().is_empty());
}

#[test]
fn recent_does_not_open_outer_repo_when_dot_git_is_gone() {
    // 바깥 저장소 안의 하위 폴더를 목록에 기록해 둔 상태에서, 하위 폴더의 .git만 지워졌다.
    let data = tempfile::tempdir().unwrap();
    let outer = tempfile::tempdir().unwrap();
    let inner = outer.path().join("안쪽");
    std::fs::create_dir(&inner).unwrap();
    let (core, _) = core(Some(outer.path()), data.path());

    let r = core.open_project(&inner, OpenHow::Recent).unwrap();
    assert!(matches!(r, ProjectOpenResult::NotARepo { .. }), "{r:?}");
    // 사용자가 그 폴더를 직접 고르면 바깥 저장소를 연다(지금 동작 유지).
    let r = core.open_project(&inner, OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
}

#[test]
fn init_records_the_new_repo() {
    let data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let (core, _) = core(None, data.path());
    core.open_project(folder.path(), OpenHow::Picked).unwrap();
    core.execute(folder.path(), &ActionInput::Init, None)
        .unwrap();
    assert_eq!(core.recent_projects().len(), 1);
}

#[test]
fn remove_from_list() {
    let data = tempfile::tempdir().unwrap();
    let a = tempfile::tempdir().unwrap();
    let (core, _) = core(Some(a.path()), data.path());
    core.open_project(a.path(), OpenHow::Picked).unwrap();
    core.remove_recent(&real(a.path()));
    assert!(core.recent_projects().is_empty());
}

#[test]
fn write_failure_does_not_break_opening() {
    // 목록 폴더가 사라져도(쓸 수 없어도) 프로젝트는 열린다.
    let data = tempfile::tempdir().unwrap();
    let a = tempfile::tempdir().unwrap();
    let (core, _) = core(Some(a.path()), data.path());
    drop(data);
    let r = core.open_project(a.path(), OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }));
}

#[test]
fn app_id_matches_tauri_config() {
    let conf = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tauri.conf.json"),
    )
    .unwrap();
    let conf: serde_json::Value = serde_json::from_str(&conf).unwrap();
    assert_eq!(conf["identifier"], higit_core::APP_ID);
}

/// 고정(D-203): 옛 파일 읽기, 물려받기, 한도, 최근 한도 밖에서도 남기.
mod pinning {
    use super::*;
    use higit_core::recent::{MAX_PINNED, PinError};
    use higit_core::view::{ErrorCode, ErrorView};

    #[test]
    fn old_files_without_pins_still_read() {
        let data = tempfile::tempdir().unwrap();
        std::fs::write(
            data.path().join("recent.json"),
            r#"{"version":1,"projects":[{"path":"/x/a","name":"a","lastOpenedAt":5}]}"#,
        )
        .unwrap();
        let store = RecentStore::open(data.path());
        let list = store.list();
        assert_eq!(list.len(), 1);
        assert!(!list[0].pinned && list[0].pinned_branches.is_empty());
        // 고정이 없으면 새 칸을 쓰지 않아 옛 앱도 그대로 읽는다.
        store.pin(Path::new("/x/a"), false).unwrap();
        let text = std::fs::read_to_string(data.path().join("recent.json")).unwrap();
        assert!(!text.contains("pinned"), "{text}");
    }

    #[test]
    fn pins_survive_reopening_and_the_recent_limit() {
        let data = tempfile::tempdir().unwrap();
        let store = RecentStore::open(data.path());
        let dirs: Vec<_> = (0..MAX_PINNED + MAX_RECENT + 3)
            .map(|_| tempfile::tempdir().unwrap())
            .collect();
        for d in &dirs[..MAX_PINNED + 1] {
            store.touch(&real(d.path()));
        }
        for d in &dirs[..MAX_PINNED] {
            store.pin(&real(d.path()), true).unwrap();
        }
        assert_eq!(
            store.pin(&real(dirs[MAX_PINNED].path()), true),
            Err(PinError::Limit)
        );
        // 고정한 것을 다시 열어도 고정은 남는다.
        store.touch(&real(dirs[0].path()));
        store
            .pin_branch(&real(dirs[0].path()), "기능/로그인", true)
            .unwrap();
        store.touch(&real(dirs[0].path()));
        // 최근 21개 이상을 더 열어도 고정 10개는 남는다.
        for d in &dirs[MAX_PINNED + 1..] {
            store.touch(&real(d.path()));
        }
        let again = RecentStore::open(data.path());
        let list = again.list();
        assert_eq!(list.iter().filter(|e| e.pinned).count(), MAX_PINNED);
        assert_eq!(list.iter().filter(|e| !e.pinned).count(), MAX_RECENT);
        let first = list
            .iter()
            .find(|e| Path::new(&e.path) == real(dirs[0].path()))
            .unwrap();
        assert_eq!(first.pinned_branches, ["기능/로그인"]);
        // 목록에서 빼면 고정도 풀린다.
        again.remove(&real(dirs[0].path()));
        assert_eq!(
            again.pin(&real(dirs[0].path()), true),
            Err(PinError::NotListed)
        );
    }

    #[test]
    fn branch_pins_are_limited_and_only_for_existing_branches() {
        let data = tempfile::tempdir().unwrap();
        let repo = tempfile::tempdir().unwrap();
        let (core, git) = core(Some(repo.path()), data.path());
        core.open_project(repo.path(), OpenHow::Picked).unwrap();
        git.refs.lock().unwrap().push(git::RefEntry {
            name: "refs/heads/main".into(),
            oid: "a".repeat(40),
            time: 0,
            summary: String::new(),
        });
        core.pin_branch(repo.path(), "main", true).unwrap();
        assert_eq!(core.recent_projects()[0].pinned_branches, ["main"]);
        // 없는 브랜치는 고정하지 않는다(풀기는 된다).
        let err = core.pin_branch(repo.path(), "없음", true).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
        core.pin_branch(repo.path(), "없음", false).unwrap();
        // 브랜치는 10개까지, 이름은 200바이트까지.
        drop(core);
        let store = RecentStore::open(data.path());
        for i in 1..10 {
            store
                .pin_branch(&real(repo.path()), &format!("b{i}"), true)
                .unwrap();
        }
        assert_eq!(
            store.pin_branch(&real(repo.path()), "b10", true),
            Err(PinError::Limit)
        );
        assert_eq!(
            store.pin_branch(&real(repo.path()), &"가".repeat(70), true),
            Err(PinError::TooLong)
        );
    }

    #[test]
    fn unopened_repos_and_unlisted_projects_are_refused() {
        let data = tempfile::tempdir().unwrap();
        let repo = tempfile::tempdir().unwrap();
        let (core, _) = core(Some(repo.path()), data.path());
        let other = tempfile::tempdir().unwrap();
        assert!(core.pin_branch(other.path(), "main", true).is_err());
        let err = core.pin_project(&real(other.path()), true).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
    }
}
