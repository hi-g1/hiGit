//! 브랜치 지우기·이름 바꾸기(`branch-delete`·`branch-rename`, D-103). 로컬 브랜치만 다룬다.
//! 지우기는 끝 버전을 백업한 뒤 지우고 맡긴 ref를 백업으로 옮긴다. Ctrl+Z가 브랜치를 되살리고
//! 같은 이름의 원격 브랜치가 하나뿐이면 다시 잇는다. 이름 바꾸기는 upstream을 끊고 맡긴 ref를
//! 옮긴다. Ctrl+Z로 원래 이름(upstream은 끊긴 채), 다시 하기는 없다.

use git::{GitOp, RepoPath};

use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct BranchEditAction {
    meta: ActionMeta,
}

impl BranchEditAction {
    pub fn delete() -> Self {
        Self {
            meta: ActionMeta {
                id: "branch-delete",
                glossary_id: "branch-delete",
                risk: Risk::Caution,
                undo: crate::undo::UndoKind::RestoreRef,
            },
        }
    }

    pub fn rename() -> Self {
        Self {
            meta: ActionMeta {
                id: "branch-rename",
                glossary_id: "branch-rename",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Inverse,
            },
        }
    }
}

/// 있는 로컬 브랜치의 이름(앞뒤 공백 없이).
fn existing(git: &dyn git::GitBackend, repo: &RepoPath, raw: &str) -> Result<String> {
    let name = raw.trim();
    crate::actions::names::validate_ref_like(name)?;
    let local = git
        .branches(repo)?
        .into_iter()
        .any(|b| b.kind == git::BranchKind::Local && b.name == name);
    if !local {
        return Err(CoreError::InvalidInput("그 브랜치가 없어요"));
    }
    Ok(name.to_string())
}

impl Action for BranchEditAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn tracked_refs(&self, input: &ActionInput) -> Vec<String> {
        match input {
            ActionInput::BranchDelete(i) => vec![format!("refs/heads/{}", i.name.trim())],
            ActionInput::BranchRename(i) => vec![
                format!("refs/heads/{}", i.from.trim()),
                format!("refs/heads/{}", i.to.trim()),
            ],
            _ => Vec::new(),
        }
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        match (self.meta.id, input) {
            ("branch-delete", ActionInput::BranchDelete(i)) => {
                let name = existing(git, repo, &i.name)?;
                let branches = git.branches(repo)?;
                let current = git.current_branch(repo)?;
                let default =
                    crate::query::default_branch(git, repo, &branches, &git.remotes(repo)?)?;
                if current.as_deref() == Some(name.as_str())
                    || default.as_deref() == Some(name.as_str())
                    || crate::actions::merge::checked_out_elsewhere(git, repo, &name)?
                {
                    return Err(CoreError::BranchProtected);
                }
                refuse_during_operation(git, repo)?;
                let tip = git
                    .resolve_ref(repo, &format!("refs/heads/{name}"))?
                    .ok_or(CoreError::InvalidInput("그 브랜치가 없어요"))?;
                // 끝 버전을 먼저 살려 둔다. 못 남기면 지우지 않는다.
                crate::undo::backup::backup_oid(git, repo, &tip, &format!("branch-delete-{name}"))?;
                git.run(repo, &GitOp::BranchDelete { name: name.clone() })?;
                // 지운 뒤에 옮긴다: 지우기가 실패하면 맡긴 파일은 제자리에 있다.
                crate::parked::retire_to_backup(git, repo, &name)?;
                Ok(Outcome::BranchDeleted { name })
            }
            ("branch-rename", ActionInput::BranchRename(i)) => {
                let from = existing(git, repo, &i.from)?;
                let to = i.to.trim().to_string();
                if !git.check_branch_name(&to)? {
                    return Err(CoreError::InvalidInput("쓸 수 없는 브랜치 이름이에요"));
                }
                if to == from {
                    return Err(CoreError::NoChange);
                }
                // 대소문자만 다른 이름은 Windows에서 같은 파일이다. 같은 브랜치의 대소문자만 바꾸는
                // 것은 `-M`으로 한다.
                let case_only = to.to_lowercase() == from.to_lowercase();
                let taken = git.branches(repo)?.iter().any(|b| {
                    b.kind == git::BranchKind::Local
                        && b.name != from
                        && b.name.to_lowercase() == to.to_lowercase()
                });
                if taken {
                    return Err(CoreError::BranchExists);
                }
                refuse_during_operation(git, repo)?;
                if crate::marker::exists(&git.git_dir(repo)?, crate::marker::Name::Unpark) {
                    return Err(CoreError::OperationInProgress);
                }
                let had_upstream = git.branches(repo)?.iter().any(|b| {
                    b.kind == git::BranchKind::Local && b.name == from && b.upstream.is_some()
                });
                git.run(
                    repo,
                    &GitOp::BranchRename {
                        from: from.clone(),
                        to: to.clone(),
                        force: case_only,
                    },
                )?;
                if had_upstream {
                    git.run(repo, &GitOp::UnsetUpstream { branch: to.clone() })?;
                }
                crate::parked::move_ref(git, repo, &from, &to)?;
                Ok(Outcome::BranchRenamed { from, to })
            }
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}
