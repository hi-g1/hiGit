//! 원격 태그 지우기(remote-tag-delete, D-182). 원격에서 지운 태그는 다른 사람·GitHub 릴리스가 이미 쓰고
//! 있을 수 있어 앱이 되돌릴 수 없다(`Irreversible`, 확인 토큰). 대상은 로컬에 있는 태그만이고, 원격 태그가
//! 그 값일 때만 지운다(값 있는 lease). 로컬 태그는 그대로 두어 다시 올릴 수 있다.

use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

use super::remote::{Net, pick_remote};
use super::repo_of;

pub struct RemoteTagDeleteAction {
    meta: ActionMeta,
}

impl RemoteTagDeleteAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "remote-tag-delete",
                glossary_id: "remote-tag-delete",
                risk: Risk::Danger,
                undo: crate::undo::UndoKind::Irreversible,
            },
        }
    }
}

impl Default for RemoteTagDeleteAction {
    fn default() -> Self {
        Self::new()
    }
}

/// 로컬 태그 ref의 값 그대로(주석 태그는 벗기지 않은 태그 객체). 원격 lease와 같은 값이다.
pub(crate) fn local_tag_value(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    name: &str,
) -> Result<Option<git::Oid>> {
    let full = format!("refs/tags/{name}");
    Ok(git
        .list_refs(repo, &full)?
        .into_iter()
        .find(|r| r.name == full)
        .and_then(|r| git::Oid::parse(&r.oid)))
}

impl Action for RemoteTagDeleteAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::RemoteTagDelete(input) = input else {
            return Err(CoreError::InputMismatch("remote-tag-delete"));
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let name = super::tag::tag_name(git, &input.name)?;
        let expect = git::Oid::parse(&input.expect).ok_or(CoreError::InvalidInput("버전"))?;
        // 확인 창이 본 로컬 태그 값(가벼운 태그는 버전, 주석 태그는 태그 객체)과 같아야 한다.
        if local_tag_value(git, repo, &name)?.as_ref() != Some(&expect) {
            return Err(CoreError::InvalidInput("그 태그가 바뀌었어요"));
        }
        let remote = pick_remote(git, repo)?;
        let net = Net {
            git,
            repo,
            github: ctx.github,
            background: false,
            use_helper: ctx.settings.git_helper,
        };
        match net.push_delete_tag(&remote, &name, &expect) {
            Ok(_) => Ok(Outcome::RemoteTagDeleted { name }),
            // lease가 어긋나면 서버에 다시 물어 없음·다른 값을 나눈다.
            Err(CoreError::Git(e)) if e.kind() == git::FailureKind::LeaseStale => {
                match net.server_ref(&remote, &format!("refs/tags/{name}"))? {
                    None => Err(CoreError::Git(git::GitError::Refused {
                        kind: git::FailureKind::RemoteRefMissing,
                        detail: e.detail().to_owned(),
                    })),
                    Some(_) => Err(CoreError::RemoteTagDiffers),
                }
            }
            Err(e) => Err(e),
        }
    }
}
