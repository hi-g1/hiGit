use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

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
    /// 찾은 git 실행 파일(설정 "고급"에 보인다, D-160). 로그·오류에는 쓰지 않는다.
    git_path: String,
    executor: Executor,
    queries: Queries,
    /// 지금 작업 폴더를 쓰는 작업이 실행 중인 저장소. 같은 곳에서 두 작업이 겹치면 index.lock이 부딪힌다.
    busy: Mutex<HashSet<PathBuf>>,
    /// 지금 원격과 통신 중인 저장소. 원격 확인(fetch)은 이것만 잡아 버전 저장을 막지 않는다(H13).
    net_busy: Mutex<HashSet<PathBuf>>,
    /// 저절로 하는 원격 확인이 `net_busy`를 잡고 있는 저장소. 사람이 누른 원격 작업은 이것이
    /// 끝날 때까지 기다린다(D-068).
    bg_fetch: Mutex<HashSet<PathBuf>>,
    /// 지금 감시 중인 저장소. watch.rs가 쓴다.
    pub(crate) watcher: Mutex<Option<watcher::RepoWatcher>>,
    recent: Arc<RecentStore>,
    /// 프로젝트 열기를 거친 경로. 조회·액션·감시는 여기 있는 것만 받는다(D-037).
    pub(crate) opened: Arc<OpenedRepos>,
    /// 저장소 신뢰 기록(D-039).
    trust: Arc<TrustStore>,
    /// 로그 파일(D-078). 앱 데이터 폴더를 못 쓰면 남기지 않는다.
    logger: crate::log::Logger,
    /// 지금 도는 사람이 시킨 작업(저장소 → 무엇, 마지막 진행 알림의 경과). 진행 알림과
    /// 취소에 쓴다(D-080).
    running: Arc<Mutex<std::collections::HashMap<PathBuf, Running>>>,
    /// 신뢰 확인을 물은 경로. 신뢰·주인 허용 명령은 core가 물은 경로만 받는다(D-037).
    asked: Mutex<Vec<(PathBuf, Ask)>>,
    /// GitHub 연결(D-109).
    github: Arc<crate::github::GitHubService>,
    /// 시험용: 복제 주소로 로컬 절대 경로를 받는다.
    allow_local_clone: bool,
    /// 앱 설정(D-127).
    settings: Arc<crate::settings::SettingsStore>,
    /// 전역 이름·이메일 쓰기를 한 번에 하나로(D-125).
    identity_lock: Mutex<()>,
}

/// 도는 작업 하나.
#[derive(Clone, Copy)]
struct Running {
    /// 액션 id, 또는 `undo`·`redo`.
    action: &'static str,
    /// 마지막 진행 알림의 경과(초). 강제로 멈추기를 core도 확인한다.
    elapsed_secs: u64,
}

/// 원격 작업. 언제든 취소할 수 있고 진행 알림이 오래 없으면 "응답 없음"이다. 화면도 이 목록을 바인딩으로 받는다.
pub const REMOTE_ACTIONS: [&str; 11] = [
    "fetch",
    "pull",
    "push",
    "sync",
    "clone",
    "force-push",
    "remote-branch-delete",
    "remote-branch-rename",
    "tag-push",
    "fork",
    "remote-tag-delete",
];

/// 복제 대상 경로의 최대 글자 수(Windows 260자 한도 안에서 저장소 안 파일 자리를 남긴다).
const CLONE_TARGET_MAX_CHARS: usize = 200;
/// 로컬 쓰기는 이만큼 지나야 강제로 멈출 수 있다(D-080). 화면도 이때부터 단추를 보인다.
const STUCK_LOCAL_SECS: u64 = 60;

/// core가 사용자에게 물은 것.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    /// 실행되는 설정(NeedsTrust).
    Trust,
    /// 폴더 주인(NotOwned).
    Owner,
}

/// 진행 알림 하나. 초 단위로 줄인다.
fn progress(
    repo: &Path,
    action: &str,
    elapsed: std::time::Duration,
    idle: std::time::Duration,
) -> crate::view::OpProgress {
    let secs = |d: std::time::Duration| u32::try_from(d.as_secs()).unwrap_or(u32::MAX);
    crate::view::OpProgress {
        repo: repo.to_string_lossy().into_owned(),
        action: action.to_string(),
        elapsed_secs: secs(elapsed),
        idle_secs: secs(idle),
    }
}

