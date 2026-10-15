use std::path::Path;
use std::sync::Arc;

use git::{CliBackend, GitBackend, GitVersion, RepoPath};
use serde_json::Value;

use crate::query::Queries;
use crate::view::{ActionMetaView, ActionResult, FolderInfo, PreviewView, RepoInfo};
use crate::{CoreError, ExecCtx, Executor, PreviewToken, Registry, RepoState, Result, actions};

/// 지원하는 가장 낮은 git 버전(D-003).
pub const MIN_GIT_VERSION: GitVersion = GitVersion::new(2, 30, 0);

/// 앱이 쓰는 core의 입구. app은 이 타입만 부른다.
pub struct Core {
    git: Arc<dyn GitBackend>,
    git_version: GitVersion,
    executor: Executor,
    queries: Queries,
}

/// 시스템 git을 찾고 버전을 확인한 뒤 Core를 만든다.
pub fn init() -> Result<Core> {
    let git_path = platform::find_git().ok_or(CoreError::GitMissing)?;
    let backend = CliBackend::new(git_path, Arc::new(platform::configure_child));
    Core::with_backend(Arc::new(backend))
}

impl Core {
    pub fn with_backend(git: Arc<dyn GitBackend>) -> Result<Self> {
        let git_version = git.version().map_err(|e| match e {
            git::GitError::Spawn(_) => CoreError::GitMissing,
            other => CoreError::Git(other),
        })?;
        if git_version < MIN_GIT_VERSION {
            return Err(CoreError::GitTooOld { found: git_version });
        }

        let mut registry = Registry::new();
        actions::register_all(&mut registry)?;
        Ok(Self {
            executor: Executor::new(registry),
            queries: Queries::new(git.clone()),
            git,
            git_version,
        })
    }

    pub fn git_version(&self) -> GitVersion {
        self.git_version
    }

    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    /// 폴더를 연다. 저장소가 아니어도 오류가 아니다(화면이 "관리 시작"을 보여준다).
    pub fn open_folder(&self, path: &Path) -> Result<FolderInfo> {
        let state = self.state_of(path)?;
        Ok(FolderInfo {
            folder: state.folder.to_string_lossy().into_owned(),
            repo: state.repo.map(|r| RepoInfo::from_path(r.as_path())),
        })
    }

    pub fn action_metas(&self) -> Vec<ActionMetaView> {
        self.executor
            .registry()
            .metas()
            .map(ActionMetaView::from)
            .collect()
    }

    pub fn preview(&self, folder: &Path, id: &str, input: &Value) -> Result<PreviewView> {
        let state = self.state_of(folder)?;
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
        };
        let (_preview, token) = self.executor.preview(&ctx, id, input)?;
        Ok(PreviewView { token: token.0 })
    }

    pub fn execute(
        &self,
        folder: &Path,
        id: &str,
        input: &Value,
        preview_token: Option<String>,
    ) -> Result<ActionResult> {
        let state = self.state_of(folder)?;
        let ctx = ExecCtx {
            git: self.git.as_ref(),
            state: &state,
        };
        let token = preview_token.map(PreviewToken);
        let outcome = self.executor.execute(&ctx, id, input, token.as_ref())?;
        Ok(outcome.into())
    }

    /// 폴더를 정규화하고 저장소인지 확인한다. 저장소 경로도 같은 형식으로 정규화한다.
    fn state_of(&self, path: &Path) -> Result<RepoState> {
        let folder = platform::normalize_repo_path(path).map_err(CoreError::FolderNotFound)?;
        let repo = self.git.discover(&folder)?.map(|r| {
            platform::normalize_repo_path(r.as_path())
                .map(RepoPath::new)
                .unwrap_or(r)
        });
        Ok(RepoState { folder, repo })
    }
}
