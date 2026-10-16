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
    /// 앱이 시작한 작업의 표식을 확인하는 키(D-102).
    key: crate::undo::JournalKey,
    /// GitHub 연결(검토 요청 조회, D-112).
    github: Arc<crate::github::GitHubService>,
}

/// 합치기 제안의 겹침 검사가 이보다 오래 걸리면 그 저장소에서는 다시 하지 않는다.
const SLOW_OVERLAP: Duration = Duration::from_secs(1);

/// 검토 요청 목록 한 쪽의 개수.
const PR_PAGE: u32 = 30;
/// 댓글 수 상한(쪽 번호 계산이 엉뚱하게 커지지 않게, GitHub 목록은 쪽마다 20개).
const PR_COMMENTS_MAX: u32 = 100_000;
/// 새 검토 요청의 대상 브랜치 후보 최대 개수.
const PR_BASE_OPTIONS: usize = 100;

/// 한 번에 읽는 버전 기록의 최대 개수.
pub const MAX_LOG_PAGE: u32 = 500;

impl Queries {
    pub(crate) fn new(
        git: Arc<dyn GitBackend>,
        opened: Arc<OpenedRepos>,
        recent: Arc<RecentStore>,
        trust: Arc<TrustStore>,
        key: crate::undo::JournalKey,
        github: Arc<crate::github::GitHubService>,
    ) -> Self {
        Self {
            git,
            opened,
            recent,
            trust,
            hint_cache: Mutex::new(HashMap::new()),
            slow_overlap: Mutex::new(HashSet::new()),
            key,
            github,
        }
    }

    pub(crate) fn set_github(&mut self, github: Arc<crate::github::GitHubService>) {
        self.github = github;
    }

    /// 열린 저장소의 호스팅 정보(D-112). 네트워크와 키체인 없이 로컬 정보만.
    pub fn hosting_context(&self, repo: &Path) -> Result<crate::view::HostingContextView> {
        let root = self.opened_repo(repo)?;
        let status = self.github.status();
        let target = crate::github::pr::pr_target(self.git.as_ref(), &root)?;
        let github = target.as_ref().and_then(|t| t.repo.as_ref());
        let (branch, head, base) = match &target {
            Some(t) => (t.branch.clone(), t.head.clone(), t.default_branch.clone()),
            None => (self.git.current_branch(&root)?, None, None),
        };
        Ok(crate::view::HostingContextView {
            provider: github.map(|_| crate::view::HostingProviderView::Github),
            repo: github.map(|r| format!("{}/{}", r.owner, r.name)),
            configured: status.configured,
            connected: status.connected,
            is_default_branch: branch.is_some() && branch == base,
            branch,
            head,
            base,
            via_fork: target.as_ref().is_some_and(|t| t.via_fork),
        })
    }

    /// 지금 브랜치가 올리는 GitHub 저장소와 대상. GitHub가 아니면 `NotGithubRemote`.
    fn github_target(
        &self,
        root: &RepoPath,
    ) -> Result<(crate::github::pr::PrTarget, hosting::RemoteRepo)> {
        let target =
            crate::github::pr::pr_target(self.git.as_ref(), root)?.ok_or(CoreError::NoRemote)?;
        let repo = target.repo.clone().ok_or(CoreError::Github {
            kind: crate::GithubFailure::NotGithubRemote,
            detail: None,
        })?;
        Ok((target, repo))
    }

    /// 검토 요청 목록 한 쪽(D-112). 토큰이 필요하고 401이면 연결을 지운다.
    pub fn pr_list(
        &self,
        repo: &Path,
        request: &crate::view::PrListRequest,
    ) -> Result<crate::view::PrListView> {
        let root = self.opened_repo(repo)?;
        let (target, github) = self.github_target(&root)?;
        if let Some(branch) = &request.branch
            && !self.git.check_branch_name(branch)?
        {
            return Err(CoreError::InvalidInput("브랜치 이름"));
        }
        let query = hosting::PrQuery {
            state: crate::github::pr::state_filter(request.state),
            head_branch: request.branch.clone(),
            head_owner: target.head_owner().map(str::to_owned),
            page: request.page.max(1),
            per_page: PR_PAGE,
        };
        let page = self
            .github
            .call(|provider, token| provider.list_prs(token, &github, &query))?;
        Ok(crate::view::PrListView {
            items: page
                .items
                .iter()
                .map(|pr| crate::github::pr::row(pr, target.head.as_deref(), target.head_owner()))
                .collect(),
            has_more: page.has_more,
        })
    }

    /// 검토 요청 하나: 합칠 수 있는지와 시험 결과 요약.
    pub fn pr_detail(&self, repo: &Path, number: u32) -> Result<crate::view::PrDetailView> {
        let root = self.opened_repo(repo)?;
        if number == 0 {
            return Err(CoreError::InvalidInput("검토 요청 번호"));
        }
        let (target, github) = self.github_target(&root)?;
        let detail = self
            .github
            .call(|provider, token| provider.pr_detail(token, &github, u64::from(number)))?;
        Ok(crate::github::pr::detail(
            &detail,
            target.head.as_deref(),
            target.head_owner(),
        ))
    }

    /// 서버가 이 원격 브랜치를 보호하는지(D-183 ④). GitHub 원격이고 연결됐을 때만 묻고, 그 밖이나 읽지
    /// 못하면 `None`(안내만 빠진다). 확인 창이 열릴 때 한 번 읽는다.
    pub fn branch_protection(
        &self,
        repo: &Path,
        remote: &str,
        branch: &str,
    ) -> Result<Option<bool>> {
        let root = self.opened_repo(repo)?;
        crate::actions::remote_branch::checked_remote(self.git.as_ref(), &root, remote)?;
        crate::actions::names::validate_ref_like(branch)?;
        if !self.github.status().connected {
            return Ok(None);
        }
        let Some(github) = crate::github::pr::github_push_repo(self.git.as_ref(), &root, remote)?
        else {
            return Ok(None);
        };
        Ok(self
            .github
            .call(|provider, token| provider.branch_protected(token, &github, branch))
            .unwrap_or(None))
    }

    /// 검토 요청 화면의 저장소 정보(D-177): 올릴 권한, 허용된 합치기 방법.
    pub fn pr_repo(&self, repo: &Path) -> Result<crate::view::PrRepoView> {
        let root = self.opened_repo(repo)?;
        let (_, github) = self.github_target(&root)?;
        let info = self
            .github
            .call(|provider, token| provider.repo_info(token, &github))?;
        Ok(crate::github::pr::repo_view(
            &info,
            self.github.status().login,
        ))
    }

