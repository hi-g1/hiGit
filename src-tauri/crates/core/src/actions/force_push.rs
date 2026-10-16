//! 덮어쓰며 올리기(`force-push`, D-128). 원격의 옛 기록은 앱이 되돌릴 수 없어 `Irreversible`이고
//! 실행기가 확인 토큰을 요구한다. 지키는 것 넷: 설정 스위치, 확인 토큰(입력 전체), 원격 기본 브랜치
//! 거절(로컬 이름 규칙과 서버가 알려 준 기본 브랜치), 확인 창이 본 값에만 거는 lease.

use git::{FailureKind, GitError, Oid};

use super::remote::{Net, upstream};
use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk, guard,
};

pub struct ForcePushAction {
    meta: ActionMeta,
}

impl ForcePushAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "force-push",
                glossary_id: "force-push",
                risk: Risk::Danger,
                undo: crate::undo::UndoKind::Irreversible,
            },
        }
    }
}

impl Default for ForcePushAction {
    fn default() -> Self {
        Self::new()
    }
}

/// 추적 ref가 확인 창이 본 값과 다르다(그사이 받아 왔다). lease 실패와 같은 안내로.
fn remote_moved() -> CoreError {
    CoreError::Git(GitError::Refused {
        kind: FailureKind::LeaseStale,
        detail: "tracking ref moved".into(),
    })
}

impl Action for ForcePushAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    /// 로컬 HEAD는 그대로다(원격만 바뀐다). 원격의 옛 끝은 아래에서 따로 백업한다.
    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::ForcePush(input) = input else {
            return Err(CoreError::InputMismatch("force-push"));
        };
        if !ctx.settings.allow_published_rewrite {
            return Err(CoreError::ForcePushDisabled);
        }
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let branch = guard::require_branch(git, repo)?;
        refuse_during_operation(git, repo)?;
        super::refuse_conflicts(git, repo)?;
        let up = upstream(git, repo)?.ok_or(CoreError::Git(GitError::Refused {
            kind: FailureKind::NoUpstream,
            detail: "no upstream".into(),
        }))?;
        let (remote, remote_branch) = up.push_target(git)?;
        // 화면이 본 이름과 core가 upstream에서 구한 이름이 같아야 한다(토큰이 이름까지 묶는다).
        if input.remote != remote || input.branch != remote_branch {
            return Err(CoreError::InvalidInput("올릴 곳이 바뀌었어요"));
        }
        // 기본 브랜치로 보이면(이름만 같은 것도) 브랜치 이름을 똑같이 입력해야 한다(D-183).
        let looks_default = guard::protected_branch(git, repo, None, &branch)?
            || guard::protected_branch(git, repo, Some(&remote), &remote_branch)?;
        let head = git
            .resolve_ref(repo, "HEAD")?
            .ok_or(CoreError::InvalidInput("올릴 버전이 없어요"))?;
        if Oid::parse(&input.expect_head).as_ref() != Some(&head) {
            return Err(CoreError::InvalidInput("지금 버전이 바뀌었어요"));
        }
        let expect = Oid::parse(&input.expect_remote)
            .ok_or(CoreError::InvalidInput("원격 버전이 해시가 아니에요"))?;
        if git.resolve_ref(repo, &up.rev())?.as_ref() != Some(&expect) {
            return Err(remote_moved());
        }
        // 원격에 내 기록에 없는 버전이 없으면 덮어쓸 것이 없다(그냥 올리면 된다).
        let (_, dropped) = git.count_between(repo, head.as_str(), expect.as_str())?;
        if dropped == 0 {
            return Err(CoreError::NothingToOverwrite);
        }
        let net = Net {
            git,
            repo,
            github: ctx.github,
            background: false,
            use_helper: ctx.settings.git_helper,
        };
        // 서버가 알려 주는 기본 브랜치도 이름 입력으로만 연다. 읽지 못하면 기본 브랜치로 본다(D-183 ②).
        let server_default = match net.server_head(&remote)? {
            Some(server) => server == remote_branch,
            None => true,
        };
        if looks_default || server_default {
            guard::confirm_name(input.confirm_branch.as_deref(), &remote_branch)?;
        }
        // 원격의 옛 끝을 살려 둔다. 못 남기면 올리지 않는다(D-034 4번).
        crate::undo::backup::backup_oid(git, repo, &expect, &format!("force-push-{branch}"))?;
        net.push_force(&remote, &remote_branch, &head, &expect)?;
        // 원격이 내 기록이 됐으니 "올린 기록을 바꿨다" 표식은 끝났다(D-129).
        crate::rewrote::forget(git, repo, ctx.key, &branch)?;
        Ok(Outcome::ForcePushed { dropped })
    }
}
