//! 새 검토 요청(pr-create, D-113). 올리기는 하지 않는다: 화면이 먼저 `push`를 부르고 끝나면
//! 이 액션을 부른다. 올라가 있지 않으면 `NotPushed`로 거절한다.

use crate::github::pr::pr_target;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, GithubFailure, Outcome, RepoState,
    Result, Risk,
};

use super::{needs_repo, repo_of};

pub struct PrCreateAction {
    meta: ActionMeta,
}

impl PrCreateAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "pr-create",
                glossary_id: "pr",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for PrCreateAction {
    fn default() -> Self {
        Self::new()
    }
}

fn github(kind: GithubFailure) -> CoreError {
    CoreError::Github { kind, detail: None }
}

impl Action for PrCreateAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::PrCreate(i) = input else {
            return Err(CoreError::InputMismatch("pr-create"));
        };
        let git = ctx.git;
        let repo = repo_of(ctx)?;
        crate::guard::require_branch(git, repo)?;
        crate::actions::switch::refuse_during_operation(git, repo)?;
        let title = i.title.trim();
        if title.is_empty()
            || title.len() > hosting::PR_TITLE_MAX_BYTES
            || title.chars().any(char::is_control)
        {
            return Err(CoreError::InvalidInput("검토 요청 제목"));
        }
        if i.body.len() > hosting::PR_BODY_MAX_BYTES || i.body.contains('\0') {
            return Err(CoreError::InvalidInput("검토 요청 본문"));
        }
        if !git.check_branch_name(&i.base)? {
            return Err(CoreError::InvalidInput("대상 브랜치"));
        }
        let target = pr_target(git, repo)?.ok_or(CoreError::NoRemote)?;
        let github_repo = target
            .repo
            .clone()
            .ok_or_else(|| github(GithubFailure::NotGithubRemote))?;
        if !target.remote_branches.contains(&i.base) {
            return Err(CoreError::InvalidInput("대상 브랜치"));
        }
        // head는 올라가 있는 원격 브랜치 이름이다. 안 올린 버전이 있으면 먼저 올려야 한다.
        let (Some(head), Some(tracking)) = (target.head.clone(), target.tracking.clone()) else {
            return Err(github(GithubFailure::NotPushed));
        };
        let (unpushed, _) = git.count_between(repo, "HEAD", &format!("refs/remotes/{tracking}"))?;
        if unpushed > 0 {
            return Err(github(GithubFailure::NotPushed));
        }
        if head == i.base || target.default_branch.as_deref() == Some(head.as_str()) {
            return Err(github(GithubFailure::PrSameBranch));
        }
        let base_ref = format!("refs/remotes/{}/{}", target.remote, i.base);
        let (ahead, _) = git.count_between(repo, &format!("refs/remotes/{tracking}"), &base_ref)?;
        if ahead == 0 {
            return Err(github(GithubFailure::PrNoDifference));
        }
        // 포크 경로면 포크의 원래 저장소가 기준 저장소인지 보내기 전에 확인한다(D-181).
        if let Some(fork) = &target.fork_repo {
            let info = ctx
                .github
                .call(|provider, token| provider.repo_info(token, fork))?;
            let parent_ok = info.parent.as_ref().is_some_and(|p| {
                p.owner.eq_ignore_ascii_case(&github_repo.owner)
                    && p.name.eq_ignore_ascii_case(&github_repo.name)
            });
            if !parent_ok {
                return Err(github(GithubFailure::ForkParentMismatch));
            }
        }
        let new = hosting::NewPr {
            title: title.to_owned(),
            body: i.body.clone(),
            head,
            head_owner: target.head_owner().map(str::to_owned),
            base: i.base.clone(),
            draft: i.draft,
        };
        let created = ctx
            .github
            .call(|provider, token| provider.create_pr(token, &github_repo, &new))?;
        Ok(Outcome::PrCreated {
            number: u32::try_from(created.number).unwrap_or(u32::MAX),
        })
    }
}
