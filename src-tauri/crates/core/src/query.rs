use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use git::{GitBackend, RepoPath};

use crate::access::{OpenedRepos, check_path_len};
use crate::actions::paths;
use crate::parked;
use crate::recent::RecentStore;
use crate::trust::{TrustDecision, TrustState, TrustStore};
use crate::view::{
    BranchListView, BranchNameCheck, BranchRow, CommitRow, DiffRequest, FileDiffView, HomeMapView,
    LogPage, LogQuery, MapCommit, MergeHintKind, MergeHintView, NameProblem, ProjectSummaryView,
    SecretCheckRequest, SecretCheckView, SecretFinding, SecretRule, StatusView,
};
use crate::view::{
    ConflictContext, ConflictFileKind, ConflictFileSummary, ConflictFileView, ConflictSide,
    ConflictStateView,
};
use crate::{CoreError, Result};

/// 조회(D-010). 미리보기·위험도·백업이 없으므로 실행기와 레지스트리를 거치지 않는다.
pub struct Queries {
    git: Arc<dyn GitBackend>,
    opened: Arc<OpenedRepos>,
    recent: Arc<RecentStore>,
    trust: Arc<TrustStore>,
    /// 합치기 제안 결과(저장소 → (HEAD, 기본 브랜치 해시, 결과)). 둘이 그대로면 다시 계산하지 않는다.
    hint_cache: Mutex<HashMap<PathBuf, (String, String, MergeHintView)>>,
    /// 겹침 검사가 오래 걸린 저장소. 제안에서 겹친 파일 표시를 뺀다.
    slow_overlap: Mutex<HashSet<PathBuf>>,
}

/// 합치기 제안의 겹침 검사가 이보다 오래 걸리면 그 저장소에서는 다시 하지 않는다.
const SLOW_OVERLAP: Duration = Duration::from_secs(1);

/// 한 번에 읽는 버전 기록의 최대 개수.
pub const MAX_LOG_PAGE: u32 = 500;

impl Queries {
    pub(crate) fn new(
        git: Arc<dyn GitBackend>,
        opened: Arc<OpenedRepos>,
        recent: Arc<RecentStore>,
        trust: Arc<TrustStore>,
    ) -> Self {
        Self {
            git,
            opened,
            recent,
            trust,
            hint_cache: Mutex::new(HashMap::new()),
            slow_overlap: Mutex::new(HashSet::new()),
        }
    }

    /// 충돌 해결 화면의 상태(D-069). 합치는 중이 아니면 `active: false`.
    pub fn conflict_state(&self, repo: &Path) -> Result<ConflictStateView> {
        const MAX_FILES: usize = 200;
        let root = self.opened_repo(repo)?;
        let merge_head = self.git.resolve_ref(&root, "MERGE_HEAD")?;
        let entries = self.git.index_entries(&root)?;
        let conflicts = crate::conflict::conflict_paths(entries);
        let other = crate::conflict::other_operation(&self.git.git_dir(&root)?).is_some();
        let context = crate::conflict::context(self.git.as_ref(), &root)?;
        let mut view = ConflictStateView {
            active: context.is_some(),
            context: match context {
                Some(crate::conflict::Context::Unpark(_)) => ConflictContext::Unpark,
                _ => ConflictContext::Merge,
            },
            ours: None,
            theirs: None,
            files: Vec::new(),
            more_files: 0,
            clean_files: 0,
            other_operation: context.is_none() && (other || !conflicts.is_empty()),
        };
        let theirs_rev = match (&context, merge_head) {
            (Some(crate::conflict::Context::Unpark(m)), _) => match git::Oid::parse(&m.oid) {
                Some(oid) => oid,
                None => return Ok(view),
            },
            (Some(crate::conflict::Context::Merge), Some(head)) => head,
            _ => return Ok(view),
        };
        let merge_head = theirs_rev;
        let branches = self.git.branches(&root)?;
        let side = |rev: &str, label: Option<String>| -> Result<Option<ConflictSide>> {
            let commit = self
                .git
                .log(
                    &root,
                    &git::LogQuery {
                        skip: 0,
                        limit: 1,
                        rev: Some(rev.to_string()),
                    },
                )?
                .into_iter()
                .next();
            Ok(commit.map(|c| ConflictSide {
                label: label.unwrap_or_else(|| c.short_oid.clone()),
                author: c.author_name,
                at: c.author_time,
                summary: c.summary,
                short_oid: c.short_oid,
            }))
        };
        let current = self.git.current_branch(&root)?;
        let theirs_name = branches
            .iter()
            .filter(|b| b.oid == merge_head.as_str() && !b.is_head)
            .min_by_key(|b| b.kind != git::BranchKind::Local)
            .map(|b| b.name.clone());
        view.ours = side("HEAD", current)?;
        view.theirs = side(merge_head.as_str(), theirs_name)?;
        view.more_files = u32::try_from(conflicts.len().saturating_sub(MAX_FILES)).unwrap_or(0);
        for c in conflicts.iter().take(MAX_FILES) {
            let file = crate::conflict::read_file(self.git.as_ref(), &root, c)?;
            let hunks = match file.kind {
                ConflictFileKind::Text => u32::try_from(file.hunks.len()).unwrap_or(u32::MAX),
                ConflictFileKind::Whole => 1,
            };
            view.files.push(ConflictFileSummary {
                path: c.path.clone(),
                kind: file.kind,
                hunks,
            });
        }
        let status = self.git.status(&root)?;
        view.clean_files = u32::try_from(
            status
                .entries
                .iter()
                .filter(|e| e.kind != git::EntryKind::Unmerged)
                .filter(|e| !matches!(e.index, git::Change::Unmodified | git::Change::Untracked))
                .count(),
        )
        .unwrap_or(u32::MAX);
        Ok(view)
    }

