use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use git::{CliBackend, GitBackend, GitVersion, RepoPath};

use crate::access::{OpenedRepos, check_path_len};
use crate::query::Queries;
use crate::recent::RecentStore;
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
    /// 지금 액션이 실행 중인 저장소(또는 폴더). 같은 곳에서 두 작업이 겹치면 index.lock이 부딪힌다.
    busy: Mutex<HashSet<PathBuf>>,
    /// 지금 감시 중인 저장소. watch.rs가 쓴다.
    pub(crate) watcher: Mutex<Option<watcher::RepoWatcher>>,
    recent: Arc<RecentStore>,
    /// 프로젝트 열기를 거친 경로. 조회·액션·감시는 여기 있는 것만 받는다(D-037).
    pub(crate) opened: Arc<OpenedRepos>,
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
    let backend = CliBackend::new(git_path, Arc::new(platform::configure_child));
    // 앱 데이터 폴더를 못 만들면 최근 목록은 메모리에만 둔다(앱은 계속 쓸 수 있다).
    let recent = match platform::app_data_dir(APP_ID) {
        Ok(dir) => RecentStore::open(&dir),
        Err(_) => RecentStore::in_memory(),
    };
    let mut core = Core::with_backend_and_recent(Arc::new(backend), recent)?;
    core.git_build = git_build;
    Ok(core)
}

impl Core {
    /// 최근 목록을 메모리에만 두는 Core(테스트용).
    pub fn with_backend(git: Arc<dyn GitBackend>) -> Result<Self> {
        Self::with_backend_and_recent(git, RecentStore::in_memory())
    }

    pub fn with_backend_and_recent(git: Arc<dyn GitBackend>, recent: RecentStore) -> Result<Self> {
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
        Ok(Self {
            executor: Executor::new(registry),
            queries: Queries::new(git.clone(), opened.clone(), recent.clone()),
            git,
            git_version,
            git_build: platform::GitBuild::Unknown,
            busy: Mutex::new(HashSet::new()),
            watcher: Mutex::new(None),
            recent,
            opened,
        })
    }

    pub fn git_version(&self) -> GitVersion {
        self.git_version
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
        let Some(repo) = self.git.discover(&folder)? else {
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
        trust_check(&repo)?;
        self.opened.insert(&repo);
        self.recent.touch(&repo);
        Ok(ProjectOpenResult::Opened {
            repo: RepoInfo::from_path(&repo),
        })
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
        let state = self.state_of(folder)?;
        let key = state
            .repo
            .as_ref()
            .map(|r| r.as_path().to_path_buf())
            .unwrap_or_else(|| state.folder.clone());
        if !self
            .busy
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.clone())
        {
            return Err(CoreError::Busy);
        }
        let _guard = BusyGuard {
            busy: &self.busy,
            key,
        };
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
        };
        let token = preview_token.map(PreviewToken);
        let outcome = self
            .executor
            .execute(&ctx, input.action_id(), input, token.as_ref())?;
        if let crate::Outcome::RepoCreated(repo) = &outcome {
            self.opened.insert(repo.as_path());
            self.recent.touch(repo.as_path());
        }
        Ok(outcome.into())
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

/// 저장소 신뢰 확인 자리(보안 계획에서 채운다). 프로젝트 열기는 모두 여기를 지난다.
fn trust_check(_repo: &Path) -> Result<()> {
    Ok(())
}
