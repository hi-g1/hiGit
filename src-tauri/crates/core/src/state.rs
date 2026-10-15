use std::path::PathBuf;

use git::RepoPath;

/// 열린 폴더와 그 폴더가 저장소인지. 저장소가 아니어도 열 수 있다(관리 시작을 보여주려고).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoState {
    /// 사용자가 고른 폴더(정규화된 절대 경로).
    pub folder: PathBuf,
    /// 폴더가 속한 저장소의 최상위. 저장소가 아니면 `None`.
    pub repo: Option<RepoPath>,
}
