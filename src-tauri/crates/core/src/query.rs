use std::path::Path;
use std::sync::Arc;

use git::{GitBackend, RepoPath};

use crate::Result;
use crate::view::{CommitRow, LogPage, LogQuery, StatusView};

/// 조회(D-010). 미리보기·위험도·백업이 없으므로 실행기와 레지스트리를 거치지 않는다.
pub struct Queries {
    git: Arc<dyn GitBackend>,
}

impl Queries {
    pub(crate) fn new(git: Arc<dyn GitBackend>) -> Self {
        Self { git }
    }

    pub fn status(&self, repo: &Path) -> Result<StatusView> {
        Ok(self.git.status(&RepoPath::new(repo))?.into())
    }

    pub fn log(&self, repo: &Path, query: LogQuery) -> Result<LogPage> {
        // 하나 더 받아서 다음 쪽이 있는지 안다.
        let mut commits = self.git.log(
            &RepoPath::new(repo),
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