/// 원격 확인이 끝나기를 기다리는 간격과, 그 시간 제한 위에 더 기다리는 여유.
const WAIT_STEP: std::time::Duration = std::time::Duration::from_millis(100);
const WAIT_SLACK: std::time::Duration = std::time::Duration::from_secs(5);

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
    let git_path_text = git_path.to_string_lossy().into_owned();
    let backend = CliBackend::new(git_path, Arc::new(platform::configure_child))
        .with_process_control(git::ProcessControl {
            prepare: Arc::new(platform::prepare_killable),
            kill: Arc::new(platform::kill_tree),
            terminate: Arc::new(platform::terminate_tree),
            git_pids: Arc::new(platform::running_git_pids),
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
    // git이 credential helper로 다시 실행할 앱 실행 파일(D-111). 없으면 helper를 붙이지 않는다.
    backend.set_helper_program(platform::current_exe_path());
    let mut core = Core::with_stores(Arc::new(backend), recent, trust)?;
    core.git_build = git_build;
    core.git_path = git_path_text;
    core.set_github(github_service(data_dir.as_deref()));
    if let Some(dir) = &data_dir {
        core.settings = Arc::new(crate::settings::SettingsStore::open(dir));
    }
    if let Some(dir) = &data_dir {
        core.logger = crate::log::Logger::open(dir);
    }
    // journal 서명 키는 앱 데이터에 둔다. 폴더를 못 쓰면 이번 실행에서만 쓰는 키(D-063).
    if let Some(dir) = &data_dir {
        core.set_key(crate::undo::JournalKey::load_or_create(dir));
    }
    Ok(core)
}

/// GitHub OAuth App의 client id(D-109). 빌드 때 넣고, 디버그 빌드만 실행 때 같은 변수도 받는다.
fn github_client_id() -> Option<String> {
    let built = option_env!("HIGIT_GITHUB_CLIENT_ID").map(str::to_owned);
    if cfg!(debug_assertions) {
        built.or_else(|| std::env::var("HIGIT_GITHUB_CLIENT_ID").ok())
    } else {
        built
    }
}

fn github_service(data_dir: Option<&Path>) -> crate::github::GitHubService {
    let config = hosting::GitHubConfig::new(github_client_id().as_deref());
    let provider = hosting::GitHub::new(Arc::new(hosting::UreqTransport::new()), config.clone());
    crate::github::GitHubService::new(
        Arc::new(provider),
        &config,
        platform::Keychain::new(crate::github::KEYCHAIN_SERVICE),
        data_dir,
    )
}

impl Core {
    /// **테스트 전용.** GitHub 연결 서비스를 바꾼다(가짜 전송·메모리 키체인).
    #[doc(hidden)]
    pub fn with_github_for_tests(mut self, github: crate::github::GitHubService) -> Self {
        self.set_github(github);
        self
    }

    fn set_github(&mut self, github: crate::github::GitHubService) {
        let github = Arc::new(github);
        self.queries.set_github(github.clone());
        self.github = github;
    }

    /// **테스트 전용.** 앱 데이터 폴더처럼 `dir`의 키 파일을 쓴다(껐다 켠 뒤에도 같은 키).
    #[doc(hidden)]
    pub fn with_key_dir_for_tests(mut self, dir: &Path) -> Self {
        self.set_key(crate::undo::JournalKey::load_or_create(dir));
        self
    }

    /// 테스트가 앱이 쓴 것과 같은 표식을 만들 때 쓴다.
    #[cfg(test)]
    pub(crate) fn key_for_tests(&self) -> &crate::undo::JournalKey {
        self.executor.journal_key()
    }

    /// journal·표식 서명 키를 정한다. 실행기와 조회에 같이 준다.
    fn set_key(&mut self, key: crate::undo::JournalKey) {
        self.executor.set_journal_key(key.clone());
        self.queries.set_key(key);
    }

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
        // 실행기(journal·표식 쓰기)와 조회(표식 읽기)가 같은 키를 쓴다.
        let key = crate::undo::JournalKey::ephemeral();
        let github = Arc::new(crate::github::GitHubService::unconfigured());
        Ok(Self {
            executor: Executor::new(registry).with_journal_key(key.clone()),
            queries: Queries::new(
                git.clone(),
                opened.clone(),
                recent.clone(),
                trust.clone(),
                key,
                github.clone(),
            ),
            git,
            git_version,
            git_build: platform::GitBuild::Unknown,
            git_path: String::new(),
            busy: Mutex::new(HashSet::new()),
            net_busy: Mutex::new(HashSet::new()),
            bg_fetch: Mutex::new(HashSet::new()),
            watcher: Mutex::new(None),
            recent,
            opened,
            trust,
            asked: Mutex::new(Vec::new()),
            logger: crate::log::Logger::disabled(),
            running: Arc::default(),
            github,
            allow_local_clone: false,
            settings: Arc::new(crate::settings::SettingsStore::in_memory()),
            identity_lock: Mutex::new(()),
        })
    }

    /// 전역(없으면 시스템) 이름·이메일(D-125). 열린 저장소가 필요 없다.
    pub fn identity(&self) -> Result<crate::view::IdentityView> {
        let id = self.git.identity()?;
        let complete = id.name.is_some() && id.email.is_some();
        Ok(crate::view::IdentityView {
            name: id.name,
            email: id.email,
            complete,
            noreply_email: self.github.noreply_email(),
        })
    }

    /// 전역 이메일이 GitHub 계정의 것인지(D-186). 결과 종류만 준다. 열린 저장소가 필요 없다.
    pub fn github_email_check(&self) -> Result<crate::view::EmailCheckView> {
        let id = self.git.identity()?;
        self.github.email_check(id.email.as_deref())
    }

    /// 덮어쓰며 올리기 확인 창의 값(D-128). 설정이 꺼져 있으면 `Disabled`로 막힌다.
    pub fn force_push_plan(&self, repo: &Path) -> Result<crate::view::ForcePushPlanView> {
        let allow = self.settings.get().allow_published_rewrite;
        self.queries.force_push_plan(repo, allow)
    }

    /// 기록 정리 카드 지도의 처음 모습(D-106). 올림 검사는 앱 설정을 따른다(D-127).
    pub fn rewrite_plan(
        &self,
        repo: &Path,
        oids: Vec<String>,
    ) -> Result<crate::view::RewritePlanView> {
        let allow = self.settings.get().allow_published_rewrite;
        self.queries.rewrite_plan(repo, oids, allow)
    }

    /// 앱 설정(D-127).
    pub fn settings(&self) -> crate::view::AppSettingsView {
        self.settings.get().into()
    }

    /// 앱 설정의 알려진 칸만 바꾼다. 파일을 못 써도 이번 실행 동안은 바뀐 값을 쓴다.
    pub fn set_settings(
        &self,
        patch: &crate::view::AppSettingsPatch,
    ) -> crate::view::AppSettingsView {
        self.settings.set(patch).into()
    }

    /// **테스트 전용.** 앱 데이터 폴더처럼 `dir`의 `settings.json`을 쓴다.
    #[doc(hidden)]
    pub fn with_settings_dir_for_tests(mut self, dir: &Path) -> Self {
        self.settings = Arc::new(crate::settings::SettingsStore::open(dir));
        self
    }

    /// **테스트 전용.** 복제 주소로 로컬 절대 경로(bare 원격)를 받는다.
    #[doc(hidden)]
    pub fn allow_local_clone_for_tests(mut self) -> Self {
        self.allow_local_clone = true;
        self
    }

    /// 설정의 GitHub 구역 상태. 키체인을 건드리지 않는다.
    pub fn github_status(&self) -> crate::view::GitHubStatusView {
        self.github.status()
    }

    /// 내 GitHub 저장소 목록 한 쪽(D-133). 저장소를 받지 않는 앱 수준 조회.
    pub fn github_repos(&self, page: u32) -> Result<crate::view::GitHubRepoListView> {
        self.github.repos(page)
    }

    /// GitHub 로그인(device flow)을 시작한다.
    pub fn github_connect_start(
        &self,
        add: Option<crate::view::ScopeAdd>,
    ) -> Result<crate::view::LoginChallengeView> {
        let result = self.github.connect_start(add);
        self.logger.action("github", "connect-start", &result);
        result
    }

    /// 승인됐는지 묻는다. 간격 안이면 네트워크 없이 기다리라고 답한다.
    pub fn github_connect_poll(&self) -> Result<crate::view::LoginPollView> {
        let result = self.github.connect_poll();
        if !matches!(result, Ok(crate::view::LoginPollView::Pending { .. })) {
            self.logger.action("github", "connect-poll", &result);
        }
        result
    }

    pub fn github_connect_cancel(&self) {
        self.github.connect_cancel();
    }

    /// 연결을 끊는다(이 컴퓨터에서만).
    pub fn github_disconnect(&self) -> Result<()> {
        let result = self.github.disconnect();
        self.logger.action("github", "disconnect", &result);
        result
    }

    /// 기본 브라우저로 열 GitHub 페이지 주소. 화면은 주소를 넘기지 않고 종류만 고른다.
    /// 검토 요청은 열린 저장소(`folder`)의 GitHub 원격에서 만든다.
    pub fn hosting_page_url(
        &self,
        folder: Option<&Path>,
        page: crate::view::HostingPage,
    ) -> Result<String> {
        use crate::view::HostingPage;
        match page {
            HostingPage::DeviceLogin => Ok(self.github.device_login_url()),
            HostingPage::AppSettings => Ok(self.github.app_settings_url()),
            HostingPage::PullRequest { number } => {
                let folder = folder.ok_or(CoreError::NotOpened)?;
                let repo = self.queries.github_repo(folder)?;
                self.github.pull_request_url(&repo, number)
            }
            HostingPage::Repository => {
                let folder = folder.ok_or(CoreError::NotOpened)?;
                let repo = self.queries.github_repo(folder)?;
                self.github.repository_url(&repo)
            }
        }
    }

    /// 오래 걸리는 작업의 진행을 받을 곳(앱이 화면 이벤트로 보낸다, D-080).
    pub fn set_progress_sink(&self, sink: Arc<dyn Fn(crate::view::OpProgress) + Send + Sync>) {
        let running = self.running.clone();
        self.git
            .set_observer(Some(Arc::new(move |path, elapsed, idle| {
                let action = {
                    let mut running = running.lock().unwrap_or_else(PoisonError::into_inner);
                    running.get_mut(path).map(|r| {
                        r.elapsed_secs = elapsed.as_secs();
                        r.action
                    })
                };
                if let Some(action) = action {
                    sink(progress(path, action, elapsed, idle));
                }
            })));
    }

    /// 이 저장소에서 도는 원격 작업을 취소하거나, 60초 넘게 도는 로컬 쓰기를 강제로 멈춘다
    /// (D-080). 그 밖에는 아무것도 하지 않는다. 확인과 취소 표시를 한 잠금 안에서 해서 끝난
    /// 작업 뒤에 표시가 남지 않게 한다.
    pub fn cancel(&self, folder: &Path) -> Result<()> {
        // 복제는 대상 폴더와 정확히 같을 때만 취소한다. 열린 폴더가 아니어도 된다.
        if let Some(key) = clone_key(folder) {
            let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            if running.get(&key).is_some_and(|r| r.action == "clone") {
                self.git.request_cancel(&RepoPath::new(&key));
                return Ok(());
            }
        }
        let state = self.state_of(folder)?;
        let repo = state.repo.ok_or(CoreError::NotARepo)?;
        let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        let allowed = running.get(repo.as_path()).is_some_and(|r| {
            REMOTE_ACTIONS.contains(&r.action) || r.elapsed_secs >= STUCK_LOCAL_SECS
        });
        if allowed {
            self.git.request_cancel(&repo);
        }
        Ok(())
    }

    /// 사람이 시킨 작업 하나를 진행 알림·취소 대상으로 두고 실행한다. 취소 표시는 이 작업
    /// 동안만 둔다(끝난 뒤 남으면 다음 작업이 바로 멈춘다).
    fn tracked<T>(
        &self,
        git: &dyn GitBackend,
        repo: &RepoPath,
        action: &'static str,
        f: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let key = repo.as_path().to_path_buf();
        {
            let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            git.clear_cancel(repo);
            running.insert(
                key.clone(),
                Running {
                    action,
                    elapsed_secs: 0,
                },
            );
        }
        let result = f();
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        git.clear_cancel(repo);
        running.remove(&key);
        result
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
            git_path: self.git_path.clone(),
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
        // 30일 지난 백업 ref를 조금씩 정리한다(D-079). 신뢰 확인 뒤라 끈 설정은 끈 채로 돈다.
        // 정리가 실패해도 열기는 성공이다.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = crate::undo::backup::prune(
            self.git.as_ref(),
            &repo_path,
            now,
            crate::undo::backup::KEEP_DAYS,
            crate::undo::backup::PRUNE_AT_ONCE,
        );
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
            key: self.executor.journal_key(),
            github: &self.github,
            settings: self.settings.get(),
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
        // 전역 설정이라 폴더 없이 돈다(D-125). 로그에는 액션 id와 결과 코드만.
        if let ActionInput::IdentitySet(_) = input {
            let _one = self
                .identity_lock
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let state = RepoState {
                folder: PathBuf::new(),
                repo: None,
            };
            let ctx = ExecCtx::new(
                self.git.as_ref(),
                &state,
                self.executor.journal_key(),
                &self.github,
            );
            let result = self.executor.execute(&ctx, "identity-set", input, None);
            self.logger.action("execute", "identity-set", &result);
            return Ok(result?.into());
        }
        if let ActionInput::Clone(clone) = input {
            let result = self.run_clone(folder, clone);
            self.logger.action("execute", "clone", &result);
            let outcome = result?;
            if let crate::Outcome::RepoCreated(repo) = &outcome {
                self.opened.insert(repo.as_path());
                self.recent.touch(repo.as_path());
            }
            return Ok(outcome.into());
        }
        let token = preview_token.map(PreviewToken);
        // 올리기는 작업 폴더를 쓰지 않아 원격 잠금만 잡는다. 가져오기·동기화는 합치므로 둘 다(D-068).
        let locks = match input.action_id() {
            "fetch"
            | "push"
            | "pr-create"
            | "pr-comment"
            | "pr-merge"
            | "pr-close"
            | "pr-reopen"
            | "remote-tag-delete"
            | "force-push"
            | "remote-branch-delete"
            | "tag-push" => Locks::Network,
            // 이름 바꾸기는 upstream을 다시 잇느라 `.git/config`도 쓴다(계획 010 R8).
            "remote-branch-rename" => Locks::Both,
            // 저장소를 만든 뒤 `origin`을 더해 `.git/config`를 쓴다(D-132). 포크도 원격 `fork`를 더한다.
            "publish" | "fork" => Locks::Both,
            // 잠금 정리는 앱의 다른 작업이 하나도 없을 때만(D-081).
            "pull" | "sync" | "clear-lock" => Locks::Both,
            _ => Locks::Worktree,
        };
        let background = matches!(input, ActionInput::Fetch(i) if i.background);
        let result = self.with_locks_by(folder, locks, background, |executor, ctx| {
            let run = || executor.execute(ctx, input.action_id(), input, token.as_ref());
            // 원격 확인은 화면에 진행을 보이지 않고 취소도 받지 않는다.
            match ctx.state.repo.as_ref() {
                Some(repo) if !background => self.tracked(ctx.git, repo, input.action_id(), run),
                _ => run(),
            }
        });
        self.logger.action("execute", input.action_id(), &result);
        let outcome = result?;
        if let crate::Outcome::RepoCreated(repo) = &outcome {
            self.opened.insert(repo.as_path());
            self.recent.touch(repo.as_path());
        }
        Ok(outcome.into())
    }

    /// 마지막 작업을 되돌린다(Ctrl+Z, D-042).
    pub fn undo(&self, folder: &Path) -> Result<crate::view::UndoResult> {
        let result = self.with_repo_lock(folder, |executor, ctx| match ctx.state.repo.as_ref() {
            // 올린 버전을 되돌리는 새 버전은 서명·hook으로 오래 걸릴 수 있다(D-073, D-080).
            Some(repo) => self.tracked(ctx.git, repo, "undo", || executor.undo(ctx)),
            None => executor.undo(ctx),
        });
        self.logger.action("undo", "last", &result);
        result.map(Into::into)
    }

    /// 되돌린 작업을 다시 한다(Ctrl+Shift+Z, Ctrl+Y).
    pub fn redo(&self, folder: &Path) -> Result<crate::view::UndoResult> {
        let result = self.with_repo_lock(folder, |executor, ctx| match ctx.state.repo.as_ref() {
            Some(repo) => self.tracked(ctx.git, repo, "redo", || executor.redo(ctx)),
            None => executor.redo(ctx),
        });
        self.logger.action("redo", "last", &result);
        result.map(Into::into)
    }

    /// 작업 내역(D-085). 조회라 잠금을 잡지 않는다.
    pub fn recovery(&self, folder: &Path) -> Result<crate::view::RecoveryView> {
        let state = self.state_of(folder)?;
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
            key: self.executor.journal_key(),
            github: &self.github,
            settings: self.settings.get(),
        };
        Ok(crate::view::RecoveryView {
            items: self.executor.recovery(&ctx)?,
        })
    }

    /// 끝나지 않은 작업(D-082): 오래된 잠금 파일, stash 목록에 남은 맡긴 항목, 다른 도구의 작업.
    /// 조회라 잠금을 잡지 않는다. 앱의 작업이 이 저장소에서 돌면 잠금과 맡긴 항목은 비워 둔다.
    pub fn pending_work(&self, folder: &Path) -> Result<crate::view::PendingWorkView> {
        let state = self.state_of(folder)?;
        let repo = state.repo.ok_or(CoreError::NotARepo)?;
        let git_dir = self.git.git_dir(&repo)?;
        // 앱의 작업이 돌면 멈춘 작업이 앱 것인지 아직 모른다(표식과 git 상태가 바뀌는 중).
        if self.repo_busy(repo.as_path()) {
            return Ok(crate::view::PendingWorkView::default());
        }
        // 멈춘 작업이 없는데 남은 표식은 여기서만 지운다(조회는 지우지 않는다, D-102).
        crate::operation::tidy(self.git.as_ref(), &repo)?;
        let other_operation = crate::conflict::foreign_operation(
            self.git.as_ref(),
            &repo,
            self.executor.journal_key(),
        )?;
        let common_dir = self.git.common_dir(&repo)?;
        let stale_locks = crate::locks::stale(&git_dir, &common_dir, SystemTime::now(), || {
            self.git.other_git_running()
        })
        .into_iter()
        .map(|l| l.name)
        .collect();
        let leftover = crate::parked::leftovers(self.git.as_ref(), &repo)?.len();
        Ok(crate::view::PendingWorkView {
            stale_locks,
            leftover_parked: u32::try_from(leftover).unwrap_or(u32::MAX),
            other_operation,
        })
    }

    /// 충돌 해결 화면의 상태(D-069). 앱의 작업이 이 저장소에서 돌면 맥락이 바뀌는 중이라 `Busy`로
    /// 답한다(화면은 받은 상태를 그대로 둔다, 계획 008 E9).
    pub fn conflict_state(&self, folder: &Path) -> Result<crate::view::ConflictStateView> {
        let state = self.state_of(folder)?;
        if let Some(repo) = &state.repo
            && self.repo_busy(repo.as_path())
        {
            return Err(CoreError::Busy);
        }
        self.queries.conflict_state(folder)
    }

    /// 앱의 작업이 이 저장소의 작업 폴더나 원격을 쓰는 중인지.
    pub(crate) fn repo_busy(&self, repo: &Path) -> bool {
        [&self.busy, &self.net_busy].iter().any(|set| {
            set.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .contains(repo)
        })
    }

    /// 로그 폴더(D-078). 로그를 남기지 않으면 `None`.
    pub fn log_dir(&self) -> Option<&Path> {
        self.logger.dir()
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
        self.with_locks_by(folder, locks, false, f)
    }

    /// `background`는 저절로 하는 원격 확인. 사람이 누른 원격 작업은 그것이 잡은 원격 잠금을
    /// 끝날 때까지(최대 원격 확인 시간 제한) 기다리고, 그 밖의 겹침은 바로 `Busy`로 거절한다.
    fn with_locks_by<T>(
        &self,
        folder: &Path,
        locks: Locks,
        background: bool,
        f: impl FnOnce(&Executor, &ExecCtx<'_>) -> Result<T>,
    ) -> Result<T> {
        let state = self.state_of(folder)?;
        let key = state
            .repo
            .as_ref()
            .map(|r| r.as_path().to_path_buf())
            .unwrap_or_else(|| state.folder.clone());
        let deadline = std::time::Instant::now() + git::BACKGROUND_TIMEOUT + WAIT_SLACK;
        let mut guards = Vec::new();
        for set in locks.sets(&self.busy, &self.net_busy) {
            let network = std::ptr::eq(set, &self.net_busy);
            loop {
                if set
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(key.clone())
                {
                    break;
                }
                let held_by_background = self
                    .bg_fetch
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .contains(&key);
                let wait = network
                    && !background
                    && held_by_background
                    && std::time::Instant::now() < deadline;
                if !wait {
                    return Err(CoreError::Busy);
                }
                std::thread::sleep(WAIT_STEP);
            }
            guards.push(BusyGuard {
                busy: set,
                key: key.clone(),
            });
            if network && background {
                self.bg_fetch
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(key.clone());
                guards.push(BusyGuard {
                    busy: &self.bg_fetch,
                    key: key.clone(),
                });
            }
        }
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
            key: self.executor.journal_key(),
            github: &self.github,
            settings: self.settings.get(),
        };
        f(&self.executor, &ctx)
    }

    /// 복제(D-115). 열린 저장소가 있을 수 없는 작업이라 D-037의 "열린 경로만" 대신 대상 경로를
    /// 검사한다: 부모는 있는 폴더(링크 아님)이고 어떤 저장소의 `.git` 안이 아니며, 대상은 없거나 빈
    /// 폴더. 잠금·진행·취소의 키는 대상 경로다. 실패·취소 때 이번에 만든 대상만 지운다.
    fn run_clone(&self, parent: &Path, input: &crate::view::CloneInput) -> Result<crate::Outcome> {
        let url = if self.allow_local_clone && Path::new(input.url.trim()).is_absolute() {
            input.url.trim().to_owned()
        } else {
            crate::actions::names::validate_clone_url(&input.url)?
        };
        crate::actions::names::validate_clone_name(&input.name)?;
        check_path_len(parent)?;
        let meta = std::fs::symlink_metadata(parent).map_err(CoreError::FolderNotFound)?;
        if !meta.is_dir() {
            return Err(CoreError::FolderNotFound(std::io::Error::other(
                "not a folder",
            )));
        }
        let parent = platform::normalize_repo_path(parent).map_err(CoreError::FolderNotFound)?;
        if parent
            .components()
            .any(|c| c.as_os_str().eq_ignore_ascii_case(".git"))
        {
            return Err(CoreError::InvalidInput(
                "저장소의 .git 폴더 안에는 복제할 수 없어요",
            ));
        }
        let target = parent.join(&input.name);
        if target.to_string_lossy().chars().count() > CLONE_TARGET_MAX_CHARS {
            return Err(CoreError::InvalidInput("경로가 너무 길어요"));
        }
        let existed = match std::fs::symlink_metadata(&target) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(CoreError::FolderNotFound(e)),
            Ok(m) if m.is_dir() => {
                let empty = std::fs::read_dir(&target)
                    .map_err(CoreError::FolderNotFound)?
                    .next()
                    .is_none();
                if !empty {
                    return Err(CoreError::CloneTargetExists);
                }
                true
            }
            Ok(_) => return Err(CoreError::CloneTargetExists),
        };
        let distro = matches!(self.git_build, platform::GitBuild::DistroPackage { .. });
        if crate::git_security::judge(self.git_version, distro)
            == crate::git_security::SecurityJudgement::Outdated
            && !input.accept_old_git
        {
            return Err(CoreError::GitOutdated);
        }
        let _guards = self.lock_target(&target)?;
        let state = RepoState {
            folder: parent.clone(),
            repo: None,
        };
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
            key: self.executor.journal_key(),
            github: &self.github,
            settings: self.settings.get(),
        };
        let input = ActionInput::Clone(crate::view::CloneInput {
            url,
            ..input.clone()
        });
        let result = self.tracked(self.git.as_ref(), &RepoPath::new(&target), "clone", || {
            self.executor.execute(&ctx, "clone", &input, None)
        });
        if result.is_err() && !existed {
            remove_created_target(&parent, &target);
        }
        result
    }

    /// 대상 경로로 작업 폴더·원격 잠금을 함께 잡는다. 같은 대상에 다른 복제가 돌면 `Busy`.
    fn lock_target(&self, target: &Path) -> Result<Vec<BusyGuard<'_>>> {
        let mut guards = Vec::new();
        for set in [&self.busy, &self.net_busy] {
            if !set
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(target.to_path_buf())
            {
                return Err(CoreError::Busy);
            }
            guards.push(BusyGuard {
                busy: set,
                key: target.to_path_buf(),
            });
        }
        Ok(guards)
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