    /// 검토 요청의 최근 댓글 20개(D-178). `total`은 상세가 준 대화 댓글 수로, 마지막 쪽 계산에만 쓴다.
    pub fn pr_comments(
        &self,
        repo: &Path,
        number: u32,
        total: u32,
    ) -> Result<crate::view::PrCommentsView> {
        let root = self.opened_repo(repo)?;
        if number == 0 {
            return Err(CoreError::InvalidInput("검토 요청 번호"));
        }
        let (_, github) = self.github_target(&root)?;
        let total = total.min(PR_COMMENTS_MAX);
        let (last, prev) = crate::github::pr::comment_pages(total);
        let per = hosting::COMMENTS_PAGE;
        let n = u64::from(number);
        let (older, newest) = self.github.call(|provider, token| {
            let newest = provider.list_comments(token, &github, n, last, per)?;
            let older = match prev {
                Some(page) => provider.list_comments(token, &github, n, page, per)?,
                None => Vec::new(),
            };
            Ok((older, newest))
        })?;
        let viewer = self.github.status().login;
        Ok(crate::github::pr::comments_view(
            older,
            newest,
            total,
            viewer.as_deref(),
        ))
    }

    /// 새 검토 요청 대화상자의 기본값(D-113). 네트워크 없이 로컬 정보만.
    pub fn pr_draft(&self, repo: &Path) -> Result<crate::view::PrDraftView> {
        let root = self.opened_repo(repo)?;
        let (target, _) = self.github_target(&root)?;
        let branch = target.branch.clone().ok_or(CoreError::InvalidInput(
            "특정 버전을 보는 중에는 검토 요청을 만들 수 없어요",
        ))?;
        let mut options: Vec<String> = target
            .remote_branches
            .iter()
            .filter(|b| Some(*b) != target.head.as_ref() && **b != branch)
            .cloned()
            .collect();
        options.sort();
        if let Some(default) = &target.default_branch
            && let Some(i) = options.iter().position(|b| b == default)
        {
            let default = options.remove(i);
            options.insert(0, default);
        }
        options.truncate(PR_BASE_OPTIONS);
        let base = target
            .default_branch
            .clone()
            .filter(|d| options.contains(d))
            .or_else(|| options.first().cloned())
            .unwrap_or_default();
        let ahead = if base.is_empty() {
            0
        } else {
            let base_ref = format!("refs/remotes/{}/{base}", target.remote);
            match self.git.resolve_ref(&root, &base_ref)? {
                Some(_) => self.git.count_between(&root, "HEAD", &base_ref)?.0,
                None => 0,
            }
        };
        let unpushed = match &target.tracking {
            Some(tracking) => {
                self.git
                    .count_between(&root, "HEAD", &format!("refs/remotes/{tracking}"))?
                    .0
            }
            None => ahead,
        };
        let only_summary = if ahead == 1 {
            self.git
                .log(
                    &root,
                    &git::LogQuery {
                        skip: 0,
                        limit: 1,
                        rev: None,
                    },
                )?
                .first()
                .map(|c| c.summary.clone())
        } else {
            None
        };
        Ok(crate::view::PrDraftView {
            suggestion_title: crate::github::pr::suggest_title(
                &branch,
                ahead,
                only_summary.as_deref(),
            ),
            branch,
            via_fork: target.via_fork,
            head: target.head,
            base,
            base_options: options,
            ahead,
            unpushed,
        })
    }

    /// 복제 대화상자의 기본값. 저장소를 열지 않아도 된다.
    pub fn clone_defaults(&self) -> crate::view::CloneDefaultsView {
        crate::view::CloneDefaultsView {
            parent: platform::home_dir().map(|h| h.to_string_lossy().into_owned()),
        }
    }

    /// 열린 저장소의 GitHub 저장소. `origin`이 GitHub면 그것, 아니면 처음 나오는 GitHub 원격.
    pub(crate) fn github_repo(&self, repo: &Path) -> Result<hosting::RemoteRepo> {
        let root = self.opened_repo(repo)?;
        let remotes = self.git.remotes(&root)?;
        let found = remotes
            .iter()
            .filter_map(|r| hosting::GitHub::detect(&r.url).map(|g| (r.name == "origin", g)))
            .max_by_key(|(origin, _)| *origin);
        found.map(|(_, g)| g).ok_or(CoreError::NoRemote)
    }

    pub(crate) fn set_key(&mut self, key: crate::undo::JournalKey) {
        self.key = key;
    }

    /// 충돌 해결 화면의 상태(D-069). 합치는 중이 아니면 `active: false`.
    pub fn conflict_state(&self, repo: &Path) -> Result<ConflictStateView> {
        let (mut view, target) = self.conflict_state_with_target(repo)?;
        // 앞서 고른 것을 붙인다(D-211). 합칠 쪽이 바뀌었으면 지운다.
        if let Some(theirs) = &target {
            let git_dir = self.git.git_dir(&self.opened_repo(repo)?)?;
            view.draft = crate::conflict_draft::load(&git_dir, &self.key, theirs, &view.files);
        }
        Ok(view)
    }

    /// 충돌 해결 화면에서 고른 것을 남긴다(D-211). 합치는 중이 아니면 거절한다.
    pub(crate) fn save_conflict_draft(
        &self,
        repo: &Path,
        draft: &crate::view::ConflictDraftView,
    ) -> Result<()> {
        let (view, target) = self.conflict_state_with_target(repo)?;
        let theirs = target.ok_or(CoreError::InvalidInput("합치는 중이 아니에요"))?;
        let git_dir = self.git.git_dir(&self.opened_repo(repo)?)?;
        crate::conflict_draft::save(&git_dir, &self.key, &theirs, draft, &view.files)
    }