    /// 충돌 파일 하나의 고를 곳(D-086). 충돌 중인 경로만 받는다.
    pub fn conflict_file(&self, repo: &Path, path: &str) -> Result<ConflictFileView> {
        let root = self.opened_repo(repo)?;
        paths::validate_one(path)?;
        let conflicts = crate::conflict::conflict_paths(self.git.index_entries(&root)?);
        let c = conflicts
            .iter()
            .find(|c| c.path == path)
            .ok_or(CoreError::InvalidInput("충돌한 파일이 아니에요"))?;
        crate::conflict::read_file(self.git.as_ref(), &root, c)
    }

    /// 임시 보관 목록(D-076). 맡겨 두기가 잠깐 만든 항목("higit parked …")은 뺀다.
    pub fn stashes(&self, repo: &Path) -> Result<crate::view::StashListView> {
        use crate::actions::stash::{PARKED_PREFIX, STASH_PREFIX, stash_message};
        let root = self.opened_repo(repo)?;
        let mut items = Vec::new();
        for e in self.git.stash_list(&root)? {
            let message = stash_message(&e.subject).to_string();
            if message.starts_with(PARKED_PREFIX) {
                continue;
            }
            let files = self.git.parked_paths(&root, &e.oid)?.len();
            let branch = message
                .strip_prefix(STASH_PREFIX)
                .map(|b| b.trim().to_string())
                .filter(|b| !b.is_empty());
            items.push(crate::view::StashItem {
                oid: e.oid.to_string(),
                message,
                branch,
                at: e.at,
                files: u32::try_from(files).unwrap_or(u32::MAX),
            });
        }
        Ok(crate::view::StashListView { items })
    }

    /// 합치기 제안(D-048, D-071). 기본 브랜치에 새 버전이 있으면 "가져와 합치기", 내 브랜치만
    /// 앞서 있으면 "main에 합치기". 기준은 홈 노선도와 같은 로컬 기본 브랜치다.
    pub fn merge_hint(&self, repo: &Path) -> Result<MergeHintView> {
        let root = self.opened_repo(repo)?;
        let status = self.git.status(&root)?;
        let (Some(current), Some(head)) = (status.branch.head.clone(), status.branch.oid.clone())
        else {
            return Ok(MergeHintView::none());
        };
        let branches = self.git.branches(&root)?;
        let remotes = self.git.remotes(&root)?;
        let Some(default) = self.default_branch(&root, &branches, &remotes)? else {
            return Ok(MergeHintView::none());
        };
        let main_ref = format!("refs/heads/{default}");
        let main_oid = match self.git.resolve_ref(&root, &main_ref)? {
            Some(oid) if default != current => oid.to_string(),
            _ => return Ok(MergeHintView::none()),
        };
        let key = root.as_path().to_path_buf();
        if let Some((h, m, view)) = self
            .hint_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
            && *h == head
            && *m == main_oid
        {
            return Ok(view.clone());
        }
        let (behind, ahead) = self.git.count_between(&root, &main_ref, "HEAD")?;
        let kind = if behind > 0 {
            MergeHintKind::Bring
        } else if ahead > 0 {
            MergeHintKind::Into
        } else {
            MergeHintKind::None
        };
        let mut view = MergeHintView {
            kind,
            branch: Some(default),
            behind,
            ahead,
            overlap: Vec::new(),
            overlap_count: 0,
        };
        let slow = self
            .slow_overlap
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(&key);
        // 갈라졌을 때만 겹침을 본다. 오래 걸린 저장소에서는 다시 하지 않는다.
        if behind > 0 && ahead > 0 && !slow {
            let start = Instant::now();
            if let git::MergeCheck::Overlap { paths } =
                self.git.merge_check(&root, None, "HEAD", &main_ref)?
            {
                view.overlap_count = u32::try_from(paths.len()).unwrap_or(u32::MAX);
                view.overlap = paths.into_iter().take(5).collect();
            }
            if start.elapsed() > SLOW_OVERLAP {
                self.slow_overlap
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(key.clone());
            }
        }
        self.hint_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key, (head, main_oid, view.clone()));
        Ok(view)
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