/// 취소 요청의 폴더를 복제 대상 키(정규화한 부모 + 이름)로. 대상이 아직 없어도 된다.
fn clone_key(folder: &Path) -> Option<PathBuf> {
    let name = folder.file_name()?;
    let parent = platform::normalize_repo_path(folder.parent()?).ok()?;
    Some(parent.join(name))
}

/// 이번 복제가 만든 대상 폴더만 지운다(링크가 아니고 바로 그 부모 아래일 때).
/// 확인과 복제 사이에 다른 프로그램이 같은 이름의 폴더를 만들었을 수 있으므로, git이 만든
/// 모양(`.git`이 있거나 비어 있음)일 때만 지운다.
fn remove_created_target(parent: &Path, target: &Path) {
    let Ok(meta) = std::fs::symlink_metadata(target) else {
        return;
    };
    if !meta.is_dir() || meta.file_type().is_symlink() || target.parent() != Some(parent) {
        return;
    }
    if std::fs::symlink_metadata(target.join(".git")).is_ok() {
        let _ = std::fs::remove_dir_all(target);
    } else {
        // 비어 있을 때만 지워진다.
        let _ = std::fs::remove_dir(target);
    }
}

#[cfg(test)]
mod clone_cleanup_tests {
    use super::remove_created_target;

