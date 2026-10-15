use std::path::{Path, PathBuf};

use crate::{
    Branch, Commit, DiffTarget, FileDiff, GitOp, GitVersion, LogQuery, Oid, OpOutput, RefEntry,
    RefExpect, Remote, RepoPath, Result, Status,
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

    /// 로컬·원격 브랜치 목록. 원격의 `HEAD` 별칭은 뺀다. 저장이 없으면 빈 목록.
    fn branches(&self, repo: &RepoPath) -> Result<Vec<Branch>>;

    /// 원격 목록(이름과 주소).
    fn remotes(&self, repo: &RepoPath) -> Result<Vec<Remote>>;

    /// `prefix`(예: `refs/higit/parked/`) 아래 ref 목록.
    fn list_refs(&self, repo: &RepoPath, prefix: &str) -> Result<Vec<RefEntry>>;

    /// 원격의 기본 브랜치 이름(`refs/remotes/<원격>/HEAD`가 가리키는 것). 없으면 `None`.
    fn remote_head(&self, repo: &RepoPath, remote: &str) -> Result<Option<String>>;

    /// 새 브랜치 이름으로 쓸 수 있는지. git 규칙에 더해 `-`로 시작하는 이름과 `HEAD`를 막는다.
    fn check_branch_name(&self, name: &str) -> Result<bool>;

    /// ref나 리비전(`HEAD`, `refs/heads/main`)이 가리키는 버전. 없으면 `None`.
    fn resolve_ref(&self, repo: &RepoPath, name: &str) -> Result<Option<Oid>>;

    /// core가 만든 ref(`refs/…`)를 `new`로 쓴다. `expect`가 맞지 않으면(그사이 바뀜) 실패한다.
    fn update_ref(&self, repo: &RepoPath, name: &str, new: &Oid, expect: &RefExpect) -> Result<()>;

    /// core가 만든 ref를 지운다. `expect`가 맞지 않으면 실패한다.
    fn delete_ref(&self, repo: &RepoPath, name: &str, expect: &RefExpect) -> Result<()>;

    /// 이 작업 트리의 git 폴더(절대 경로). worktree면 `.git/worktrees/<이름>`.
    fn git_dir(&self, repo: &RepoPath) -> Result<PathBuf>;

    /// `ancestor`가 `descendant`의 조상이거나 같은 버전인지.
    fn is_ancestor(&self, repo: &RepoPath, ancestor: &str, descendant: &str) -> Result<bool>;

    /// `left...right`에서 왼쪽에만 있는 버전 수와 오른쪽에만 있는 버전 수.
    fn count_between(&self, repo: &RepoPath, left: &str, right: &str) -> Result<(u32, u32)>;

    /// 두 버전이 갈라진 버전. 기록이 이어져 있지 않으면 `None`.
    fn merge_base(&self, repo: &RepoPath, a: &str, b: &str) -> Result<Option<Oid>>;

    /// 상태를 바꾸는 작업은 모두 여기로 온다. [`GitOp::Init`]에서 `repo`는 대상 폴더다.
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;
}
