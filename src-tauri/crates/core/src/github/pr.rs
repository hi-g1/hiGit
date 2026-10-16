//! 지금 브랜치의 검토 요청 대상(D-113)과 hosting 결과 → 화면 타입.
//!
//! 대상 원격은 지금 브랜치 upstream의 원격(없으면 원격이 하나면 그것, 여럿이면 origin)이고,
//! 그 원격의 올리기 주소가 github.com일 때만 GitHub 저장소다. head는 로컬 브랜치 이름이 아니라
//! 올라가 있는 원격 브랜치 이름이다.

use git::{BranchKind, GitBackend, RepoPath};

use crate::Result;
use crate::view::{
    ChecksView, MergeableView, PrDetailView, PrRowView, PrStateFilterView, PrStateView,
};

/// 지금 브랜치의 검토 요청 대상.
pub(crate) struct PrTarget {
    /// 원격 이름.
    pub remote: String,
    /// github.com이면 그 저장소.
    pub repo: Option<hosting::RemoteRepo>,
    /// 지금 로컬 브랜치. 특정 버전을 보는 중이면 `None`.
    pub branch: Option<String>,
    /// upstream의 원격 브랜치 이름(원격 접두사 뺀 것). 올린 적 없거나 사라졌으면 `None`.
    pub head: Option<String>,
    /// upstream의 원격 추적 이름(`origin/x`).
    pub tracking: Option<String>,
    pub default_branch: Option<String>,
    /// 그 원격의 원격 브랜치 이름들(접두사 뺀 것, `HEAD` 제외).
    pub remote_branches: Vec<String>,
}

/// 대상 원격을 정한다. 원격이 없거나 고를 수 없으면 `None`.
pub(crate) fn pr_target(git: &dyn GitBackend, root: &RepoPath) -> Result<Option<PrTarget>> {
    let branches = git.branches(root)?;
    let remotes = git.remotes(root)?;
    let current = branches
        .iter()
        .find(|b| b.is_head && b.kind == BranchKind::Local);
    let upstream = current.filter(|b| !b.gone).and_then(|b| b.upstream.clone());
    let upstream_remote = upstream.as_deref().and_then(|u| {
        remotes
            .iter()
            .filter(|r| u.starts_with(&format!("{}/", r.name)))
            .max_by_key(|r| r.name.len())
    });
    let remote = match (upstream_remote, remotes.as_slice()) {
        (Some(r), _) => r.name.clone(),
        (None, []) => return Ok(None),
        (None, [only]) => only.name.clone(),
        (None, many) => match many.iter().find(|r| r.name == "origin") {
            Some(r) => r.name.clone(),
            None => return Ok(None),
        },
    };
    crate::actions::names::validate_ref_like(&remote)?;
    let urls = git.remote_urls(root, &remote)?;
    // 올리기 주소가 모두 같은 github.com 저장소일 때만.
    let mut found: Option<hosting::RemoteRepo> = None;
    let mut same = !urls.push.is_empty();
    for url in &urls.push {
        match (hosting::GitHub::detect(url), &found) {
            (Some(repo), None) => found = Some(repo),
            (Some(repo), Some(prev)) if repo_eq(&repo, prev) => {}
            _ => same = false,
        }
    }
    let repo = found.filter(|_| same);
    let prefix = format!("{remote}/");
    let head = upstream
        .as_deref()
        .filter(|_| upstream_remote.is_some())
        .and_then(|u| u.strip_prefix(&prefix))
        .map(str::to_owned);
    let default_branch = crate::query::default_branch(git, root, &branches, &remotes)?;
    let remote_branches = branches
        .iter()
        .filter_map(|b| match &b.kind {
            BranchKind::Remote { remote: r, branch } if *r == remote && branch != "HEAD" => {
                Some(branch.clone())
            }
            _ => None,
        })
        .collect();
    Ok(Some(PrTarget {
        remote,
        repo,
        branch: current.map(|b| b.name.clone()),
        tracking: head.as_ref().map(|h| format!("{prefix}{h}")),
        head,
        default_branch,
        remote_branches,
    }))
}

fn repo_eq(a: &hosting::RemoteRepo, b: &hosting::RemoteRepo) -> bool {
    a.host == b.host
        && a.owner.eq_ignore_ascii_case(&b.owner)
        && a.name.eq_ignore_ascii_case(&b.name)
}

pub(crate) fn state_filter(state: PrStateFilterView) -> hosting::PrStateFilter {
    match state {
        PrStateFilterView::Open => hosting::PrStateFilter::Open,
        PrStateFilterView::Closed => hosting::PrStateFilter::Closed,
        PrStateFilterView::All => hosting::PrStateFilter::All,
    }
}

/// `current_head`: 이 저장소의 지금 브랜치가 올라가 있는 원격 브랜치 이름.
pub(crate) fn row(pr: &hosting::PullRequest, current_head: Option<&str>) -> PrRowView {
    PrRowView {
        number: u32::try_from(pr.number).unwrap_or(u32::MAX),
        title: pr.title.clone(),
        state: match pr.state {
            hosting::PrState::Open => PrStateView::Open,
            hosting::PrState::Closed => PrStateView::Closed,
            hosting::PrState::Merged => PrStateView::Merged,
        },
        draft: pr.draft,
        head: pr.head_branch.clone(),
        head_owner: pr.head_owner.clone(),
        base: pr.base_branch.clone(),
        author: pr.author.clone(),
        updated_at: pr.updated_at,
        is_current_branch: pr.head_owner.is_none()
            && !pr.head_deleted
            && current_head.is_some_and(|h| h == pr.head_branch),
    }
}

