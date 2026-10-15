use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use git::{CliBackend, GitBackend, GitVersion, RepoPath};

use crate::access::{OpenedRepos, check_path_len};
use crate::query::Queries;
use crate::recent::RecentStore;
use crate::trust::{TrustDecision, TrustState, TrustStore};
use crate::view::{
    ActionInput, ActionMetaView, ActionResult, AppStatus, OpenHow, PreviewView, ProjectOpenResult,
    RecentProject, RepoInfo,
};
use crate::{CoreError, ExecCtx, Executor, PreviewToken, Registry, RepoState, Result, actions};

/// 지원하는 가장 낮은 git 버전(D-003).
pub const MIN_GIT_VERSION: GitVersion = GitVersion::new(2, 30, 0);

/// 앱 identifier(D-009). `tauri.conf.json`의 identifier와 같아야 한다(테스트로 확인).
pub const APP_ID: &str = "dev.higit.desktop";

/// 앱이 쓰는 core의 입구. app은 이 타입만 부른다.
pub struct Core {
    pub(crate) git: Arc<dyn GitBackend>,
    git_version: GitVersion,
    /// git 설치 방식. 보안 수정 기준 판정에 쓴다(D-050).
    git_build: platform::GitBuild,
    executor: Executor,
    queries: Queries,
    /// 지금 작업 폴더를 쓰는 작업이 실행 중인 저장소. 같은 곳에서 두 작업이 겹치면 index.lock이 부딪힌다.
    busy: Mutex<HashSet<PathBuf>>,
    /// 지금 원격과 통신 중인 저장소. 원격 확인(fetch)은 이것만 잡아 버전 저장을 막지 않는다(H13).
    net_busy: Mutex<HashSet<PathBuf>>,
    /// 지금 감시 중인 저장소. watch.rs가 쓴다.
    pub(crate) watcher: Mutex<Option<watcher::RepoWatcher>>,
    recent: Arc<RecentStore>,
    /// 프로젝트 열기를 거친 경로. 조회·액션·감시는 여기 있는 것만 받는다(D-037).
    pub(crate) opened: Arc<OpenedRepos>,
    /// 저장소 신뢰 기록(D-039).
    trust: Arc<TrustStore>,
    /// 신뢰 확인을 물은 경로. 신뢰·주인 허용 명령은 core가 물은 경로만 받는다(D-037).
    asked: Mutex<Vec<(PathBuf, Ask)>>,
}

/// core가 사용자에게 물은 것.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    /// 실행되는 설정(NeedsTrust).
    Trust,
    /// 폴더 주인(NotOwned).
    Owner,
}

/// 작업이 잡는 잠금.
#[derive(Clone, Copy)]
enum Locks {
    Worktree,
    Network,
    Both,
}

impl Locks {
    fn sets<'a>(
        self,
        worktree: &'a Mutex<HashSet<PathBuf>>,
        network: &'a Mutex<HashSet<PathBuf>>,
    ) -> Vec<&'a Mutex<HashSet<PathBuf>>> {
        match self {
            Locks::Worktree => vec![worktree],
            Locks::Network => vec![network],
            Locks::Both => vec![worktree, network],
        }
    }
}

/// 실행이 끝나면(오류여도) 잠금을 푼다.
struct BusyGuard<'a> {
    busy: &'a Mutex<HashSet<PathBuf>>,
    key: PathBuf,
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.busy
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.key);
    }
}

/// 시스템 git을 찾고 버전을 확인한 뒤 Core를 만든다.
pub fn init() -> Result<Core> {
    let git_path = platform::find_git().ok_or(CoreError::GitMissing)?;
    let git_build = platform::git_build(&git_path);
    let backend = CliBackend::new(git_path, Arc::new(platform::configure_child))
        .with_process_control(git::ProcessControl {
            prepare: Arc::new(platform::prepare_killable),
            kill: Arc::new(platform::kill_tree),
        });
    // 앱 데이터 폴더를 못 만들면 최근 목록은 메모리에만 둔다(앱은 계속 쓸 수 있다).
    let data_dir = platform::app_data_dir(APP_ID).ok();
    let recent = match &data_dir {
        Some(dir) => RecentStore::open(dir),
        None => RecentStore::in_memory(),
    };
    let trust = match &data_dir {
        Some(dir) => TrustStore::open(dir),
        None => TrustStore::in_memory(),
    };
    let mut core = Core::with_stores(Arc::new(backend), recent, trust)?;
    core.git_build = git_build;
    // journal 서명 키는 앱 데이터에 둔다. 폴더를 못 쓰면 이번 실행에서만 쓰는 키(D-063).
    if let Some(dir) = &data_dir {
        core.executor
            .set_journal_key(crate::undo::JournalKey::load_or_create(dir));
    }
    Ok(core)
}

