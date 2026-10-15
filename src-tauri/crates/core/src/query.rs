use std::path::Path;
use std::sync::Arc;

use git::{GitBackend, RepoPath};

use crate::access::{OpenedRepos, check_path_len};
use crate::actions::paths;
use crate::parked;
use crate::recent::RecentStore;
use crate::view::{
    BranchListView, BranchNameCheck, BranchRow, CommitRow, DiffRequest, FileDiffView, LogPage,
    LogQuery, NameProblem, ProjectSummaryView, StatusView,
};
use crate::{CoreError, Result};

/// 조회(D-010). 미리보기·위험도·백업이 없으므로 실행기와 레지스트리를 거치지 않는다.
pub struct Queries {
    git: Arc<dyn GitBackend>,
    opened: Arc<OpenedRepos>,
    recent: Arc<RecentStore>,
}

/// 한 번에 읽는 버전 기록의 최대 개수.
pub const MAX_LOG_PAGE: u32 = 500;

impl Queries {
    pub(crate) fn new(
        git: Arc<dyn GitBackend>,
        opened: Arc<OpenedRepos>,
        recent: Arc<RecentStore>,
    ) -> Self {
        Self {
            git,
            opened,
            recent,
        }
    }

    /// 화면이 보낸 저장소 경로를 정규화하고, 열린 프로젝트인지 확인한다.
    fn opened_repo(&self, path: &Path) -> Result<RepoPath> {
        let repo = repo_path(path)?;
        self.opened.require(repo.as_path())?;
        Ok(repo)
    }

    pub fn status(&self, repo: &Path) -> Result<StatusView> {
        Ok(self.git.status(&self.opened_repo(repo)?)?.into())
    }

    /// 파일 하나를 마지막 저장과 비교한다(넣은 것과 안 넣은 것을 합쳐서).
    pub fn diff(&self, repo: &Path, request: &DiffRequest) -> Result<FileDiffView> {
        // 새 파일 비교(--no-index)는 git이 경로를 저장소로 묶지 않으므로 여기서 막는다.
        let root = self.opened_repo(repo)?;
        for path in std::iter::once(&request.path).chain(request.orig_path.as_ref()) {
            paths::validate_one(path)?;
        }
        // 링크는 안·밖 어디를 가리키든 따라가 읽지 않고 대상만 보여준다(D-052).
        // 링크 자신이 밖을 가리키는 폴더 링크 아래에 있으면 막는다.
        if let Some(target) = link_target(root.as_path(), &request.path) {
            if let Some(parent) = Path::new(&request.path).parent() {
                stays_inside(root.as_path(), &parent.to_string_lossy())?;
            }
            return Ok(FileDiffView::link(target));
        }
        for path in std::iter::once(&request.path).chain(request.orig_path.as_ref()) {
            stays_inside(root.as_path(), path)?;
        }
        Ok(self.git.diff(&root, &request.into())?.into())
    }

    /// 시작 화면의 프로젝트 상태 요약(D-031). 여러 프로젝트를 한꺼번에 읽으므로 새 git 호출을
    /// 만들지 않고 status·log만 쓴다. 둘 다 `GIT_OPTIONAL_LOCKS=0`이라 다른 작업의 index.lock과
    /// 부딪히지 않는다. 기록한 저장소의 최상위가 아니면(바깥 저장소가 잡히면) `NotARepo`.
    pub fn project_summary(&self, repo: &Path) -> Result<ProjectSummaryView> {
        let root = repo_path(repo)?;
        // 시작 화면은 열지 않은 최근 프로젝트의 상태도 읽는다.
        let listed = self
            .recent
            .list()
            .iter()
            .any(|e| platform::same_path(Path::new(&e.path), root.as_path()));
        if !listed {
            self.opened.require(root.as_path())?;
        }
        let top = self
            .git
            .discover(root.as_path())?
            .ok_or(CoreError::NotARepo)?;
        let top = platform::normalize_repo_path(top.as_path())
            .unwrap_or_else(|_| top.as_path().to_path_buf());
        if !platform::same_path(&top, root.as_path()) {
            return Err(CoreError::NotARepo);
        }
        let status = self.git.status(&root)?;
        let conflicted = status
            .entries
            .iter()
            .filter(|e| e.kind == git::EntryKind::Unmerged)
            .count();
        let last_commit = if status.branch.oid.is_some() {
            self.git
                .log(
                    &root,
                    &git::LogQuery {
                        skip: 0,
                        limit: 1,
                        rev: None,
                    },
                )?
                .into_iter()
                .next()
                .map(CommitRow::from)
        } else {
            None
        };
        let changed_files = u32::try_from(status.entries.len()).unwrap_or(u32::MAX);
        let view = StatusView::from(status);
        Ok(ProjectSummaryView {
            branch: view.branch,
            changed_files,
            conflicted_files: u32::try_from(conflicted).unwrap_or(u32::MAX),
            last_commit,
        })
    }

