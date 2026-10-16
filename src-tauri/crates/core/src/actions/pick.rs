//! 되돌리기(`revert`)와 이 버전만 복사하기(`cherry-pick`)의 시작(D-098, D-101).
//!
//! 시작 전에 서명한 표식(`operation.json`)을 남기고 git을 부른다. 끝까지 가면 표식을 지운다.
//! 충돌이나 hook으로 멈추면 표식을 두고 충돌 해결 화면으로 간다(마치기·그만두기는
//! `conflict-finish`·`conflict-abort`). 비어 버리면 건너뛰고 `NoChange`. 앱 데이터 폴더를 못 써
//! 서명 키가 이번 실행만의 것이면 멈춘 작업을 껐다 켠 뒤 이어갈 수 없으므로 바로 그만둔다(D-102).

use git::{GitOp, LogQuery, Oid};

use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::guard::{self, Scope};
use crate::operation::{self, OpKind, OpMarker};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct PickAction {
    meta: ActionMeta,
    kind: OpKind,
}

impl PickAction {
    pub fn revert() -> Self {
        Self {
            meta: ActionMeta {
                id: "revert",
                glossary_id: "revert",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::RestoreTree,
            },
            kind: OpKind::Revert,
        }
    }

    pub fn cherry_pick() -> Self {
        Self {
            meta: ActionMeta {
                id: "cherry-pick",
                glossary_id: "cherry-pick",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::RestoreTree,
            },
            kind: OpKind::CherryPick,
        }
    }

    /// 멈춘 동안 남는 git의 표시 파일.
    fn pick_head(&self) -> &'static str {
        match self.kind {
            OpKind::Revert => "REVERT_HEAD",
            _ => "CHERRY_PICK_HEAD",
        }
    }
}

impl Action for PickAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let oid = match (self.kind, input) {
            (OpKind::Revert, ActionInput::Revert(i))
            | (OpKind::CherryPick, ActionInput::CherryPick(i)) => &i.oid,
            _ => return Err(CoreError::InputMismatch(self.meta.id)),
        };
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let branch = guard::require_branch(git, repo)?;
        refuse_during_operation(git, repo)?;
        super::refuse_conflicts(git, repo)?;
        // 넣은 파일이 있으면 새 버전에 섞인다.
        guard::refuse_staged(git, repo)?;
        let head = git
            .resolve_ref(repo, "HEAD")?
            .ok_or(CoreError::InvalidInput("없는 버전이에요"))?;
        let merge = |oid: &Oid| -> Result<bool> {
            Ok(git.resolve_ref(repo, &format!("{oid}^2"))?.is_some())
        };
        let op = match self.kind {
            OpKind::Revert => {
                let oid = guard::verified_commit(git, repo, oid, Scope::HeadAncestor)?;
                // 합친 버전은 첫 부모 쪽을 기준으로 되돌린다.
                let mainline = merge(&oid)?.then_some(1);
                GitOp::RevertStart { oid, mainline }
            }
            _ => {
                let oid = guard::verified_commit(git, repo, oid, Scope::BranchReachable)?;
                if merge(&oid)? {
                    return Err(CoreError::InvalidInput("합친 버전은 복사할 수 없어요"));
                }
                if git.is_ancestor(repo, oid.as_str(), head.as_str())? {
                    return Err(CoreError::InvalidInput(
                        "이미 지금 브랜치에 있는 버전이에요",
                    ));
                }
                GitOp::CherryPickStart { oid }
            }
        };
        if !git.has_identity(repo)? {
            return Err(CoreError::IdentityMissing);
        }
        let picked = match &op {
            GitOp::RevertStart { oid, .. } | GitOp::CherryPickStart { oid } => oid.clone(),
            _ => unreachable!(),
        };
        let git_dir = git.git_dir(repo)?;
        operation::write(
            &git_dir,
            ctx.key,
            &OpMarker {
                kind: self.kind,
                branch,
                head_before: head.to_string(),
                base: None,
                oids: vec![picked.to_string()],
                started_at: crate::undo::journal::now(),
            },
        )?;
        match git.run(repo, &op) {
            Ok(_) => {
                operation::clear(&git_dir);
                let commit = git
                    .log(
                        repo,
                        &LogQuery {
                            skip: 0,
                            limit: 1,
                            rev: None,
                        },
                    )?
                    .into_iter()
                    .next()
                    .ok_or(CoreError::NotARepo)?;
                let (short_oid, summary) = (commit.short_oid, commit.summary);
                Ok(match self.kind {
                    OpKind::Revert => Outcome::Reverted { short_oid, summary },
                    _ => Outcome::Picked { short_oid, summary },
                })
            }
            Err(e) => self.stopped(ctx, repo, e),
        }
    }
}

impl PickAction {
    /// git이 실패한 뒤의 상태를 본다.
    fn stopped(
        &self,
        ctx: &ExecCtx<'_>,
        repo: &git::RepoPath,
        error: git::GitError,
    ) -> Result<Outcome> {
        let git = ctx.git;
        let git_dir = git.git_dir(repo)?;
        let paused = git.resolve_ref(repo, self.pick_head())?.is_some();
        let paths = crate::actions::merge::conflict_paths(git, repo)?;
        if !paused && paths.is_empty() {
            // 시작하지 못했다(고친 파일과 겹침 등). git이 아무것도 바꾸지 않았다. 되돌리기는
            // 비어 버리면 멈추지 않고 "저장할 것 없음"으로 끝난다.
            operation::clear(&git_dir);
            if error.kind() == git::FailureKind::NothingToCommit {
                return Err(CoreError::NoChange);
            }
            return Err(error.into());
        }
        let staged = git
            .status(repo)?
            .entries
            .iter()
            .any(|e| !matches!(e.index, git::Change::Unmodified | git::Change::Untracked));
        if paths.is_empty() && !staged {
            // 이미 같은 내용이라 비어 버렸다.
            git.run(
                repo,
                &GitOp::OperationSkip {
                    kind: self.kind.git(),
                },
            )?;
            operation::clear(&git_dir);
            return Err(CoreError::NoChange);
        }
        if !ctx.key.is_persistent() {
            let _ = git.run(
                repo,
                &GitOp::OperationAbort {
                    kind: self.kind.git(),
                },
            );
            operation::clear(&git_dir);
            return Err(error.into());
        }
        Ok(Outcome::OperationConflicts {
            context: self.kind.context(),
            paths,
        })
    }
}