pub(crate) fn detail(detail: &hosting::PrDetail, current_head: Option<&str>) -> PrDetailView {
    PrDetailView {
        row: row(&detail.pr, current_head),
        mergeable: match detail.mergeable {
            hosting::Mergeable::Clean => MergeableView::Clean,
            hosting::Mergeable::Conflicts => MergeableView::Conflicts,
            hosting::Mergeable::Behind => MergeableView::Behind,
            hosting::Mergeable::Blocked => MergeableView::Blocked,
            hosting::Mergeable::Unknown => MergeableView::Unknown,
        },
        checks: detail.checks.map(|c| ChecksView {
            success: c.success,
            failed: c.failed,
            pending: c.pending,
            partial: c.partial,
        }),
        reviews: detail.reviews.map(|r| crate::view::ReviewsView {
            approved: r.approved,
            changes_requested: r.changes_requested,
        }),
        comments: detail.comments,
        issue_comments: detail.issue_comments,
        head_oid: detail.head_sha.clone(),
    }
}

/// 저장소 정보 → 화면 값(D-177, D-179). 방법 칸이 하나도 없으면 `[merge]`(서버가 마지막에 정함).
pub(crate) fn repo_view(
    info: &hosting::RepoInfo,
    viewer: Option<String>,
) -> crate::view::PrRepoView {
    use crate::view::MergeMethodView as M;
    let mut methods = Vec::new();
    if info.allow_merge_commit.is_none() && info.allow_squash_merge.is_none() {
        methods.push(M::Merge);
    } else {
        if info.allow_merge_commit == Some(true) {
            methods.push(M::Merge);
        }
        if info.allow_squash_merge == Some(true) {
            methods.push(M::Squash);
        }
    }
    crate::view::PrRepoView {
        can_push: info.can_push,
        can_merge: info.can_push,
        rebase_only: methods.is_empty() && info.allow_rebase_merge == Some(true),
        merge_methods: methods,
        archived: info.archived,
        viewer,
    }
}

/// 지금 브랜치가 올리는 GitHub 저장소. GitHub가 아니면 `NotGithubRemote`, 원격이 없으면 `NoRemote`.
pub(crate) fn github_repo(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
) -> crate::Result<hosting::RemoteRepo> {
    pr_target(git, repo)?
        .ok_or(crate::CoreError::NoRemote)?
        .repo
        .ok_or(crate::CoreError::Github {
            kind: crate::GithubFailure::NotGithubRemote,
            detail: None,
        })
}

/// 마지막 두 쪽에서 최근 [`hosting::COMMENTS_PAGE`]개를 고른다(D-178). `total`은 대화 댓글 수.
pub(crate) fn comment_pages(total: u32) -> (u32, Option<u32>) {
    let per = hosting::COMMENTS_PAGE;
    let last = total.div_ceil(per).max(1);
    // 마지막 쪽이 덜 찼으면 그 앞 쪽도 읽어 20개를 채운다.
    let prev = (last > 1 && !total.is_multiple_of(per)).then(|| last - 1);
    (last, prev)
}

pub(crate) fn comments_view(
    mut older: Vec<hosting::Comment>,
    last: Vec<hosting::Comment>,
    total: u32,
    viewer: Option<&str>,
) -> crate::view::PrCommentsView {
    let per = hosting::COMMENTS_PAGE as usize;
    older.extend(last);
    let skip = older.len().saturating_sub(per);
    let items: Vec<_> = older
        .into_iter()
        .skip(skip)
        .map(|c| crate::view::PrCommentView {
            mine: matches!((&c.author, viewer), (Some(a), Some(v)) if a.eq_ignore_ascii_case(v)),
            author: c.author,
            created_at: c.created_at,
            body: c.body,
        })
        .collect();
    crate::view::PrCommentsView {
        partial: (total as usize) > items.len(),
        total,
        items,
    }
}

/// 제목 제안: 앞선 버전이 하나면 그 요약, 아니면 브랜치 이름의 `-`·`_`·`/`를 공백으로.
pub(crate) fn suggest_title(branch: &str, ahead: u32, only_summary: Option<&str>) -> String {
    if ahead == 1
        && let Some(summary) = only_summary.map(str::trim).filter(|s| !s.is_empty())
    {
        return summary.to_owned();
    }
    branch
        .chars()
        .map(|c| if matches!(c, '-' | '_' | '/') { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_suggestions() {
        assert_eq!(
            suggest_title("feature/login-page", 2, None),
            "feature login page"
        );
        assert_eq!(suggest_title("fix_bug", 0, Some("x")), "fix bug");
        assert_eq!(
            suggest_title("feature/x", 1, Some(" 로그인 화면 추가 ")),
            "로그인 화면 추가"
        );
        assert_eq!(suggest_title("a--b", 1, Some("  ")), "a b");
    }
}