    /// 브랜치 목록(D-047). 맡겨 둔 파일 수와 기본 브랜치를 함께 준다.
    pub fn branches(&self, repo: &Path) -> Result<BranchListView> {
        let root = self.opened_repo(repo)?;
        let branches = self.git.branches(&root)?;
        let remotes = self.git.remotes(&root)?;
        let parked_refs = self.git.list_refs(&root, parked::PARKED_PREFIX)?;
        let default_branch = self.default_branch(&root, &branches, &remotes)?;

        let mut parked: Vec<(String, u32)> = Vec::new();
        for r in &parked_refs {
            let (Some(branch), Some(oid)) = (parked::branch_of(&r.name), git::Oid::parse(&r.oid))
            else {
                continue;
            };
            let count = self.git.parked_paths(&root, &oid)?.len();
            parked.push((branch, u32::try_from(count).unwrap_or(u32::MAX)));
        }
        let parked_count = |name: &str| {
            parked
                .iter()
                .find(|(b, _)| b == name)
                .map_or(0, |(_, n)| *n)
        };

        let mut local = Vec::new();
        let mut remote = Vec::new();
        for b in &branches {
            let is_local = b.kind == git::BranchKind::Local;
            let count = if is_local { parked_count(&b.name) } else { 0 };
            let row = BranchRow {
                name: b.name.clone(),
                remote: match &b.kind {
                    git::BranchKind::Remote { remote, .. } => Some(remote.clone()),
                    git::BranchKind::Local => None,
                },
                is_head: b.is_head,
                is_default: is_local && default_branch.as_deref() == Some(b.name.as_str()),
                upstream: b.upstream.clone(),
                ahead: b.ahead,
                behind: b.behind,
                gone: b.gone,
                last_author: b.last_author.clone(),
                last_at: b.last_time,
                parked_count: count,
                park_blocked: b.is_head && count > 0,
            };
            if is_local {
                local.push(row);
            } else {
                remote.push(row);
            }
        }
        let orphan_parked = parked
            .iter()
            .filter(|(name, _)| !local.iter().any(|b| &b.name == name))
            .map(|(name, _)| name.clone())
            .collect();
        Ok(BranchListView {
            local,
            remote,
            default_branch,
            orphan_parked,
        })
    }

    /// 새 브랜치 이름으로 쓸 수 있는지. 대소문자만 다른 기존 이름도 막는다(Windows ref 충돌).
    pub fn check_branch_name(&self, repo: &Path, name: &str) -> Result<BranchNameCheck> {
        let root = self.opened_repo(repo)?;
        let problem = if name.trim().is_empty() {
            Some(NameProblem::Empty)
        } else if !self.git.check_branch_name(name)? {
            Some(NameProblem::Invalid)
        } else if self.git.branches(&root)?.iter().any(|b| {
            b.kind == git::BranchKind::Local && b.name.to_lowercase() == name.to_lowercase()
        }) {
            Some(NameProblem::Exists)
        } else {
            None
        };
        Ok(BranchNameCheck {
            ok: problem.is_none(),
            problem,
        })
    }

    /// 기본 브랜치: 지금 브랜치 upstream의 원격(없으면 origin, 그다음 첫 원격)의 HEAD,
    /// 그것도 없으면 로컬 main, master 순(D-048).
    fn default_branch(
        &self,
        root: &RepoPath,
        branches: &[git::Branch],
        remotes: &[git::Remote],
    ) -> Result<Option<String>> {
        let upstream_remote = branches
            .iter()
            .find(|b| b.is_head)
            .and_then(|b| b.upstream.as_deref())
            .and_then(|u| {
                remotes
                    .iter()
                    .filter(|r| u.starts_with(&format!("{}/", r.name)))
                    .max_by_key(|r| r.name.len())
            });
        let remote = upstream_remote
            .or_else(|| remotes.iter().find(|r| r.name == "origin"))
            .or_else(|| remotes.first());
        if let Some(remote) = remote
            && let Some(head) = self.git.remote_head(root, &remote.name)?
        {
            return Ok(Some(head));
        }
        let has_local = |n: &str| {
            branches
                .iter()
                .any(|b| b.kind == git::BranchKind::Local && b.name == n)
        };
        Ok(["main", "master"]
            .into_iter()
            .find(|n| has_local(n))
            .map(str::to_string))
    }

    pub fn log(&self, repo: &Path, query: LogQuery) -> Result<LogPage> {
        let limit = query.limit.min(MAX_LOG_PAGE);
        // 하나 더 받아서 다음 쪽이 있는지 안다.
        let mut commits = self.git.log(
            &self.opened_repo(repo)?,
            &git::LogQuery {
                skip: query.skip,
                limit: limit + 1,
                rev: None,
            },
        )?;
        let has_more = commits.len() > limit as usize;
        commits.truncate(limit as usize);
        Ok(LogPage {
            commits: commits.into_iter().map(CommitRow::from).collect(),
            has_more,
        })
    }
}

/// 화면이 보낸 경로를 폴더 열기 때와 같은 형식으로 정규화한다.
fn repo_path(path: &Path) -> Result<RepoPath> {
    check_path_len(path)?;
    platform::normalize_repo_path(path)
        .map(RepoPath::new)
        .map_err(CoreError::FolderNotFound)
}

/// `rel`이 심볼릭 링크(Windows 정션 포함)면 가리키는 곳. 아니면 `None`.
fn link_target(root: &Path, rel: &str) -> Option<String> {
    let full = root.join(rel);
    let meta = std::fs::symlink_metadata(&full).ok()?;
    if !meta.file_type().is_symlink() {
        return None;
    }
    let target = std::fs::read_link(&full).ok()?;
    Some(target.to_string_lossy().into_owned())
}

/// 경로 글자는 저장소 안이어도 심볼릭 링크가 밖을 가리킬 수 있다. 실제 위치를 풀어 확인한다.
/// 없는 파일(지운 파일)은 그대로 둔다. 대상이 없는 링크와 밖을 가리키는 링크는 막는다.
fn stays_inside(root: &Path, rel: &str) -> Result<()> {
    if rel.is_empty() {
        return Ok(());
    }
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
