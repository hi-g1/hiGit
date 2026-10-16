//! 올리는 곳·작업 폴더를 바꾸는 설정의 신뢰 확인을 실제 git으로 시험한다(D-212).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use higit_core::recent::RecentStore;
use higit_core::trust::{TrustDecision, TrustStore};
use higit_core::view::{
    ActionInput, CommitInput, ErrorCode, ErrorView, FetchInput, OpenHow, ProjectOpenResult,
    SyncInput, TagPushInput,
};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false"])
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

fn core_with(data: &Path, home: Option<PathBuf>) -> Core {
    let backend = git::CliBackend::new(
        "git",
        Arc::new(move |cmd: &mut std::process::Command| {
            if let Some(home) = &home {
                cmd.env("HOME", home)
                    .env("XDG_CONFIG_HOME", home.join(".config"))
                    .env("GIT_CONFIG_NOSYSTEM", "1");
            }
        }),
    )
    .allow_local_remotes_for_tests();
    Core::with_stores(
        Arc::new(backend) as Arc<dyn GitBackend>,
        RecentStore::in_memory(),
        TrustStore::open(data),
    )
    .unwrap()
}

fn core(data: &Path) -> Core {
    core_with(data, None)
}

fn code(e: &higit_core::CoreError) -> ErrorCode {
    ErrorView::from(e).code
}

fn init(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
        ("core.autocrlf", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
}

/// 버전 하나와 bare 원격 `origin`(main을 올려 둠)이 있는 저장소.
struct Fixture {
    repo: tempfile::TempDir,
    origin: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let repo = tempfile::tempdir().unwrap();
        let origin = tempfile::tempdir().unwrap();
        init(repo.path());
        git(origin.path(), &["init", "-q", "--bare", "-b", "main"]);
        std::fs::write(repo.path().join("a.txt"), "a\n").unwrap();
        git(repo.path(), &["add", "-A"]);
        git(repo.path(), &["commit", "-q", "-m", "처음"]);
        let url = origin.path().to_string_lossy().into_owned();
        git(repo.path(), &["remote", "add", "origin", &url]);
        git(repo.path(), &["push", "-q", "-u", "origin", "main"]);
        Self { repo, origin }
    }

    fn dir(&self) -> PathBuf {
        platform::normalize_repo_path(self.repo.path()).unwrap()
    }
}

fn refs(bare: &Path) -> String {
    git(bare, &["for-each-ref", "--format=%(refname) %(objectname)"])
}

#[test]
fn each_redirect_key_asks() {
    let cases = [
        ("remote.origin.pushurl", "https://evil.example/x.git"),
        ("remote.origin.proxy", "http://proxy.example:8080"),
        ("url.https://evil.example/.insteadOf", "https://github.com/"),
        (
            "url.https://evil.example/.pushInsteadOf",
            "https://github.com/",
        ),
        ("http.sslVerify", "false"),
        (
            "http.https://github.com/.extraHeader",
            "Authorization: Bearer secret",
        ),
    ];
    for (key, value) in cases {
        let data = tempfile::tempdir().unwrap();
        let repo = tempfile::tempdir().unwrap();
        init(repo.path());
        git(repo.path(), &["config", key, value]);
        let r = core(data.path())
            .open_project(repo.path(), OpenHow::Picked)
            .unwrap();
        let ProjectOpenResult::NeedsTrust {
            keys,
            redirects,
            can_disable,
            ..
        } = r
        else {
            panic!("{key}: {r:?}");
        };
        assert!(keys.is_empty(), "{key}: {keys:?}");
        assert!(can_disable);
        assert_eq!(redirects.len(), 1, "{key}: {redirects:?}");
        // 값 속 비밀은 화면에 가지 않는다.
        let shown = format!("{redirects:?}");
        assert!(
            !shown.contains("secret") && !shown.contains("Bearer"),
            "{shown}"
        );
    }
}