    /// 충돌 상태와 합칠 쪽 버전 전체 해시(고른 것을 묶는 대상).
    fn conflict_state_with_target(
        &self,
        repo: &Path,
    ) -> Result<(ConflictStateView, Option<String>)> {
        use crate::conflict::Context;
        use crate::operation::OpKind;
        const MAX_FILES: usize = 200;
        let root = self.opened_repo(repo)?;
        let context = crate::conflict::context(self.git.as_ref(), &root, &self.key)?;
        let flip = context.as_ref().is_some_and(Context::flipped);
        let entries = self.git.index_entries(&root)?;
        let conflicts = crate::conflict::conflict_paths(entries, flip);
        let other =
            crate::conflict::foreign_operation(self.git.as_ref(), &root, &self.key)?.is_some();
        let mut view = ConflictStateView {
            active: context.is_some(),
            context: match &context {
                Some(Context::Unpark(_)) => ConflictContext::Unpark,
                Some(Context::Operation(op)) => op.kind.context(),
                _ => ConflictContext::Merge,
            },
            ours: None,
            theirs: None,
            files: Vec::new(),
            more_files: 0,
            clean_files: 0,
            other_operation: context.is_none() && (other || !conflicts.is_empty()),
            draft: None,
        };
        let Some(context) = context else {
            return Ok((view, None));
        };
        let branches = self.git.branches(&root)?;
        // 그 버전을 가리키는 로컬 브랜치, 없으면 원격 브랜치 이름. 없으면 짧은 해시를 쓴다.
        let name_of = |oid: &git::Oid| {
            branches
                .iter()
                .filter(|b| b.oid == oid.as_str() && !b.is_head)
                .min_by_key(|b| b.kind != git::BranchKind::Local)
                .map(|b| b.name.clone())
        };
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
        // 다른 쪽 버전: 합치기는 MERGE_HEAD, 꺼내기는 맡긴 항목, 복사·되돌리기는 고른 버전.
        let other_rev = |name: &str| self.git.resolve_ref(&root, name);
        let current = self.git.current_branch(&root)?;
        let target = match &context {
            Context::Operation(op) if op.kind == OpKind::Rebase => {
                // 내 쪽 = 옮기는 버전, 합칠 쪽 = 정리된 기록(지금 HEAD, 이름표는 정리하는 브랜치).
                if let Some(moving) = other_rev("REBASE_HEAD")? {
                    view.ours = side(moving.as_str(), None)?;
                }
                view.theirs = side("HEAD", Some(op.branch.clone()))?;
                other_rev("HEAD")?.map(|o| o.to_string())
            }
            _ => {
                let theirs = match &context {
                    Context::Merge => other_rev("MERGE_HEAD")?,
                    Context::Unpark(m) => git::Oid::parse(&m.oid),
                    Context::Operation(op) if op.kind == OpKind::CherryPick => {
                        other_rev("CHERRY_PICK_HEAD")?
                    }
                    Context::Operation(_) => other_rev("REVERT_HEAD")?,
                };
                let Some(theirs) = theirs else {
                    return Ok((view, None));
                };
                view.ours = side("HEAD", current)?;
                view.theirs = side(theirs.as_str(), name_of(&theirs))?;
                Some(theirs.to_string())
            }
        };
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
        Ok((view, target))
    }

    /// 충돌 파일 하나의 고를 곳(D-086). 충돌 중인 경로만 받는다.
    pub fn conflict_file(&self, repo: &Path, path: &str) -> Result<ConflictFileView> {
        let root = self.opened_repo(repo)?;
        paths::validate_one(path)?;
        let context = crate::conflict::context(self.git.as_ref(), &root, &self.key)?;
        let flip = context
            .as_ref()
            .is_some_and(crate::conflict::Context::flipped);
        let conflicts = crate::conflict::conflict_paths(self.git.index_entries(&root)?, flip);
        let c = conflicts
            .iter()
            .find(|c| c.path == path)
            .ok_or(CoreError::InvalidInput("충돌한 파일이 아니에요"))?;
        crate::conflict::read_file(self.git.as_ref(), &root, c)
    }

