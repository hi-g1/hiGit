//! 최근 프로젝트 목록과 프로젝트 열기(D-032).
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