#[test]
fn disable_blocks_remote_work_until_trusted() {
    let fx = Fixture::new();
    let dir = fx.dir();
    let evil = tempfile::tempdir().unwrap();
    git(evil.path(), &["init", "-q", "--bare", "-b", "main"]);
    let evil_url = evil.path().to_string_lossy().into_owned();
    git(&dir, &["config", "remote.origin.pushurl", &evil_url]);
    git(&dir, &["tag", "v1"]);
    let origin_before = refs(fx.origin.path());

    let data = tempfile::tempdir().unwrap();
    let core = core(data.path());
    let r = core.open_project(&dir, OpenHow::Picked).unwrap();
    let ProjectOpenResult::NeedsTrust { redirects, .. } = r else {
        panic!("{r:?}");
    };
    assert_eq!(redirects[0].key, "remote.origin.pushurl");
    let r = core.trust_project(&dir, TrustDecision::Disable).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");

    // 로컬 작업은 그대로 된다.
    std::fs::write(dir.join("b.txt"), "b\n").unwrap();
    core.execute(
        &dir,
        &ActionInput::Commit(CommitInput {
            message: "둘째".into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();

    let blocked = [
        ActionInput::Fetch(FetchInput { background: false }),
        ActionInput::Fetch(FetchInput { background: true }),
        ActionInput::Push,
        ActionInput::Sync(SyncInput {
            locale: "ko".into(),
            merge_anyway: false,
        }),
        ActionInput::TagPush(TagPushInput { name: "v1".into() }),
    ];
    for input in &blocked {
        let err = core.execute(&dir, input, None).unwrap_err();
        assert_eq!(
            code(&err),
            ErrorCode::RemoteSettingsOff,
            "{input:?}: {err:?}"
        );
    }
    assert_eq!(
        refs(evil.path()),
        "",
        "끄고 열면 올리는 곳에 아무것도 가지 않는다"
    );
    assert_eq!(refs(fx.origin.path()), origin_before);

    // 다음 실행에서도 같은 결정을 따른다.
    let again = core_with(data.path(), None);
    let r = again.open_project(&dir, OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
    let err = again.execute(&dir, &ActionInput::Push, None).unwrap_err();
    assert_eq!(code(&err), ErrorCode::RemoteSettingsOff);

    // "믿고 다시 열기": 다시 묻고, 믿으면 git이 설정대로 동작한다.
    let r = again.reask_trust(&dir).unwrap();
    assert!(matches!(r, ProjectOpenResult::NeedsTrust { .. }), "{r:?}");
    let r = again.trust_project(&dir, TrustDecision::Trust).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
    again.execute(&dir, &ActionInput::Push, None).unwrap();
    assert!(refs(evil.path()).contains("refs/heads/main"));
}

#[test]
fn reask_needs_an_opened_repo_and_keeps_the_decision_until_chosen() {
    let fx = Fixture::new();
    let dir = fx.dir();
    git(&dir, &["config", "http.sslVerify", "false"]);
    let data = tempfile::tempdir().unwrap();
    let core = core(data.path());
    let err = core.reask_trust(&dir).unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotOpened);

    core.open_project(&dir, OpenHow::Picked).unwrap();
    core.trust_project(&dir, TrustDecision::Disable).unwrap();
    // 다시 묻기만 하고 고르지 않으면 원격 작업은 계속 막힌다.
    core.reask_trust(&dir).unwrap();
    let err = core
        .execute(
            &dir,
            &ActionInput::Fetch(FetchInput { background: false }),
            None,
        )
        .unwrap_err();
    assert_eq!(code(&err), ErrorCode::RemoteSettingsOff);
}

#[test]
fn global_redirect_settings_do_not_ask() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join(".gitconfig"),
        "[url \"https://mirror.example/\"]\n\tinsteadOf = https://github.com/\n[http]\n\tsslVerify = false\n",
    )
    .unwrap();
    let fx = Fixture::new();
    let data = tempfile::tempdir().unwrap();
    let core = core_with(data.path(), Some(home.path().to_path_buf()));
    let r = core.open_project(&fx.dir(), OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");
}

#[test]
fn a_changed_redirect_asks_again() {
    let fx = Fixture::new();
    let dir = fx.dir();
    git(
        &dir,
        &["config", "remote.origin.pushurl", "https://a.example/x.git"],
    );
    let data = tempfile::tempdir().unwrap();
    let core = core(data.path());
    core.open_project(&dir, OpenHow::Picked).unwrap();
    core.trust_project(&dir, TrustDecision::Trust).unwrap();
    assert!(matches!(
        core.open_project(&dir, OpenHow::Picked).unwrap(),
        ProjectOpenResult::Opened { .. }
    ));
    git(
        &dir,
        &["config", "remote.origin.pushurl", "https://b.example/x.git"],
    );
    let r = core.open_project(&dir, OpenHow::Picked).unwrap();
    let ProjectOpenResult::NeedsTrust { redirects, .. } = r else {
        panic!("{r:?}");
    };
    assert_eq!(redirects[0].target.as_deref(), Some("b.example"));
}

/// `core.worktree`가 다른 폴더를 가리키는 저장소: `.git`의 부모를 열고 다른 폴더는 건드리지 않는다.
fn assert_work_tree_pinned(repo: &Path, other: &Path) {
    let data = tempfile::tempdir().unwrap();
    let core = core(data.path());
    let r = core.open_project(repo, OpenHow::Picked).unwrap();
    let ProjectOpenResult::Opened { repo: info } = r else {
        panic!("{r:?}");
    };
    let repo = platform::normalize_repo_path(repo).unwrap();
    assert_eq!(Path::new(&info.path), repo);
    let files: Vec<String> = core
        .queries()
        .status(&repo)
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.path)
        .collect();
    assert_eq!(files, ["mine.txt"], "status는 저장소 폴더를 본다");
    core.execute(
        &repo,
        &ActionInput::Commit(CommitInput {
            message: "저장".into(),
            stage_all: true,
        }),
        None,
    )
    .unwrap();
    assert_eq!(git(&repo, &["ls-tree", "--name-only", "HEAD"]), "mine.txt");
    assert!(other.join("theirs.txt").exists());
    assert!(!other.join("mine.txt").exists());
}

#[test]
fn core_worktree_elsewhere_opens_the_git_folder_parent() {
    let repo = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    init(repo.path());
    std::fs::write(repo.path().join("mine.txt"), "m").unwrap();
    std::fs::write(other.path().join("theirs.txt"), "t").unwrap();
    let target = other.path().to_string_lossy().into_owned();
    git(repo.path(), &["config", "core.worktree", &target]);
    assert_work_tree_pinned(repo.path(), other.path());
}

#[test]
fn worktree_config_core_worktree_is_pinned_too() {
    let repo = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    init(repo.path());
    std::fs::write(repo.path().join("mine.txt"), "m").unwrap();
    std::fs::write(other.path().join("theirs.txt"), "t").unwrap();
    git(
        repo.path(),
        &["config", "extensions.worktreeConfig", "true"],
    );
    let target = other.path().to_string_lossy().into_owned();
    git(
        repo.path(),
        &["config", "--worktree", "core.worktree", &target],
    );
    assert_work_tree_pinned(repo.path(), other.path());
}

#[test]
fn linked_worktree_and_separate_git_dir_open_as_they_are() {
    let fx = Fixture::new();
    let parent = tempfile::tempdir().unwrap();
    let linked = parent.path().join("linked");
    git(
        &fx.dir(),
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "side",
            &linked.to_string_lossy(),
        ],
    );
    let data = tempfile::tempdir().unwrap();
    let core = core(data.path());
    let r = core.open_project(&linked, OpenHow::Picked).unwrap();
    let ProjectOpenResult::Opened { repo } = r else {
        panic!("{r:?}");
    };
    assert_eq!(
        Path::new(&repo.path),
        platform::normalize_repo_path(&linked).unwrap()
    );
    let status = core.queries().status(&linked).unwrap();
    assert_eq!(status.branch.name.as_deref(), Some("side"));

    // `.git` 파일이 다른 이름의 git 폴더를 가리키는 저장소도 그대로 열린다.
    let work = parent.path().join("work");
    let store = parent.path().join("store.git");
    std::fs::create_dir(&work).unwrap();
    git(
        &work,
        &["init", "-q", "--separate-git-dir", &store.to_string_lossy()],
    );
    let r = core.open_project(&work, OpenHow::Picked).unwrap();
    assert!(matches!(r, ProjectOpenResult::Opened { .. }), "{r:?}");

    // 그런 git 폴더가 다른 작업 폴더를 가리키면 열지 않는다(부모를 믿을 수 없다).
    let other = tempfile::tempdir().unwrap();
    let target = other.path().to_string_lossy().into_owned();
    git(&work, &["config", "core.worktree", &target]);
    let err = core.open_project(&work, OpenHow::Picked).unwrap_err();
    assert_eq!(code(&err), ErrorCode::NotARepo);
}
