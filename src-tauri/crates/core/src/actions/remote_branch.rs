//! 원격 브랜치 지우기·이름 바꾸기(`remote-branch-delete`·`remote-branch-rename`, D-130). 원격에서 지운
//! 브랜치는 앱이 되돌릴 수 없어 `Irreversible`이고 실행기가 확인 토큰을 요구한다. 지울 수 있는 것은 로컬
//! 추적 ref가 있고 사용자가 확인 창에서 본 값(`expect_tip`)인 브랜치뿐이고, 원격 기본 브랜치는 막는다.
//! 끝 버전은 지우기 전에 백업 ref로 남긴다. 로컬 상태를 바꾸지 않아 journal 줄은 남기지 않는다.

use git::{FailureKind, GitBackend, GitError, Oid, RefExpect, RepoPath};

use super::remote::Net;
use super::{names, repo_of};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk, guard,
};

pub struct RemoteBranchAction {
    meta: ActionMeta,
}

impl RemoteBranchAction {
    fn new(id: &'static str) -> Self {
        Self {
            meta: ActionMeta {
                id,
                glossary_id: id,
                risk: Risk::Danger,
                undo: crate::undo::UndoKind::Irreversible,
            },
        }
    }

    pub fn delete() -> Self {
        Self::new("remote-branch-delete")
    }

    pub fn rename() -> Self {
        Self::new("remote-branch-rename")
    }
}

fn remote_moved() -> CoreError {
    CoreError::Git(GitError::Refused {
        kind: FailureKind::LeaseStale,
        detail: "remote branch moved".into(),
    })
}

/// 원격이 목록에 있고 이름이 안전한지.
pub(crate) fn checked_remote(git: &dyn GitBackend, repo: &RepoPath, remote: &str) -> Result<()> {
    names::validate_ref_like(remote)?;
    if !git.remotes(repo)?.iter().any(|r| r.name == remote) {
        return Err(CoreError::NoRemote);
    }
    Ok(())
}

/// 원격 브랜치 이름이 안전한지(`HEAD` 별칭은 다루지 않는다).
fn checked_branch(branch: &str) -> Result<()> {
    names::validate_ref_like(branch)?;
    if branch == "HEAD" {
        return Err(CoreError::InvalidInput("원격 HEAD는 다루지 않아요"));
    }
    Ok(())
}

/// 로컬 추적 ref(`refs/remotes/<원격>/<브랜치>`)의 값. 목록에서 정확히 같은 이름만 본다.
pub(crate) fn tracking_tip(
    git: &dyn GitBackend,
    repo: &RepoPath,
    remote: &str,
    branch: &str,
) -> Result<Option<Oid>> {
    let name = format!("refs/remotes/{remote}/{branch}");
    Ok(git
        .list_refs(repo, &format!("refs/remotes/{remote}/"))?
        .into_iter()
        .find(|r| r.name == name)
        .and_then(|r| Oid::parse(&r.oid)))
}

/// 로컬 규칙(이름·원격 HEAD)과 서버가 알려 준 기본 브랜치로 막는다. 서버 값을 읽지 못하면 막는다.
fn refuse_default(net: &Net<'_>, remote: &str, branch: &str) -> Result<()> {
    if guard::protected_branch(net.git, net.repo, Some(remote), branch)? {
        return Err(CoreError::RemoteBranchProtected);
    }
    match net.server_head(remote)? {
        Some(server) if server != branch => Ok(()),
        _ => Err(CoreError::RemoteBranchProtected),
    }
}

