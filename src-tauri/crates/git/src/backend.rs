use std::path::{Path, PathBuf};

use crate::{
    BlobHead, Branch, Commit, DiffTarget, FileDiff, GitOp, GitVersion, IndexEntry, LogQuery,
    MergeCheck, Oid, OpOutput, RefEntry, RefExpect, Remote, RepoPath, Result, Status,
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

    /// worktree끼리 함께 쓰는 git 폴더(절대 경로). hooks와 설정이 여기 있다. worktree가 아니면 `git_dir`와 같다.
    fn common_dir(&self, repo: &RepoPath) -> Result<PathBuf>;

    /// `ancestor`가 `descendant`의 조상이거나 같은 버전인지.
    fn is_ancestor(&self, repo: &RepoPath, ancestor: &str, descendant: &str) -> Result<bool>;

    /// `left...right`에서 왼쪽에만 있는 버전 수와 오른쪽에만 있는 버전 수.
    fn count_between(&self, repo: &RepoPath, left: &str, right: &str) -> Result<(u32, u32)>;

    /// 두 버전이 갈라진 버전. 기록이 이어져 있지 않으면 `None`.
    fn merge_base(&self, repo: &RepoPath, a: &str, b: &str) -> Result<Option<Oid>>;

    /// `ours`에 `theirs`를 합치면 겹치는 파일이 있는지 미리 본다. index와 작업 폴더는 그대로다.
    /// `base`가 없으면 두 버전이 갈라진 곳을 기준으로 한다.
    fn merge_check(
        &self,
        repo: &RepoPath,
        base: Option<&str>,
        ours: &str,
        theirs: &str,
    ) -> Result<MergeCheck>;

    /// 지금 index를 트리로 저장한 해시(되돌리기에 쓴다). 충돌 중이라 만들 수 없으면 `None`.
    fn write_tree(&self, repo: &RepoPath) -> Result<Option<Oid>>;

    /// 지금 브랜치 이름. 특정 버전을 보는 중이면 `None`.
    fn current_branch(&self, repo: &RepoPath) -> Result<Option<String>>;

    /// 맡겨 둔 stash에 든 파일 경로(고친 파일과 새 파일).
    fn parked_paths(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>>;

    /// 저장소 설정(local, worktree. include 포함)의 (키, 값) 목록. 키는 git이 주는 소문자 형태.
    fn local_config(&self, repo: &RepoPath) -> Result<Vec<(String, String)>>;

    /// index 항목 전부(`ls-files -s -z`). 충돌 중이면 stage 1·2·3이 함께 나온다.
    fn index_entries(&self, repo: &RepoPath) -> Result<Vec<IndexEntry>>;

    /// 블롭들의 크기와 앞 `max_bytes`바이트. 1MB가 넘는 블롭은 앞부분을 읽지 않는다.
    /// 없는 해시는 결과에서 빠진다. 프로세스는 둘(`cat-file --batch-check`, `--batch`)만 띄운다.
    fn read_blobs(&self, repo: &RepoPath, oids: &[Oid], max_bytes: usize) -> Result<Vec<BlobHead>>;

    /// 이 저장소에서 도는 모든 git 호출에 `-c 키=값`을 붙인다(D-039 "끄고 열기"). 빈 목록이면 지운다.
    fn set_repo_overrides(&self, repo: &RepoPath, pairs: Vec<String>);

    /// 상태를 바꾸는 작업은 모두 여기로 온다. [`GitOp::Init`]에서 `repo`는 대상 폴더다.
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;
}