impl Core {
    /// 최근 목록을 메모리에만 두는 Core(테스트용).
    pub fn with_backend(git: Arc<dyn GitBackend>) -> Result<Self> {
        Self::with_backend_and_recent(git, RecentStore::in_memory())
    }

    pub fn with_backend_and_recent(git: Arc<dyn GitBackend>, recent: RecentStore) -> Result<Self> {
        Self::with_stores(git, recent, TrustStore::in_memory())
    }

    pub fn with_stores(
        git: Arc<dyn GitBackend>,
        recent: RecentStore,
        trust: TrustStore,
    ) -> Result<Self> {
        let git_version = git.version().map_err(|e| match e {
            git::GitError::Spawn(_) => CoreError::GitMissing,
            other => CoreError::Git(other),
        })?;
        if git_version < MIN_GIT_VERSION {
            return Err(CoreError::GitTooOld { found: git_version });
        }

        let mut registry = Registry::new();
        actions::register_all(&mut registry)?;
        let recent = Arc::new(recent);
        let opened = Arc::new(OpenedRepos::default());
        let trust = Arc::new(trust);
        Ok(Self {
            executor: Executor::new(registry),
            queries: Queries::new(git.clone(), opened.clone(), recent.clone(), trust.clone()),
            git,
            git_version,
            git_build: platform::GitBuild::Unknown,
            busy: Mutex::new(HashSet::new()),
            net_busy: Mutex::new(HashSet::new()),
            watcher: Mutex::new(None),
            recent,
            opened,
            trust,
            asked: Mutex::new(Vec::new()),
        })
    }

    fn remember_ask(&self, path: &Path, ask: Ask) {
        let mut asked = self.asked.lock().unwrap_or_else(PoisonError::into_inner);
        asked.retain(|(p, a)| !(*a == ask && platform::same_path(p, path)));
        asked.push((path.to_path_buf(), ask));
    }

    /// 물은 적이 있으면 지우고 참. 한 번 물음에 한 번만 답을 받는다.
    fn take_ask(&self, path: &Path, ask: Ask) -> bool {
        let mut asked = self.asked.lock().unwrap_or_else(PoisonError::into_inner);
        let before = asked.len();
        asked.retain(|(p, a)| !(*a == ask && platform::same_path(p, path)));
        asked.len() != before
    }

    pub fn git_version(&self) -> GitVersion {
        self.git_version
    }