impl Action for RemoteBranchAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    /// 로컬 HEAD는 그대로다. 원격 끝은 아래에서 따로 백업한다.
    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let net = Net {
            git,
            repo,
            github: ctx.github,
            background: false,
            use_helper: ctx.settings.git_helper,
        };
        match (self.meta.id, input) {
            ("remote-branch-delete", ActionInput::RemoteBranchDelete(i)) => {
                checked_remote(git, repo, &i.remote)?;
                checked_branch(&i.branch)?;
                let expect = Oid::parse(&i.expect_tip)
                    .ok_or(CoreError::InvalidInput("버전이 해시가 아니에요"))?;
                let tip = tracking_tip(git, repo, &i.remote, &i.branch)?
                    .ok_or(CoreError::InvalidInput("그 원격 브랜치가 없어요"))?;
                if tip != expect {
                    return Err(remote_moved());
                }
                refuse_default(&net, &i.remote, &i.branch)?;
                crate::undo::backup::backup_oid(
                    git,
                    repo,
                    &expect,
                    &format!("remote-branch-delete-{}-{}", i.remote, i.branch),
                )?;
                match net.push_delete(&i.remote, &i.branch, &expect) {
                    Ok(_) => {}
                    // 이미 없으면 값 있는 lease는 (stale info)로 거절한다. 서버에 다시 물어 없으면 성공(R3).
                    Err(CoreError::Git(e)) if e.kind() == FailureKind::LeaseStale => {
                        if net.server_branch(&i.remote, &i.branch)?.is_some() {
                            return Err(remote_moved());
                        }
                    }
                    Err(e) => return Err(e),
                }
                // 올리기가 추적 ref를 지우지 않았으면(원격 설정에 따라) 본 값일 때만 지운다.
                let name = format!("refs/remotes/{}/{}", i.remote, i.branch);
                if tracking_tip(git, repo, &i.remote, &i.branch)?.as_ref() == Some(&expect) {
                    git.delete_ref(repo, &name, &RefExpect::Is(expect))?;
                }
                Ok(Outcome::RemoteBranchDeleted {
                    remote: i.remote.clone(),
                    branch: i.branch.clone(),
                })
            }
            ("remote-branch-rename", ActionInput::RemoteBranchRename(i)) => {
                checked_remote(git, repo, &i.remote)?;
                checked_branch(&i.from)?;
                let to = i.to.trim().to_string();
                if !git.check_branch_name(&to)? || to == i.from {
                    return Err(CoreError::InvalidInput("쓸 수 없는 브랜치 이름이에요"));
                }
                let expect = Oid::parse(&i.expect_tip)
                    .ok_or(CoreError::InvalidInput("버전이 해시가 아니에요"))?;
                if tracking_tip(git, repo, &i.remote, &to)?.is_some() {
                    return Err(CoreError::RemoteNameTaken);
                }
                refuse_default(&net, &i.remote, &i.from)?;
                // 서버의 지금 값으로 다시 본다: 새 이름은 없고 옛 이름은 본 값이어야 한다.
                if net.server_branch(&i.remote, &to)?.is_some() {
                    return Err(CoreError::RemoteNameTaken);
                }
                match net.server_branch(&i.remote, &i.from)? {
                    Some(server) if server == expect => {}
                    Some(_) => return Err(remote_moved()),
                    None => return Err(CoreError::InvalidInput("그 원격 브랜치가 없어요")),
                }
                crate::undo::backup::backup_oid(
                    git,
                    repo,
                    &expect,
                    &format!("remote-branch-rename-{}-{}", i.remote, i.from),
                )?;
                match net.push_new_ref(&i.remote, &to, &expect) {
                    Ok(_) => {}
                    Err(CoreError::Git(e)) if e.kind() == FailureKind::LeaseStale => {
                        return Err(CoreError::RemoteNameTaken);
                    }
                    Err(e) => return Err(e),
                }
                // 여기부터 실패·취소면 되돌리지 않는다: 새 이름은 올라갔고 옛 이름이 남았다.
                if let Err(e) = net.push_delete(&i.remote, &i.from, &expect) {
                    let detail = e.detail().unwrap_or_else(|| e.to_string());
                    return Err(CoreError::RemoteRenamePartial(detail));
                }
                let old = format!("{}/{}", i.remote, i.from);
                let new = format!("{}/{to}", i.remote);
                if tracking_tip(git, repo, &i.remote, &to)?.is_some() {
                    for b in git.branches(repo)? {
                        if b.kind == git::BranchKind::Local
                            && b.upstream.as_deref() == Some(old.as_str())
                        {
                            git.run(
                                repo,
                                &git::GitOp::SetUpstream {
                                    branch: b.name.clone(),
                                    upstream: new.clone(),
                                },
                            )?;
                        }
                    }
                }
                Ok(Outcome::RemoteBranchRenamed {
                    remote: i.remote.clone(),
                    from: i.from.clone(),
                    to,
                })
            }
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}