    /// 임시 보관 목록(D-076). 맡겨 두기가 잠깐 만든 항목("higit parked …")은 뺀다.
    pub fn stashes(&self, repo: &Path) -> Result<crate::view::StashListView> {
        use crate::actions::stash::{STASH_PREFIX, stash_message};
        let root = self.opened_repo(repo)?;
        // 맡겨 두기 항목은 알림 줄의 "복구하기"가 다룬다. 검사를 통과하지 못한 것은 그대로 보인다.
        let parked: HashSet<git::Oid> = crate::parked::leftovers(self.git.as_ref(), &root)?
            .into_iter()
            .map(|l| l.oid)
            .collect();
        let mut items = Vec::new();
        for e in self.git.stash_list(&root)? {
            if parked.contains(&e.oid) {
                continue;
            }
            let message = stash_message(&e.subject).to_string();
            // 받은 저장소의 이상한 항목 하나가 목록 전체를 막지 않게, 못 읽으면 0개로 센다.
            let files = self.git.parked_paths(&root, &e.oid).map_or(0, |p| p.len());
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

    /// 열린 저장소의 경로. `need`면 그 로컬 브랜치가 있어야 한다(고정, D-203).
    pub(crate) fn local_branch_exists(
        &self,
        repo: &Path,
        branch: &str,
        need: bool,
    ) -> Result<RepoPath> {
        let root = self.opened_repo(repo)?;
        crate::actions::names::validate_ref_like(branch)?;
        if need
            && self
                .git
                .resolve_ref(&root, &format!("refs/heads/{branch}"))?
                .is_none()
        {
            return Err(CoreError::InvalidInput("없는 브랜치예요"));
        }
        Ok(root)
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

    /// 이미지 비교의 두 쪽(D-206). 읽기만 한다. 이전 쪽은 object database의 블롭, 지금 쪽은 버전 안 비교면
    /// 그 버전의 블롭, 아니면 상태에 바뀐 파일로 있는 작업 폴더의 일반 파일(링크·저장소 밖은 읽지 않는다).
    /// 한쪽이 3MB를 넘으면 `too_large`, 한쪽이라도 이미지가 아니면 두 쪽 모두 없다(화면은 "이진 파일").
    pub fn image_pair(
        &self,
        repo: &Path,
        request: &crate::view::ImageRequest,
    ) -> Result<crate::view::ImagePairView> {
        use crate::view::{ImageBlob, ImagePairView};
        use base64::Engine as _;
        let root = self.opened_repo(repo)?;
        let git = self.git.as_ref();
        for path in std::iter::once(&request.path).chain(request.orig_path.as_ref()) {
            paths::validate_one(path)?;
            if !git::safe_relative(path) {
                return Err(CoreError::InvalidInput("저장소 밖 경로예요"));
            }
        }
        let before_path = request.orig_path.as_deref().unwrap_or(&request.path);
        let blob = |rev: &str, path: &str| -> Result<Option<ImageRead>> {
            let Some(oid) = git.blob_at(&root, rev, path)? else {
                return Ok(None);
            };
            Ok(git
                .read_blob(&root, &oid, IMAGE_MAX_BYTES)?
                .map(|(_, bytes)| match bytes {
                    Some(b) => ImageRead::Bytes(b),
                    None => ImageRead::TooLarge,
                }))
        };
        let (before, after) = match &request.commit {
            Some(commit) => {
                let oid = crate::guard::verified_commit(
                    git,
                    &root,
                    commit,
                    crate::guard::Scope::BranchOrActivity,
                )?;
                // 첫 버전이면 첫 부모가 없어 이전 쪽도 없다.
                (
                    blob(&format!("{oid}^1"), before_path)?,
                    blob(oid.as_str(), &request.path)?,
                )
            }
            None => {
                let before = match git.resolve_ref(&root, "HEAD")? {
                    Some(_) => blob("HEAD", before_path)?,
                    None => None,
                };
                let listed = git
                    .status(&root)?
                    .entries
                    .iter()
                    .any(|e| e.path == request.path);
                let after = if listed {
                    worktree_image(root.as_path(), &request.path)
                } else {
                    None
                };
                (before, after)
            }
        };
        let mut view = ImagePairView {
            before: None,
            after: None,
            too_large: false,
        };
        if matches!(before, Some(ImageRead::TooLarge)) || matches!(after, Some(ImageRead::TooLarge))
        {
            view.too_large = true;
            return Ok(view);
        }
        let encode = |read: Option<ImageRead>| -> std::result::Result<Option<ImageBlob>, ()> {
            match read {
                Some(ImageRead::Bytes(bytes)) => {
                    let mime = crate::image::sniff(&bytes).ok_or(())?;
                    Ok(Some(ImageBlob {
                        mime,
                        bytes: u32::try_from(bytes.len()).unwrap_or(u32::MAX),
                        data: base64::engine::general_purpose::STANDARD.encode(&bytes),
                    }))
                }
                _ => Ok(None),
            }
        };
        if let (Ok(b), Ok(a)) = (encode(before), encode(after)) {
            view.before = b;
            view.after = a;
        }
        Ok(view)
    }

    /// 파일 하나를 마지막 저장과 비교한다(넣은 것과 안 넣은 것을 합쳐서).
    pub fn diff(&self, repo: &Path, request: &DiffRequest) -> Result<FileDiffView> {
        // 새 파일 비교(--no-index)는 git이 경로를 저장소로 묶지 않으므로 여기서 막는다.
        let root = self.opened_repo(repo)?;
        for path in std::iter::once(&request.path).chain(request.orig_path.as_ref()) {
            paths::validate_one(path)?;
        }
        // 버전 안의 비교는 object database의 블롭만 읽는다(작업 폴더와 무관, D-105).
        if let Some(commit) = &request.commit {
            let oid = crate::guard::verified_commit(
                self.git.as_ref(),
                &root,
                commit,
                crate::guard::Scope::BranchOrActivity,
            )?;
            let target = git::DiffTarget {
                path: request.path.clone(),
                orig_path: request.orig_path.clone(),
                kind: git::DiffKind::Commit { oid: oid.clone() },
                full_context: request.full_context,
            };
            let mut view: FileDiffView = self.git.diff(&root, &target)?.into();
            if wants_line_count(&view) {
                view.new_line_count = self.blob_line_count(&root, oid.as_str(), &request.path);
            }
            return Ok(view);
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
        let mut view: FileDiffView = self.git.diff(&root, &request.into())?.into();
        if !request.untracked && wants_line_count(&view) {
            view.new_line_count = work_tree_line_count(root.as_path(), &request.path);
        }
        Ok(view)
    }

    /// 버전 안 파일의 줄 수(D-219). 그 버전에 없거나(지움) 1MB를 넘으면 `None`.
    fn blob_line_count(&self, root: &RepoPath, rev: &str, path: &str) -> Option<u32> {
        let oid = self.git.blob_at(root, rev, path).ok().flatten()?;
        let (_, bytes) = self
            .git
            .read_blob(root, &oid, crate::view::LINE_COUNT_LIMIT)
            .ok()
            .flatten()?;
        crate::view::line_count(&bytes?)
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

        // 압축·실행 알림은 새 파일에만(이미 추적 중인 자료를 고칠 때마다 알리지 않게, D-158).
        let new_paths: std::collections::HashSet<&str> = targets
            .iter()
            .filter(|t| t.index == Change::Added || t.kind == git::EntryKind::Untracked)
            .map(|t| t.path.as_str())
            .collect();
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
            // 비밀 이름 → 큰 파일 → 압축·실행 → 내용 순. 압축·실행은 머리가 맞을 때만 내용 검사를 건너뛴다.
            let kind = new_paths
                .contains(path.as_str())
                .then(|| crate::secrets::kind_rule(&path, &head))
                .flatten();
            let content =
                || crate::secrets::content_rule(&head).map(|(rule, line)| (rule, Some(line)));
            let finding = if let Some(rule) = crate::secrets::name_rule(&path) {
                Some((rule, None))
            } else if size >= crate::secrets::HUGE_BYTES {
                Some((SecretRule::HugeFile, None))
            } else if size >= crate::secrets::LARGE_BYTES {
                Some((SecretRule::LargeFile, None))
            } else if let Some((rule, true)) = kind {
                Some((rule, None))
            } else if let Some(found) = content() {
                Some(found)
            } else {
                kind.map(|(rule, _)| (rule, None))
            };
            if let Some((rule, line)) = finding {
                view.findings.push(SecretFinding { path, rule, line });
            }
        }
        Ok(view)
    }

    /// 시작 화면이 읽는 저장소: 최근 목록에 있거나 연 저장소의 최상위이고, 믿기로 했거나 실행
    /// 설정이 없는 것. 믿기로 하지 않은 저장소는 status 같은 git을 돌리지 않는다(D-067).
    /// 상태 요약과 노선도 미리보기가 같이 쓴다(한쪽만 느슨해지지 않게).
    fn summary_root(&self, repo: &Path) -> Result<RepoPath> {
        let root = repo_path(repo)?;
        // 시작 화면은 열지 않은 최근 프로젝트도 읽는다.
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
                self.git.set_repo_policy(&root, found.disabled_policy());
            }
            TrustState::Unknown(_) => return Err(CoreError::NeedsTrust),
        }
        Ok(root)
    }

    /// 시작 화면의 프로젝트 상태 요약(D-031). 여러 프로젝트를 한꺼번에 읽으므로 새 git 호출을
    /// 만들지 않고 status·log만 쓴다. 둘 다 `GIT_OPTIONAL_LOCKS=0`이라 다른 작업의 index.lock과
    /// 부딪히지 않는다. 기록한 저장소의 최상위가 아니면(바깥 저장소가 잡히면) `NotARepo`.
    pub fn project_summary(&self, repo: &Path) -> Result<ProjectSummaryView> {
        let root = self.summary_root(repo)?;
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
            // 못 읽는 맡긴 ref(받은 저장소가 꽂은 것)는 없는 것으로 본다.
            let Ok(paths) = self.git.parked_paths(&root, &oid) else {
                continue;
            };
            let count = paths.len();
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
            // 원격 줄의 기본 브랜치: 그 원격의 HEAD와 늘 지키는 이름(D-130, 계획 010 R1). 지우기·이름 바꾸기를 흐린다.
            let is_default = match &b.kind {
                git::BranchKind::Local => default_branch.as_deref() == Some(b.name.as_str()),
                git::BranchKind::Remote { remote, branch } => {
                    branch == "HEAD"
                        || crate::guard::protected_branch(
                            self.git.as_ref(),
                            &root,
                            Some(remote),
                            branch,
                        )?
                }
            };
            let row = BranchRow {
                name: b.name.clone(),
                remote: match &b.kind {
                    git::BranchKind::Remote { remote, .. } => Some(remote.clone()),
                    git::BranchKind::Local => None,
                },
                is_head: b.is_head,
                is_default,
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
        self.map_of(&self.opened_repo(repo)?, 3)
    }

    /// 시작 화면 노선도(D-093): 홈과 같은 요약이고 지금 줄 버전 수만 `limit`(1~20).
    /// 접근은 상태 요약과 같은 규칙.
    pub fn route_preview(&self, repo: &Path, limit: u32) -> Result<HomeMapView> {
        const MAX_PREVIEW: u32 = 20;
        let root = self.summary_root(repo)?;
        self.map_of(&root, limit.clamp(1, MAX_PREVIEW))
    }

    /// main과 지금 브랜치 요약. 지금 줄에는 버전을 `shown`개까지.
    fn map_of(&self, root: &RepoPath, shown: u32) -> Result<HomeMapView> {
        let status = self.git.status(root)?;
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
        let branches = self.git.branches(root)?;
        let remotes = self.git.remotes(root)?;
        let default = self.default_branch(root, &branches, &remotes)?;
        view.default_branch = default.clone();
        let log = |rev: Option<String>, limit: u32| -> Result<Vec<MapCommit>> {
            let mut commits: Vec<MapCommit> = self
                .git
                .log(
                    root,
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
                    .merge_base(root, &main_ref, "HEAD")?
                    .map(|base| (main_ref, base))
            }
            _ => None,
        };
        match fork {
            Some((main_ref, base)) => {
                let (main_only, mine) = self.git.count_between(root, &main_ref, "HEAD")?;
                view.two_lines = true;
                view.main_since = main_only;
                view.current_commits = log(Some(format!("{base}..HEAD")), shown)?;
                view.current_more = mine.saturating_sub(shown);
                if main_only > 0 {
                    view.main_last = log(Some(main_ref), 1)?.pop();
                }
            }
            None => {
                view.current_commits = log(None, shown + 1)?;
                if view.current_commits.len() > shown as usize {
                    view.current_commits.remove(0);
                    view.current_more = 1;
                }
            }
        }
        if let Some(upstream) = &status.branch.upstream
            && let Some(oid) = self
                .git
                .resolve_ref(root, &format!("refs/remotes/{upstream}"))?
        {
            view.upstream_oid = Some(oid.to_string());
            let remote = remotes
                .iter()
                .filter(|r| upstream.starts_with(&format!("{}/", r.name)))
                .max_by_key(|r| r.name.len());
            view.upstream_label = remote.map(|r| {
                if hosting::GitHub::detect(&r.url).is_some() {
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

    fn default_branch(
        &self,
        root: &RepoPath,
        branches: &[git::Branch],
        remotes: &[git::Remote],
    ) -> Result<Option<String>> {
        default_branch(self.git.as_ref(), root, branches, remotes)
    }

    /// 버전 기록 한 쪽(D-088). 첫 쪽(`tips` 없음)은 끝점을 정하고 지금 위치·기본 브랜치·안
    /// 올린 버전을 함께 준다. 다음 쪽은 첫 쪽의 `tips`를 되돌려 받아 같은 모양으로 이어 읽는다.
    pub fn log(&self, repo: &Path, query: LogQuery) -> Result<LogPage> {
        use crate::history::{MAX_TIPS_IN, pick_tips};
        let root = self.opened_repo(repo)?;
        let limit = query.limit.min(MAX_LOG_PAGE);
        let branches = self.git.branches(&root)?;
        let mut page = LogPage {
            commits: Vec::new(),
            has_more: false,
            labels: Vec::new(),
            tips: Vec::new(),
            tips_cut: 0,
            head: None,
            current_branch: None,
            default_branch: None,
            default_tip: None,
            unpushed: Vec::new(),
        };
        let tips: Vec<git::Oid> = match &query.tips {
            Some(tips) => {
                if tips.len() > MAX_TIPS_IN {
                    return Err(CoreError::InvalidInput("끝점이 너무 많아요"));
                }
                tips.iter()
                    .map(|t| {
                        git::Oid::parse(t).ok_or(CoreError::InvalidInput("끝점이 해시가 아니에요"))
                    })
                    .collect::<Result<_>>()?
            }
            None => {
                let head = self.git.resolve_ref(&root, "HEAD")?;
                let remotes = self.git.remotes(&root)?;
                page.default_branch = self.default_branch(&root, &branches, &remotes)?;
                page.default_tip = page.default_branch.as_deref().and_then(|name| {
                    branches
                        .iter()
                        .find(|b| b.kind == git::BranchKind::Local && b.name == name)
                        .map(|b| b.oid.clone())
                });
                page.current_branch = self.git.current_branch(&root)?;
                page.head = head.as_ref().map(ToString::to_string);
                page.unpushed = self.unpushed(&root, head.as_ref(), &branches)?;
                match query.scope {
                    crate::view::LogScope::Current => head.into_iter().collect(),
                    crate::view::LogScope::All => {
                        let (tips, cut) =
                            pick_tips(head.as_ref(), page.default_branch.as_deref(), &branches);
                        page.tips_cut = cut;
                        tips
                    }
                }
            }
        };
        page.tips = tips.iter().map(ToString::to_string).collect();
        // 하나 더 받아서 다음 쪽이 있는지 안다.
        let mut commits = self.git.log_graph(
            &root,
            &git::GraphLogQuery {
                tips,
                exclude: Vec::new(),
                skip: query.skip,
                limit: limit + 1,
            },
        )?;
        page.has_more = commits.len() > limit as usize;
        commits.truncate(limit as usize);
        let here: HashSet<&str> = commits.iter().map(|c| c.oid.as_str()).collect();
        for b in branches.iter().filter(|b| b.kind == git::BranchKind::Local) {
            if here.contains(b.oid.as_str()) {
                page.labels.push(crate::view::RefLabelView {
                    oid: b.oid.clone(),
                    kind: crate::view::RefLabelKind::Branch,
                    name: b.name.clone(),
                });
            }
        }
        if !commits.is_empty() {
            for t in self.git.tags(&root)? {
                if here.contains(t.commit.as_str()) {
                    page.labels.push(crate::view::RefLabelView {
                        oid: t.commit.to_string(),
                        kind: crate::view::RefLabelKind::Tag,
                        name: t.name,
                    });
                }
            }
        }
        page.commits = commits.into_iter().map(CommitRow::from).collect();
        Ok(page)
    }

    /// 버전별 바뀐 파일·줄 수(D-089). 한 번에 100개까지. 2MB·10초 안에 못 받은 버전은 결과에
    /// 없다(화면은 "모름"으로 둔다, D-096).
    pub fn log_stats(
        &self,
        repo: &Path,
        oids: Vec<String>,
    ) -> Result<Vec<crate::view::CommitStatView>> {
        const MAX_STATS: usize = 100;
        let root = self.opened_repo(repo)?;
        if oids.len() > MAX_STATS {
            return Err(CoreError::InvalidInput("한 번에 묻는 버전이 너무 많아요"));
        }
        let oids = oids
            .iter()
            .map(|o| git::Oid::parse(o).ok_or(CoreError::InvalidInput("버전이 해시가 아니에요")))
            .collect::<Result<Vec<_>>>()?;
        Ok(self
            .git
            .commit_stats(&root, &oids)?
            .into_iter()
            .map(|s| crate::view::CommitStatView {
                oid: s.oid.to_string(),
                files: s.files,
                added: s.added,
                removed: s.removed,
            })
            .collect())
    }

    /// 버전 상세(D-105): 전체 메시지(64KB까지), 작성자·시각·부모, 첫 부모와 비교해 바뀐 파일
    /// (500개까지). 해시는 브랜치에서 닿거나 작업 내역에 있는 버전만 받는다. 읽기만 한다.
    pub fn commit_detail(&self, repo: &Path, oid: &str) -> Result<crate::view::CommitDetailView> {
        const MAX_MESSAGE: usize = 64 * 1024;
        let root = self.opened_repo(repo)?;
        let oid = crate::guard::verified_commit(
            self.git.as_ref(),
            &root,
            oid,
            crate::guard::Scope::BranchOrActivity,
        )?;
        let commit = self
            .git
            .log(
                &root,
                &git::LogQuery {
                    skip: 0,
                    limit: 1,
                    rev: Some(oid.to_string()),
                },
            )?
            .into_iter()
            .next()
            .ok_or(CoreError::InvalidInput("없는 버전이에요"))?;
        let mut message = self
            .git
            .commit_messages(&root, std::slice::from_ref(&oid))?
            .into_iter()
            .next()
            .map(|m| m.message)
            .unwrap_or_else(|| commit.summary.clone());
        if message.len() > MAX_MESSAGE {
            let mut cut = MAX_MESSAGE;
            while !message.is_char_boundary(cut) {
                cut -= 1;
            }
            message.truncate(cut);
        }
        let files = self.git.commit_files(&root, &oid)?;
        let shown = u32::try_from(files.files.len()).unwrap_or(u32::MAX);
        Ok(crate::view::CommitDetailView {
            oid: commit.oid,
            short_oid: commit.short_oid,
            message,
            author: commit.author_name,
            time: commit.author_time,
            parents: commit.parents,
            more_files: files.total.saturating_sub(shown),
            files: files.files.into_iter().map(Into::into).collect(),
        })
    }

    /// 덮어쓰며 올리기 확인 창의 값(D-128). 네트워크 없이 로컬 정보만 읽는다. `allow`는 앱 설정(D-127)이고
    /// `Core::force_push_plan`이 넣는다. 다른 사람의 버전은 작성자 이메일이 이 저장소에서 쓰는 이메일과 다른 것이고,
    /// 내 이메일을 모르면 사라지는 버전을 모두 다른 사람의 것으로 센다(더 경고하는 쪽).
    pub(crate) fn force_push_plan(
        &self,
        repo: &Path,
        allow: bool,
    ) -> Result<crate::view::ForcePushPlanView> {
        use crate::view::{DroppedVersion, ForcePushBlock as B, ForcePushPlanView};
        /// 사라질 버전을 읽는 상한과 보여 줄 수.
        const READ: u32 = 200;
        const SHOWN: usize = 5;
        let root = self.opened_repo(repo)?;
        let git = self.git.as_ref();
        let mut view = ForcePushPlanView {
            allowed: allow,
            blocked: None,
            branch: None,
            remote: None,
            remote_branch: None,
            expect_remote: None,
            expect_head: None,
            ahead: 0,
            dropped: 0,
            dropped_by_others: 0,
            sample: Vec::new(),
            rewrote_published: false,
            needs_name_confirm: false,
        };
        let blocked = |mut v: ForcePushPlanView, b: B| {
            v.blocked = Some(b);
            Ok(v)
        };
        if !allow {
            return blocked(view, B::Disabled);
        }
        let Some(branch) = git.current_branch(&root)? else {
            return blocked(view, B::Detached);
        };
        view.branch = Some(branch.clone());
        if crate::actions::switch::refuse_during_operation(git, &root).is_err()
            || git.index_entries(&root)?.iter().any(|e| e.stage > 0)
        {
            return blocked(view, B::OtherOperation);
        }
        let Some(up) = crate::actions::remote::upstream(git, &root)? else {
            return blocked(view, B::NoUpstream);
        };
        let (remote, remote_branch) = up.push_target(git)?;
        view.remote = Some(remote.clone());
        view.remote_branch = Some(remote_branch.clone());
        view.needs_name_confirm = crate::guard::protected_branch(git, &root, None, &branch)?
            || crate::guard::protected_branch(git, &root, Some(&remote), &remote_branch)?;
        let (Some(head), Some(tip)) = (
            git.resolve_ref(&root, "HEAD")?,
            git.resolve_ref(&root, &up.rev())?,
        ) else {
            return blocked(view, B::NoUpstream);
        };
        view.expect_head = Some(head.to_string());
        view.expect_remote = Some(tip.to_string());
        view.rewrote_published = crate::rewrote::marked(git, &root, &self.key)?;
        let (ahead, dropped) = git.count_between(&root, head.as_str(), tip.as_str())?;
        view.ahead = ahead;
        view.dropped = dropped;
        if dropped == 0 {
            return blocked(view, B::NothingToOverwrite);
        }
        let gone = git.log(
            &root,
            &git::LogQuery {
                skip: 0,
                limit: READ,
                rev: Some(format!("{head}..{tip}")),
            },
        )?;
        // 저장소 설정이 전역보다 이긴다(git과 같은 순서).
        let mine = git
            .local_config(&root)?
            .into_iter()
            .rev()
            .find(|(k, _)| k == "user.email")
            .map(|(_, v)| v)
            .or(git.identity()?.email);
        let by_others = |c: &&git::Commit| match &mine {
            Some(me) => !c.author_email.eq_ignore_ascii_case(me),
            None => true,
        };
        let counted = u32::try_from(gone.len()).unwrap_or(u32::MAX);
        // 상한 밖의 버전은 다른 사람의 것으로 센다.
        view.dropped_by_others = u32::try_from(gone.iter().filter(by_others).count())
            .unwrap_or(u32::MAX)
            .saturating_add(dropped.saturating_sub(counted));
        view.sample = gone
            .iter()
            .take(SHOWN)
            .map(|c| DroppedVersion {
                short_oid: c.short_oid.clone(),
                summary: c.summary.clone(),
                author: c.author_name.clone(),
            })
            .collect();
        Ok(view)
    }

    /// GitHub에 올리기 대화상자의 처음 값(D-132). 네트워크 없이 읽는다(GitHub 상태는 키체인을 건드리지 않음).
    pub fn publish_draft(&self, repo: &Path) -> Result<crate::view::PublishDraftView> {
        let root = self.opened_repo(repo)?;
        let git = self.git.as_ref();
        let folder = root
            .as_path()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let versions = git
            .log(
                &root,
                &git::LogQuery {
                    skip: 0,
                    limit: 10_000,
                    rev: None,
                },
            )?
            .len();
        let status = self.github.status();
        Ok(crate::view::PublishDraftView {
            suggested_name: crate::actions::publish::suggest_name(&folder),
            branch: git.current_branch(&root)?,
            versions: u32::try_from(versions).unwrap_or(u32::MAX),
            remotes: u32::try_from(git.remotes(&root)?.len()).unwrap_or(u32::MAX),
            configured: status.configured,
            connected: status.connected,
            login: status.login,
        })
    }

    /// 원격 브랜치 지우기·이름 바꾸기 확인 창의 값(D-130). 네트워크 없이 로컬 추적 ref로 계산한다.
    /// 원격은 목록에 있어야 하고 추적 ref는 목록에서 정확히 같은 이름만 찾는다(계획 010 R14).
    /// 브랜치 바꾸기 대화상자의 값(D-195). ref·index·작업 폴더를 바꾸지 않는다(스냅샷 객체만 만든다).
    pub fn switch_plan(
        &self,
        repo: &Path,
        branch: &str,
        remote: Option<&str>,
    ) -> Result<crate::view::SwitchPlanView> {
        use crate::actions::switch::{CarryCheck, carry_check};
        let root = self.opened_repo(repo)?;
        let git = self.git.as_ref();
        crate::actions::names::validate_ref_like(branch)?;
        if let Some(r) = remote {
            crate::actions::names::validate_ref_like(r)?;
        }
        let mut view = crate::view::SwitchPlanView {
            dirty_files: 0,
            can_carry: true,
            reason: None,
            blocked: Vec::new(),
            blocked_total: 0,
        };
        let entries = git.status(&root)?.entries;
        // 버전이 없거나 특정 버전을 보는 중이면 갈아타기가 맡기지 않으므로 묻지 않는다.
        let (Some(current), Some(head)) =
            (git.current_branch(&root)?, git.resolve_ref(&root, "HEAD")?)
        else {
            return Ok(view);
        };
        if entries.is_empty() || (remote.is_none() && current == branch) {
            return Ok(view);
        }
        view.dirty_files = u32::try_from(entries.len()).unwrap_or(u32::MAX);
        if let CarryCheck::Blocked {
            reason,
            paths,
            total,
        } = carry_check(git, &root, branch, remote, &head, &entries)?
        {
            view.can_carry = false;
            view.reason = Some(reason);
            view.blocked = paths;
            view.blocked_total = total;
        }
        Ok(view)
    }

    pub fn remote_branch_plan(
        &self,
        repo: &Path,
        remote: &str,
        branch: &str,
    ) -> Result<crate::view::RemoteBranchPlanView> {
        use crate::view::{RemoteBranchBlock as B, RemoteBranchPlanView};
        /// 이 브랜치에만 있는 버전을 세는 상한.
        const READ: u32 = 200;
        let root = self.opened_repo(repo)?;
        let git = self.git.as_ref();
        let mut view = RemoteBranchPlanView {
            remote: remote.to_string(),
            branch: branch.to_string(),
            tip: None,
            only_here: 0,
            by_others: 0,
            blocked: None,
            is_upstream_of_current: false,
            needs_name_confirm: false,
        };
        if crate::actions::remote_branch::checked_remote(git, &root, remote).is_err() {
            view.blocked = Some(B::NoRemote);
            return Ok(view);
        }
        crate::actions::names::validate_ref_like(branch)?;
        let Some(tip) = crate::actions::remote_branch::tracking_tip(git, &root, remote, branch)?
        else {
            view.blocked = Some(B::NoTrackingRef);
            return Ok(view);
        };
        view.tip = Some(tip.to_string());
        view.is_upstream_of_current = git.status(&root)?.branch.upstream.as_deref()
            == Some(format!("{remote}/{branch}").as_str());
        // 로컬에 기록된 원격 HEAD는 서버 기본 브랜치로 보고 막는다. 이름만 기본 같은 것은 이름 입력으로(D-183).
        if branch == "HEAD" || git.remote_head(&root, remote)?.as_deref() == Some(branch) {
            view.blocked = Some(B::Protected);
        } else if crate::guard::protected_branch(git, &root, Some(remote), branch)? {
            view.needs_name_confirm = true;
        }
        // 이 브랜치에만 있는 버전: 다른 원격 추적 브랜치와 로컬 브랜치에서 닿지 않는 것.
        let own = format!("refs/remotes/{remote}/{branch}");
        let exclude: Vec<git::Oid> = git
            .list_refs(&root, "refs/remotes/")?
            .into_iter()
            .chain(git.list_refs(&root, "refs/heads/")?)
            .filter(|r| r.name != own && !r.name.ends_with("/HEAD"))
            .filter_map(|r| git::Oid::parse(&r.oid))
            .collect();
        let only = git.log_graph(
            &root,
            &git::GraphLogQuery {
                tips: vec![tip],
                exclude,
                skip: 0,
                limit: READ,
            },
        )?;
        let mine = git
            .local_config(&root)?
            .into_iter()
            .rev()
            .find(|(k, _)| k == "user.email")
            .map(|(_, v)| v)
            .or(git.identity()?.email);
        view.only_here = u32::try_from(only.len()).unwrap_or(u32::MAX);
        view.by_others = u32::try_from(
            only.iter()
                .filter(|c| match &mine {
                    Some(me) => !c.author_email.eq_ignore_ascii_case(me),
                    None => true,
                })
                .count(),
        )
        .unwrap_or(u32::MAX);
        Ok(view)
    }

    /// 기록 정리 카드 지도의 처음 모습(D-100, D-106). 고른 버전이 정리할 수 있는 구간이 아니면
    /// `blocked`로 까닭을 알린다. 실행할 때 core가 같은 검사를 다시 한다. 읽기만 한다.
    /// `allow_published_rewrite`는 앱 설정(D-127)이고 `Core::rewrite_plan`이 넣는다.
    pub(crate) fn rewrite_plan(
        &self,
        repo: &Path,
        oids: Vec<String>,
        allow_published_rewrite: bool,
    ) -> Result<crate::view::RewritePlanView> {
        use crate::view::{RewriteCardView, RewritePlanView};
        let root = self.opened_repo(repo)?;
        let rule = crate::guard::published_rule(self.git.as_ref(), &root, allow_published_rewrite)?;
        let check = crate::guard::rewrite_chain(self.git.as_ref(), &root, &oids, rule)?;
        let Some(chain) = check.chain else {
            return Ok(RewritePlanView {
                cards: Vec::new(),
                blocked: check.blocked,
                stale_labels: 0,
            });
        };
        let oids: Vec<git::Oid> = chain
            .selected
            .iter()
            .filter_map(|c| git::Oid::parse(&c.oid))
            .collect();
        let messages = self.git.commit_messages(&root, &oids)?;
        let cards = chain
            .selected
            .iter()
            .map(|c| RewriteCardView {
                oid: c.oid.clone(),
                short_oid: c.short_oid.clone(),
                summary: c.summary.clone(),
                message: messages
                    .iter()
                    .find(|m| m.oid.as_str() == c.oid)
                    .map(|m| m.message.clone())
                    .unwrap_or_else(|| c.summary.clone()),
                author: c.author_name.clone(),
                time: c.author_time,
            })
            .collect();
        // 다시 놓이는 버전(고른 것과 그 위)을 가리키는 다른 브랜치·태그는 옛 버전에 남는다.
        let moved: HashSet<&str> = chain
            .selected
            .iter()
            .chain(chain.above.iter())
            .map(|c| c.oid.as_str())
            .collect();
        let branches = self
            .git
            .branches(&root)?
            .into_iter()
            .filter(|b| b.kind == git::BranchKind::Local && !b.is_head)
            .filter(|b| moved.contains(b.oid.as_str()))
            .count();
        let tags = self
            .git
            .tags(&root)?
            .into_iter()
            .filter(|t| moved.contains(t.commit.as_str()))
            .count();
        Ok(RewritePlanView {
            cards,
            blocked: check.blocked,
            stale_labels: u32::try_from(branches + tags).unwrap_or(u32::MAX),
        })
    }

    /// 지금 브랜치에서 upstream에 아직 없는 버전(최대 500개). upstream이 없으면 빈 목록.
    fn unpushed(
        &self,
        root: &RepoPath,
        head: Option<&git::Oid>,
        branches: &[git::Branch],
    ) -> Result<Vec<String>> {
        let Some(head) = head else {
            return Ok(Vec::new());
        };
        let upstream = branches
            .iter()
            .find(|b| b.is_head)
            .and_then(|b| b.upstream.as_deref())
            .and_then(|u| {
                branches
                    .iter()
                    .find(|b| matches!(b.kind, git::BranchKind::Remote { .. }) && b.name == u)
            })
            .and_then(|b| git::Oid::parse(&b.oid));
        let Some(upstream) = upstream else {
            return Ok(Vec::new());
        };
        Ok(self
            .git
            .log_graph(
                root,
                &git::GraphLogQuery {
                    tips: vec![head.clone()],
                    exclude: vec![upstream],
                    skip: 0,
                    limit: MAX_LOG_PAGE,
                },
            )?
            .into_iter()
            .map(|c| c.oid)
            .collect())
    }
}

/// 파일 앞부분(비밀 검사 상한까지). 링크를 따라가지 않는다(확인과 열기 사이에 링크로 바뀌어도 읽지 않음).
fn read_head(path: &Path) -> Vec<u8> {
    use std::io::Read;
    let mut buf = Vec::new();
    if let Ok(file) = platform::open_regular_no_follow(path) {
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

/// 마지막 묶음 뒤 접힌 줄을 셀 만한 비교인지(D-219): 묶음이 있고 잘리지 않은 글자 비교.
fn wants_line_count(view: &FileDiffView) -> bool {
    !view.binary && !view.truncated && view.link.is_none() && !view.hunks.is_empty()
}

/// 작업 폴더 파일의 줄 수(D-219). 링크·일반 파일이 아님·없음·1MB 넘음이면 `None`.
fn work_tree_line_count(root: &Path, path: &str) -> Option<u32> {
    use std::io::Read;
    let file = platform::open_regular_no_follow(&root.join(path)).ok()?;
    let mut bytes = Vec::new();
    file.take(crate::view::LINE_COUNT_LIMIT + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > crate::view::LINE_COUNT_LIMIT {
        return None;
    }
    crate::view::line_count(&bytes)
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

/// 기본 브랜치: 지금 브랜치 upstream의 원격(없으면 origin, 그다음 첫 원격)의 HEAD,
/// 그것도 없으면 로컬 main, master 순(D-048). 브랜치 지우기도 같은 판정을 쓴다.
pub(crate) fn default_branch(
    git: &dyn GitBackend,
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
        && let Some(head) = git.remote_head(root, &remote.name)?
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

/// 이미지 비교에서 한쪽 상한(D-206).
const IMAGE_MAX_BYTES: u64 = 3 * 1024 * 1024;

/// 이미지 비교 한쪽을 읽은 결과.
enum ImageRead {
    Bytes(Vec<u8>),
    TooLarge,
}

/// 작업 폴더의 일반 파일을 읽는다. 중간 폴더가 링크(정션 포함)거나 마지막이 링크·일반 파일이 아니면 읽지 않는다.
fn worktree_image(root: &Path, rel: &str) -> Option<ImageRead> {
    use std::io::Read as _;
    let mut at = root.to_path_buf();
    let mut parts = rel.split('/').peekable();
    while let Some(part) = parts.next() {
        at.push(part);
        if parts.peek().is_some() {
            // `FileType::is_dir`은 링크(정션 포함)이면 거짓이다.
            let meta = std::fs::symlink_metadata(&at).ok()?;
            if !meta.file_type().is_dir() {
                return None;
            }
        }
    }
    let file = platform::open_regular_no_follow(&at).ok()?;
    let size = file.metadata().ok()?.len();
    if size > IMAGE_MAX_BYTES {
        return Some(ImageRead::TooLarge);
    }
    let mut bytes = Vec::new();
    // 연 뒤에 커졌어도 상한 + 1바이트까지만 읽는다.
    file.take(IMAGE_MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > IMAGE_MAX_BYTES {
        return Some(ImageRead::TooLarge);
    }
    Some(ImageRead::Bytes(bytes))
}

#[cfg(test)]
mod tests {
    #[test]
    fn github_label_comes_from_the_host_part_only() {
        let github = |url: &str| hosting::GitHub::detect(url).is_some();
        assert!(github("https://github.com/a/b.git"));
        assert!(github("ssh://git@github.com:22/a/b"));
        assert!(github("git@github.com:a/b.git"));
        assert!(!github("https://evil.example/github.com/x"));
        assert!(!github("/local/path"));
        // 사용자 정보가 든 https 주소는 GitHub 이름표를 붙이지 않는다(반복 4에서 바뀜).
        assert!(!github("https://user:pw@GitHub.com/a/b"));
    }
}
