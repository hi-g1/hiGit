//! 포크해서 요청하기의 첫 단계(fork, D-181). 올릴 권한이 없는 GitHub 저장소를 내 계정에 포크하고 원격
//! `fork`로 이은 뒤 지금 브랜치를 그리로 올린다. 이후 올리기·동기화는 upstream이 `fork`라 기존 액션이 간다.
//! 원격 이름은 상수이고 주소는 검사한 로그인·이름으로 core가 만든다.

use git::GitOp;

use crate::github::pr::FORK_REMOTE;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, GithubFailure, Outcome, RepoState,
    Result, Risk,
};

use super::remote::Net;
use super::{needs_repo, repo_of};

/// 포크가 준비될 때까지 묻는 최대 횟수(간격은 서비스가 정함, 기본 2초).
const FORK_READY_TRIES: u32 = 15;

pub struct ForkAction {
    meta: ActionMeta,
}

impl ForkAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "fork",
                glossary_id: "fork",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for ForkAction {
    fn default() -> Self {
        Self::new()
    }
}

fn failure(kind: GithubFailure, detail: Option<&str>) -> CoreError {
    CoreError::Github {
        kind,
        detail: detail.map(str::to_owned),
    }
}

/// 원격 `fork`가 이미 있을 때: 받기·올리기 주소가 모두 `url` 하나면 다시 쓴다(D-188). 받은 저장소가
/// `pushurl`을 심어 두었을 수 있어 올리기 주소까지 본다.
pub(crate) fn reusable_fork_remote(urls: &git::RemoteUrls, url: &str) -> bool {
    !urls.fetch.is_empty()
        && !urls.push.is_empty()
        && urls.fetch.iter().chain(&urls.push).all(|u| u == url)
}

impl Action for ForkAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Fork = input else {
            return Err(CoreError::InputMismatch("fork"));
        };
        let git = ctx.git;
        let repo = repo_of(ctx)?;
        crate::guard::require_branch(git, repo)?;
        crate::actions::switch::refuse_during_operation(git, repo)?;
        let status = ctx.github.status();
        let login = status
            .login
            .filter(|_| status.connected)
            .ok_or_else(|| failure(GithubFailure::NotConnected, None))?;
        let target = crate::github::pr::pr_target(git, repo)?.ok_or(CoreError::NoRemote)?;
        let base = target
            .repo
            .clone()
            .ok_or_else(|| failure(GithubFailure::NotGithubRemote, None))?;
        let branch = target
            .branch
            .clone()
            .ok_or(CoreError::InvalidInput("브랜치"))?;
        let fork = ctx
            .github
            .call(|provider, token| provider.create_fork(token, &base))?;
        // 내 계정의 포크여야 한다. 주소는 검사한 소유자·이름으로만 만든다.
        if !fork.owner.eq_ignore_ascii_case(&login) {
            return Err(failure(
                GithubFailure::BadResponse,
                Some("unexpected owner"),
            ));
        }
        let url = format!("https://github.com/{}/{}.git", fork.owner, fork.name);
        let parsed = hosting::GitHub::detect(&url);
        if parsed.as_ref().map(|r| (r.owner.as_str(), r.name.as_str()))
            != Some((fork.owner.as_str(), fork.name.as_str()))
        {
            return Err(failure(GithubFailure::BadResponse, Some("unexpected fork")));
        }
        // 새 포크는 GitHub가 준비하는 데 시간이 걸린다. 매번 취소를 본다.
        let mut ready = false;
        for attempt in 0..FORK_READY_TRIES {
            if git.cancel_requested(repo) {
                return Err(failure(GithubFailure::Cancelled, None));
            }
            if ctx
                .github
                .call(|provider, token| provider.repo_exists(token, &fork))?
            {
                ready = true;
                break;
            }
            if attempt + 1 < FORK_READY_TRIES {
                ctx.github.wait_for_fork();
            }
        }
        if !ready {
            return Err(failure(GithubFailure::ForkNotReady, None));
        }
        if git.remotes(repo)?.iter().any(|r| r.name == FORK_REMOTE) {
            if !reusable_fork_remote(&git.remote_urls(repo, FORK_REMOTE)?, &url) {
                return Err(failure(GithubFailure::ForkRemoteTaken, None));
            }
        } else {
            git.run(
                repo,
                &GitOp::RemoteAdd {
                    name: FORK_REMOTE.into(),
                    url,
                },
            )?;
        }
        let net = Net {
            git,
            repo,
            github: ctx.github,
            background: false,
            use_helper: ctx.settings.git_helper,
        };
        net.push((FORK_REMOTE.to_owned(), branch), true)?;
        Ok(Outcome::Forked {
            owner: fork.owner,
            name: fork.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls(fetch: &[&str], push: &[&str]) -> git::RemoteUrls {
        git::RemoteUrls {
            fetch: fetch.iter().map(|s| s.to_string()).collect(),
            push: push.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn fork_remote_is_reused_only_when_every_url_matches() {
        let url = "https://github.com/mona/hello.git";
        assert!(reusable_fork_remote(&urls(&[url], &[url]), url));
        for (fetch, push) in [
            (&["https://github.com/evil/x.git"][..], &[url][..]),
            (&[url][..], &["https://evil.example/x.git"][..]),
            (&[url][..], &[url, "https://evil.example/x.git"][..]),
            (&[][..], &[][..]),
        ] {
            assert!(
                !reusable_fork_remote(&urls(fetch, push), url),
                "{fetch:?} {push:?}"
            );
        }
    }
}
