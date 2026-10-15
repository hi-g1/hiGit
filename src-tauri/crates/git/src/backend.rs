use std::path::{Path, PathBuf};

use crate::{
    Commit, DiffTarget, FileDiff, GitOp, GitVersion, LogQuery, OpOutput, RepoPath, Result, Status,
};

/// 모든 Git 작업의 입구(D-003). 목표 설계는 docs/architecture.md에 있고,
/// 여기에는 지금 반복에서 구현한 메서드만 둔다.
pub trait GitBackend: Send + Sync {
    /// 설치된 git의 버전.
    fn version(&self) -> Result<GitVersion>;

    /// `path`가 속한 저장소의 최상위 폴더. 저장소가 아니면 `None`.
    fn discover(&self, path: &Path) -> Result<Option<RepoPath>>;

    /// 마지막 저장 이후 바뀐 파일과 현재 갈래 정보.
    fn status(&self, repo: &RepoPath) -> Result<Status>;

    /// 저장 기록. 최근 것부터. 저장이 하나도 없으면 빈 목록.
    fn log(&self, repo: &RepoPath, query: &LogQuery) -> Result<Vec<Commit>>;

    /// 전역 무시 규칙 파일(`core.excludesFile`) 경로. 설정이 없으면 `None`.
    fn excludes_file(&self, repo: &RepoPath) -> Result<Option<PathBuf>>;

    /// 파일 하나를 마지막 저장과 줄 단위로 비교한다.
    fn diff(&self, repo: &RepoPath, target: &DiffTarget) -> Result<FileDiff>;

    /// 상태를 바꾸는 작업은 모두 여기로 온다. [`GitOp::Init`]에서 `repo`는 대상 폴더다.
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;
}