    #[test]
    fn removes_only_git_made_folders() {
        let parent = tempfile::tempdir().unwrap();
        let p = parent.path();
        let made = p.join("made");
        std::fs::create_dir_all(made.join(".git").join("objects")).unwrap();
        std::fs::write(made.join("a.txt"), "x").unwrap();
        remove_created_target(p, &made);
        assert!(!made.exists());

        let empty = p.join("empty");
        std::fs::create_dir(&empty).unwrap();
        remove_created_target(p, &empty);
        assert!(!empty.exists());

        // 다른 프로그램이 그사이 만든 폴더(.git 없음, 파일 있음)는 남긴다.
        let other = p.join("other");
        std::fs::create_dir(&other).unwrap();
        std::fs::write(other.join("mine.txt"), "내 것").unwrap();
        remove_created_target(p, &other);
        assert!(other.join("mine.txt").is_file());

        // 다른 부모 아래 대상은 건드리지 않는다.
        let elsewhere = tempfile::tempdir().unwrap();
        let foreign = elsewhere.path().join("x");
        std::fs::create_dir_all(foreign.join(".git")).unwrap();
        remove_created_target(p, &foreign);
        assert!(foreign.exists());
    }
}

#[cfg(test)]
mod lock_tests {
    use super::*;

    fn opened_core() -> (tempfile::TempDir, Core, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let ok = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success();
        assert!(ok);
        let core = Core::with_backend(Arc::new(CliBackend::new("git", Arc::new(|_| {})))).unwrap();
        core.open_project(dir.path(), OpenHow::Picked).unwrap();
        let key = platform::normalize_repo_path(dir.path()).unwrap();
        (dir, core, key)
    }

