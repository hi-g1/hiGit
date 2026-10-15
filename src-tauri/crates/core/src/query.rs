use std::path::Path;
use std::sync::Arc;

use git::{GitBackend, RepoPath};

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
        Ok(self.git.diff(&repo_path(repo)?, &request.into())?.into())
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