    /// Git 내려받기 페이지 주소(https만). 앱이 기본 브라우저로 연다.
    pub fn git_download_url(&self) -> &'static str {
        platform::git_download_url()
    }

    /// 화면 시작에 필요한 git 정보. 보안 수정 기준은 거부 기준과 따로 판정한다(D-050).
    pub fn app_status(&self) -> AppStatus {
        let package = match &self.git_build {
            platform::GitBuild::DistroPackage { version } => Some(version.clone()),
            _ => None,
        };
        AppStatus {
            git_version: self.git_version.to_string(),
            git_security: crate::git_security::judge(self.git_version, package.is_some()).into(),
            git_package_version: package,
        }
    }

    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    /// 프로젝트 열기의 유일한 입구(D-032). 앱을 켤 때 열기, 최근 목록, 폴더 고르기,
    /// 프로젝트 메뉴가 모두 여기를 거친다.
    pub fn open_project(&self, path: &Path, how: OpenHow) -> Result<ProjectOpenResult> {
        check_path_len(path)?;
        let Ok(folder) = platform::normalize_repo_path(path) else {
            return Ok(ProjectOpenResult::Missing);
        };
        let folder_text = folder.to_string_lossy().into_owned();
        let discovered = match self.git.discover(&folder) {
            Err(e) if e.kind() == git::FailureKind::DubiousOwnership => {
                self.remember_ask(&folder, Ask::Owner);
                return Ok(ProjectOpenResult::NotOwned {
                    folder: folder_text,
                });
            }
            other => other?,
        };
        let Some(repo) = discovered else {
            // 저장소가 아닌 폴더도 열린 것으로 기록한다. 이 폴더에서 "버전 관리 시작"을 할 수 있다.
            self.opened.insert(&folder);
            return Ok(ProjectOpenResult::NotARepo {
                folder: folder_text,
            });
        };
        let repo = platform::normalize_repo_path(repo.as_path())
            .unwrap_or_else(|_| repo.as_path().to_path_buf());
        // 기록한 저장소의 .git만 지워지면 바깥 폴더의 저장소가 잡힌다. 그걸 대신 열지 않는다.
        if how == OpenHow::Recent && !platform::same_path(&repo, &folder) {
            return Ok(ProjectOpenResult::NotARepo {
                folder: folder_text,
            });
        }
        // 실행되는 설정이 있으면 묻는다. 믿기로 하기 전에는 열린 것으로 기록하지 않는다(D-039).
        let repo_path = RepoPath::new(&repo);
        match self.trust.check(self.git.as_ref(), &repo_path)? {
            TrustState::Clean | TrustState::Decided(TrustDecision::Trust, _) => {
                self.git.set_repo_overrides(&repo_path, Vec::new());
            }
            TrustState::Decided(TrustDecision::Disable, found) if found.can_disable => {
                self.git.set_repo_overrides(&repo_path, found.overrides);
            }
            // 끌 수 없는 키가 생겼으면 "끄고 열기" 결정이 있어도 다시 묻는다.
            TrustState::Decided(TrustDecision::Disable, found) | TrustState::Unknown(found) => {
                self.remember_ask(&repo, Ask::Trust);
                return Ok(ProjectOpenResult::NeedsTrust {
                    folder: repo.to_string_lossy().into_owned(),
                    keys: found.keys,
                    can_disable: found.can_disable,
                });
            }
        }
        self.opened.insert(&repo);
        self.recent.touch(&repo);
        Ok(ProjectOpenResult::Opened {
            repo: RepoInfo::from_path(&repo),
        })
    }

    /// 신뢰 확인에서 고른 결과를 기록하고 연다(D-039). "끄고 열기"면 실행되는 설정을 끈다.
    pub fn trust_project(&self, path: &Path, decision: TrustDecision) -> Result<ProjectOpenResult> {
        check_path_len(path)?;
        let folder = platform::normalize_repo_path(path).map_err(CoreError::FolderNotFound)?;
        let repo = self.git.discover(&folder)?.ok_or(CoreError::NotARepo)?;
        let repo = platform::normalize_repo_path(repo.as_path())
            .unwrap_or_else(|_| repo.as_path().to_path_buf());
        // 신뢰 확인 창을 거치지 않은 경로는 받지 않는다.
        if !self.take_ask(&repo, Ask::Trust) {
            return Err(CoreError::NotOpened);
        }
        let found = match self.trust.check(self.git.as_ref(), &RepoPath::new(&repo))? {
            TrustState::Clean => None,
            TrustState::Decided(_, found) | TrustState::Unknown(found) => Some(found),
        };
        if let Some(found) = found {
            if decision == TrustDecision::Disable && !found.can_disable {
                // 창은 그대로 떠 있으니 "믿고 열기"는 아직 받을 수 있다.
                self.remember_ask(&repo, Ask::Trust);
                return Err(CoreError::NeedsTrust);
            }
            self.trust.record(&repo, &found, decision);
        }
        self.open_project(&repo, OpenHow::Picked)
    }

    /// 주인이 다른 폴더를 사용자가 믿기로 했다. 전역 설정 `safe.directory`에 이 폴더 하나만 더하고 연다.
    /// `*`로 모두 열지 않는다(D-039).
    pub fn allow_folder_owner(&self, path: &Path) -> Result<ProjectOpenResult> {
        check_path_len(path)?;
        let folder = platform::normalize_repo_path(path).map_err(CoreError::FolderNotFound)?;
        // 주인 확인 창을 거치지 않은 경로는 받지 않는다. 전역 설정을 영구히 바꾸는 일이다.
        if !self.take_ask(&folder, Ask::Owner) {
            return Err(CoreError::NotOpened);
        }
        self.git.run(
            &RepoPath::new(&folder),
            &git::GitOp::AddSafeDirectory {
                path: platform::git_path_string(&folder),
            },
        )?;
        self.open_project(&folder, OpenHow::Picked)
    }

    pub fn recent_projects(&self) -> Vec<RecentProject> {
        self.recent
            .list()
            .into_iter()
            .map(RecentProject::from)
            .collect()
    }

    pub fn remove_recent(&self, path: &Path) {
        self.recent.remove(path);
    }

    pub fn action_metas(&self) -> Vec<ActionMetaView> {
        self.executor
            .registry()
            .metas()
            .map(ActionMetaView::from)
            .collect()
    }

    pub fn preview(&self, folder: &Path, input: &ActionInput) -> Result<PreviewView> {
        let state = self.state_of(folder)?;
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
        };
        let (_preview, token) = self.executor.preview(&ctx, input.action_id(), input)?;
        Ok(PreviewView { token: token.0 })
    }

    pub fn execute(
        &self,
        folder: &Path,
        input: &ActionInput,
        preview_token: Option<String>,
    ) -> Result<ActionResult> {
        let token = preview_token.map(PreviewToken);
        let locks = match input.action_id() {
            "fetch" => Locks::Network,
            "pull" | "push" | "sync" => Locks::Both,
            _ => Locks::Worktree,
        };
        let outcome = self.with_locks(folder, locks, |executor, ctx| {
            executor.execute(ctx, input.action_id(), input, token.as_ref())
        })?;
        if let crate::Outcome::RepoCreated(repo) = &outcome {
            self.opened.insert(repo.as_path());
            self.recent.touch(repo.as_path());
        }
        Ok(outcome.into())
    }

    /// 마지막 작업을 되돌린다(Ctrl+Z, D-042).
    pub fn undo(&self, folder: &Path) -> Result<crate::view::UndoResult> {
        self.with_repo_lock(folder, |executor, ctx| executor.undo(ctx))
            .map(Into::into)
    }

    /// 되돌린 작업을 다시 한다(Ctrl+Shift+Z, Ctrl+Y).
    pub fn redo(&self, folder: &Path) -> Result<crate::view::UndoResult> {
        self.with_repo_lock(folder, |executor, ctx| executor.redo(ctx))
            .map(Into::into)
    }

    fn with_repo_lock<T>(
        &self,
        folder: &Path,
        f: impl FnOnce(&Executor, &ExecCtx<'_>) -> Result<T>,
    ) -> Result<T> {
        self.with_locks(folder, Locks::Worktree, f)
    }

    /// 저장소 잠금을 잡고(같은 저장소에서 한 번에 하나) 실행기에 맡긴다.
    fn with_locks<T>(
        &self,
        folder: &Path,
        locks: Locks,
        f: impl FnOnce(&Executor, &ExecCtx<'_>) -> Result<T>,
    ) -> Result<T> {
        let state = self.state_of(folder)?;
        let key = state
            .repo
            .as_ref()
            .map(|r| r.as_path().to_path_buf())
            .unwrap_or_else(|| state.folder.clone());
        let mut guards = Vec::new();
        for set in locks.sets(&self.busy, &self.net_busy) {
            if !set
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(key.clone())
            {
                return Err(CoreError::Busy);
            }
            guards.push(BusyGuard {
                busy: set,
                key: key.clone(),
            });
        }
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
        };
        f(&self.executor, &ctx)
    }

    /// 폴더를 정규화하고 저장소인지 확인한다. 저장소 경로도 같은 형식으로 정규화한다.
    fn state_of(&self, path: &Path) -> Result<RepoState> {
        check_path_len(path)?;
        let folder = platform::normalize_repo_path(path).map_err(CoreError::FolderNotFound)?;
        self.opened.require(&folder)?;
        let repo = self.git.discover(&folder)?.map(|r| {
            platform::normalize_repo_path(r.as_path())
                .map(RepoPath::new)
                .unwrap_or(r)
        });
        Ok(RepoState { folder, repo })
    }
}