    /// 버전 저장 전에 넣을 파일에서 키·비밀번호로 보이는 것을 찾는다(D-040, D-077).
    /// 넣은 파일은 index의 내용을, 넣은 파일이 없고 `stage_all`이면 작업 폴더 파일을 본다.
    /// 링크는 읽지 않는다. 일치한 글자는 결과에 담지 않는다.
    pub fn secret_check(
        &self,
        repo: &Path,
        request: &SecretCheckRequest,
    ) -> Result<SecretCheckView> {
        use git::Change;
        const MAX_FILES: usize = 200;
        let root = self.opened_repo(repo)?;
        let status = self.git.status(&root)?;
        let staged: Vec<&git::StatusEntry> = status
            .entries
            .iter()
            .filter(|e| {
                !matches!(
                    e.index,
                    Change::Unmodified | Change::Untracked | Change::Deleted
                )
            })
            .filter(|e| e.kind != git::EntryKind::Unmerged)
            .collect();
        let from_index = !staged.is_empty();
        let targets: Vec<&git::StatusEntry> = if from_index {
            staged
        } else if request.stage_all {
            status
                .entries
                .iter()
                .filter(|e| !matches!(e.worktree, Change::Unmodified | Change::Deleted))
                .filter(|e| e.kind != git::EntryKind::Unmerged)
                .collect()
        } else {
            Vec::new()
        };
        let mut view = SecretCheckView {
            skipped: u32::try_from(targets.len().saturating_sub(MAX_FILES)).unwrap_or(u32::MAX),
            ..SecretCheckView::default()
        };
        let targets = &targets[..targets.len().min(MAX_FILES)];

        // (경로, 크기, 앞부분)
        let mut files: Vec<(String, u64, Vec<u8>)> = Vec::new();
        if from_index {
            let index = self.git.index_entries(&root)?;
            let oids: Vec<(String, git::Oid)> = targets
                .iter()
                .filter_map(|t| {
                    index
                        .iter()
                        .find(|e| e.stage == 0 && e.path == t.path && e.mode != "120000")
                        .map(|e| (t.path.clone(), e.oid.clone()))
                })
                .collect();
            let wanted: Vec<git::Oid> = oids.iter().map(|(_, o)| o.clone()).collect();
            let blobs = self
                .git
                .read_blobs(&root, &wanted, crate::secrets::SCAN_BYTES)?;
            for (path, oid) in oids {
                if let Some(b) = blobs.iter().find(|b| b.oid == oid) {
                    files.push((path, b.size, b.head.clone()));
                }
            }
        } else {
            for t in targets {
                if paths::validate_one(&t.path).is_err() {
                    continue;
                }
                let full = root.as_path().join(&t.path);
                let Ok(meta) = std::fs::symlink_metadata(&full) else {
                    continue;
                };
                if !meta.is_file() {
                    continue;
                }
                files.push((t.path.clone(), meta.len(), read_head(&full)));
            }
        }

        for (path, size, head) in files {
            let finding = if let Some(rule) = crate::secrets::name_rule(&path) {
                Some((rule, None))
            } else if size >= crate::secrets::LARGE_BYTES {
                Some((SecretRule::LargeFile, None))
            } else {
                crate::secrets::content_rule(&head).map(|(rule, line)| (rule, Some(line)))
            };
            if let Some((rule, line)) = finding {
                view.findings.push(SecretFinding { path, rule, line });
            }
        }
        Ok(view)
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
        // 믿기로 하지 않은 저장소는 status를 돌리지 않는다(core.fsmonitor가 status에서 돈다, D-067).
        match self.trust.check(self.git.as_ref(), &root)? {
            TrustState::Clean | TrustState::Decided(TrustDecision::Trust, _) => {}
            TrustState::Decided(TrustDecision::Disable, found) => {
                self.git.set_repo_overrides(&root, found.overrides);
            }
            TrustState::Unknown(_) => return Err(CoreError::NeedsTrust),
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
            remotes: remotes.into_iter().map(|r| r.name).collect(),
        })
    }

    /// 홈 두 줄 노선도(D-046). main과 지금 브랜치만, 각 줄 최근 몇 개.
    pub fn home_map(&self, repo: &Path) -> Result<HomeMapView> {
        const SHOWN: u32 = 3;
        let root = self.opened_repo(repo)?;
        let status = self.git.status(&root)?;
        let changed_files = u32::try_from(status.entries.len()).unwrap_or(u32::MAX);
        let current = status.branch.head.clone();
        let mut view = HomeMapView {
            current: current.clone(),
            default_branch: None,
            two_lines: false,
            current_commits: Vec::new(),
            current_more: 0,
            main_since: 0,
            main_last: None,
            upstream_oid: None,
            upstream_label: None,
            changed_files,
        };
        if status.branch.oid.is_none() {
            return Ok(view);
        }
        let branches = self.git.branches(&root)?;
        let remotes = self.git.remotes(&root)?;
        let default = self.default_branch(&root, &branches, &remotes)?;
        view.default_branch = default.clone();
        let log = |rev: Option<String>, limit: u32| -> Result<Vec<MapCommit>> {
            let mut commits: Vec<MapCommit> = self
                .git
                .log(
                    &root,
                    &git::LogQuery {
                        skip: 0,
                        limit,
                        rev,
                    },
                )?
                .into_iter()
                .map(|c| MapCommit {
                    oid: c.oid,
                    short_oid: c.short_oid,
                    summary: c.summary,
                })
                .collect();
            commits.reverse();
            Ok(commits)
        };
        let fork = match (&default, &current) {
            (Some(d), Some(c)) if d != c => {
                let main_ref = format!("refs/heads/{d}");
                self.git
                    .merge_base(&root, &main_ref, "HEAD")?
                    .map(|base| (main_ref, base))
            }
            _ => None,
        };
        match fork {
            Some((main_ref, base)) => {
                let (main_only, mine) = self.git.count_between(&root, &main_ref, "HEAD")?;
                view.two_lines = true;
                view.main_since = main_only;
                view.current_commits = log(Some(format!("{base}..HEAD")), SHOWN)?;
                view.current_more = mine.saturating_sub(SHOWN);
                if main_only > 0 {
                    view.main_last = log(Some(main_ref), 1)?.pop();
                }
            }
            None => {
                view.current_commits = log(None, SHOWN + 1)?;
                if view.current_commits.len() > SHOWN as usize {
                    view.current_commits.remove(0);
                    view.current_more = 1;
                }
            }
        }
        if let Some(upstream) = &status.branch.upstream
            && let Some(oid) = self
                .git
                .resolve_ref(&root, &format!("refs/remotes/{upstream}"))?
        {
            view.upstream_oid = Some(oid.to_string());
            let remote = remotes
                .iter()
                .filter(|r| upstream.starts_with(&format!("{}/", r.name)))
                .max_by_key(|r| r.name.len());
            view.upstream_label = remote.map(|r| {
                if remote_host(&r.url).is_some_and(|h| h.eq_ignore_ascii_case("github.com")) {
                    "GitHub".to_string()
                } else {
                    r.name.clone()
                }
            });
        }
        Ok(view)
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
/// 원격 주소의 호스트. `https://host/…`, `ssh://user@host:22/…`, `user@host:path`.
/// 경로에 든 글자(`https://evil/github.com/x`)로 판단하지 않게 호스트만 꺼낸다.
fn remote_host(url: &str) -> Option<&str> {
    let rest = match url.split_once("://") {
        Some((_, rest)) => rest.split('/').next()?,
        // scp 형식: 첫 `:` 앞이 `[user@]host`
        None => url.split_once(':')?.0,
    };
    let host = rest.rsplit_once('@').map_or(rest, |(_, h)| h);
    let host = host.split(':').next()?;
    (!host.is_empty()).then_some(host)
}

/// 파일 앞부분(비밀 검사 상한까지).
fn read_head(path: &Path) -> Vec<u8> {
    use std::io::Read;
    let mut buf = Vec::new();
    if let Ok(file) = std::fs::File::open(path) {
        let _ = file
            .take(crate::secrets::SCAN_BYTES as u64)
            .read_to_end(&mut buf);
    }
    buf
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_comes_from_the_host_part_only() {
        assert_eq!(
            remote_host("https://github.com/a/b.git"),
            Some("github.com")
        );
        assert_eq!(
            remote_host("https://user:pw@GitHub.com/a/b"),
            Some("GitHub.com")
        );
        assert_eq!(
            remote_host("ssh://git@github.com:22/a/b"),
            Some("github.com")
        );
        assert_eq!(remote_host("git@github.com:a/b.git"), Some("github.com"));
        assert_eq!(
            remote_host("https://evil.example/github.com/x"),
            Some("evil.example")
        );
        assert_eq!(remote_host("/local/path"), None);
    }
}
