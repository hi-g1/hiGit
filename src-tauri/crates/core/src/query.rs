use std::path::Path;
use std::sync::Arc;

use git::{GitBackend, RepoPath};

use crate::actions::paths;
use crate::view::{CommitRow, DiffRequest, FileDiffView, LogPage, LogQuery, StatusView};
use crate::{CoreError, Result};

/// 조회(D-010). 미리보기·위험도·백업이 없으므로 실행기와 레지스트리를 거치지 않는다.
pub struct Queries {
    git: Arc<dyn GitBackend>,
}

impl Queries {
    pub(crate) fn new(git: Arc<dyn GitBackend>) -> Self {
        Self { git }
    }

    pub fn status(&self, repo: &Path) -> Result<StatusView> {
        Ok(self.git.status(&repo_path(repo)?)?.into())
    }

    /// 파일 하나를 마지막 저장과 비교한다(넣은 것과 안 넣은 것을 합쳐서).
    pub fn diff(&self, repo: &Path, request: &DiffRequest) -> Result<FileDiffView> {
        // 새 파일 비교(--no-index)는 git이 경로를 저장소로 묶지 않으므로 여기서 막는다.
        let root = repo_path(repo)?;
        for path in std::iter::once(&request.path).chain(request.orig_path.as_ref()) {
            paths::validate_one(path)?;
            stays_inside(root.as_path(), path)?;
        }
        Ok(self.git.diff(&root, &request.into())?.into())
    }

    pub fn log(&self, repo: &Path, query: LogQuery) -> Result<LogPage> {
        // 하나 더 받아서 다음 쪽이 있는지 안다.
        let mut commits = self.git.log(
            &repo_path(repo)?,
            &git::LogQuery {
                skip: query.skip,
                limit: query.limit.saturating_add(1),
                rev: None,
            },
        )?;
        let has_more = commits.len() > query.limit as usize;
        commits.truncate(query.limit as usize);
        Ok(LogPage {
            commits: commits.into_iter().map(CommitRow::from).collect(),
            has_more,
        })
    }
}

/// 화면이 보낸 경로를 폴더 열기 때와 같은 형식으로 정규화한다.
fn repo_path(path: &Path) -> Result<RepoPath> {
    platform::normalize_repo_path(path)
        .map(RepoPath::new)
        .map_err(CoreError::FolderNotFound)
}

/// 경로 글자는 저장소 안이어도 심볼릭 링크가 밖을 가리킬 수 있다. 실제 위치를 풀어 확인한다.
/// 없는 파일(지운 파일)은 그대로 둔다. 대상이 없는 링크와 밖을 가리키는 링크는 막는다.
fn stays_inside(root: &Path, rel: &str) -> Result<()> {
    let full = root.join(rel);
    let Ok(meta) = std::fs::symlink_metadata(&full) else {
        return Ok(());
    };
    let outside = match platform::normalize_repo_path(&full) {
        Ok(real) => !real.starts_with(root),
        Err(_) => meta.file_type().is_symlink(),
    };
    if outside {
        return Err(CoreError::InvalidInput("저장소 밖을 가리키는 파일이에요"));
    }
    Ok(())
}