    #[test]
    fn a_user_network_action_waits_for_the_background_check() {
        let (dir, core, key) = opened_core();
        core.net_busy.lock().unwrap().insert(key.clone());
        core.bg_fetch.lock().unwrap().insert(key.clone());
        std::thread::scope(|s| {
            s.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(300));
                core.bg_fetch.lock().unwrap().remove(&key);
                core.net_busy.lock().unwrap().remove(&key);
            });
            let r = core.with_locks_by(dir.path(), Locks::Both, false, |_, _| Ok(()));
            assert!(r.is_ok(), "{r:?}");
        });
        // 다 끝나면 잠금이 모두 풀린다.
        assert!(core.busy.lock().unwrap().is_empty());
        assert!(core.net_busy.lock().unwrap().is_empty());
    }

    #[test]
    fn other_overlaps_are_refused_at_once() {
        let (dir, core, key) = opened_core();
        // 사람이 누른 원격 작업이 잡고 있으면 기다리지 않는다.
        core.net_busy.lock().unwrap().insert(key.clone());
        let r = core.with_locks_by(dir.path(), Locks::Network, false, |_, _| Ok(()));
        assert!(matches!(r, Err(CoreError::Busy)), "{r:?}");
        // 원격 확인끼리도 기다리지 않는다.
        core.bg_fetch.lock().unwrap().insert(key.clone());
        let r = core.with_locks_by(dir.path(), Locks::Network, true, |_, _| Ok(()));
        assert!(matches!(r, Err(CoreError::Busy)), "{r:?}");
        // 원격 확인 중에도 버전 저장(작업 폴더 잠금)은 된다.
        let r = core.with_locks_by(dir.path(), Locks::Worktree, false, |_, _| Ok(()));
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn a_background_check_marks_itself_while_running() {
        let (dir, core, key) = opened_core();
        core.with_locks_by(dir.path(), Locks::Network, true, |_, _| {
            assert!(core.bg_fetch.lock().unwrap().contains(&key));
            Ok(())
        })
        .unwrap();
        assert!(core.bg_fetch.lock().unwrap().is_empty());
    }
}
