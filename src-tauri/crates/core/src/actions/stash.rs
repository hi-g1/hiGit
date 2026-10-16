//! 임시 보관(`stash`)과 꺼내기(`take-out`, D-076). 맡겨 두기(`park`)와 달리 일반 stash 목록을 쓴다.

use git::{GitOp, Oid};

use super::repo_of;
use crate::actions::switch::refuse_during_operation;
use crate::view::{ActionInput, TakeOutSource};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 앱이 만든 임시 보관 항목의 메시지 머리.
pub(crate) const STASH_PREFIX: &str = "higit stash";
/// 맡겨 두기가 잠깐 만드는 항목의 메시지 머리(목록에서 빼고 보여준다).
pub(crate) const PARKED_PREFIX: &str = "higit parked";

/// reflog 제목(`On main: 메시지`, `WIP on main: …`)에서 메시지만.
pub(crate) fn stash_message(subject: &str) -> &str {
    subject.split_once(": ").map_or(subject, |(_, m)| m)
}

/// 작업 내역의 백업 ref(stash 모양)나 맡긴 ref를 깨끗한 작업 폴더에 꺼낸다(D-085). 백업 ref는
/// 남기고, 맡긴 ref는 꺼낸 뒤 백업으로 옮겨 지운다. 목록에 있는 ref만 받는다.
fn take_out_ref(git: &dyn git::GitBackend, repo: &git::RepoPath, refname: &str) -> Result<Outcome> {
    use crate::undo::backup::BACKUP_PREFIX;
    let prefix = if refname.starts_with(BACKUP_PREFIX) {
        BACKUP_PREFIX
    } else if refname.starts_with(crate::parked::PARKED_PREFIX) {
        crate::parked::PARKED_PREFIX
    } else {
        return Err(CoreError::InvalidInput("없는 항목이에요"));
    };
    let entry = git
        .list_refs(repo, prefix)?
        .into_iter()
        .find(|r| r.name == refname)
        .ok_or(CoreError::InvalidInput("없는 항목이에요"))?;
    let oid = Oid::parse(&entry.oid).ok_or(CoreError::InvalidInput("없는 항목이에요"))?;
    refuse_during_operation(git, repo)?;
    if prefix == BACKUP_PREFIX {
        if crate::recovery::kind_of(git, repo, &entry.oid) != crate::view::RestoreKind::Changes {
            return Err(CoreError::InvalidInput("꺼낼 수 있는 항목이 아니에요"));
        }
        crate::parked::refuse_unsafe(git, repo, &oid)?;
        git.run(repo, &GitOp::Unpark { oid })?;
        return Ok(Outcome::TookOut);
    }
    let branch =
        crate::parked::branch_of(refname).ok_or(CoreError::InvalidInput("없는 항목이에요"))?;
    crate::parked::refuse_unsafe(git, repo, &oid)?;
    git.run(repo, &GitOp::Unpark { oid: oid.clone() })?;
    crate::undo::backup::backup_oid(git, repo, &oid, &format!("parked-{branch}"))?;
    git.delete_ref(repo, refname, &git::RefExpect::Is(oid))?;
    Ok(Outcome::TookOut)
}

/// 지금 브랜치에 맡겨 둔 파일을 꺼낸다. 겹치면 충돌을 남기고 표식을 적어 충돌 해결 화면으로(D-083).
fn take_out_parked(
    git: &dyn git::GitBackend,
    repo: &git::RepoPath,
    key: &crate::undo::JournalKey,
    branch: &str,
) -> Result<Outcome> {
    crate::actions::names::validate_ref_like(branch)?;
    if git.current_branch(repo)?.as_deref() != Some(branch) {
        return Err(CoreError::InvalidInput(
            "지금 브랜치의 맡긴 파일만 꺼낼 수 있어요",
        ));
    }
    let name = crate::parked::ref_name(branch);
    let oid = git
        .resolve_ref(repo, &name)?
        .ok_or(CoreError::InvalidInput("맡겨 둔 파일이 없어요"))?;
    crate::parked::refuse_unsafe(git, repo, &oid)?;
    git.run(repo, &GitOp::UnparkKeep { oid: oid.clone() })?;
    let paths = crate::actions::merge::conflict_paths(git, repo)?;
    if paths.is_empty() {
        // 깨끗이 꺼냈다. 맡긴 ref는 백업으로 옮긴 뒤 지운다(D-054).
        crate::undo::backup::backup_oid(git, repo, &oid, &format!("parked-{branch}"))?;
        git.delete_ref(repo, &name, &git::RefExpect::Is(oid))?;
        return Ok(Outcome::TookOut);
    }
    crate::parked::write_marker(
        &git.git_dir(repo)?,
        key,
        &crate::parked::UnparkMarker {
            branch: branch.to_string(),
            oid: oid.to_string(),
        },
    )?;
    Ok(Outcome::MergeConflicts { paths })
}

pub struct StashAction {
    meta: ActionMeta,
}

impl StashAction {
    pub fn stash() -> Self {
        Self {
            meta: ActionMeta {
                id: "stash",
                glossary_id: "stash",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Inverse,
            },
        }
    }

    pub fn take_out() -> Self {
        Self {
            meta: ActionMeta {
                id: "take-out",
                glossary_id: "stash",
                // 용어표의 stash 위험도(safe)를 따른다. 겹치면 아무것도 바꾸지 않고 멈춘다.
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Action for StashAction {
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
            // 되돌리기가 그때 만든 항목을 알아볼 수 있게 `refs/stash` 값을 남긴다.
            ActionInput::Stash => vec!["refs/stash".to_string()],
            _ => Vec::new(),
        }
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        refuse_during_operation(git, repo)?;
        match (self.meta.id, input) {
            ("stash", ActionInput::Stash) => {
                let branch = git
                    .current_branch(repo)?
                    .unwrap_or_else(|| "HEAD".to_string());
                let out = git.run(
                    repo,
                    &GitOp::Park {
                        message: format!("{STASH_PREFIX} {branch}"),
                    },
                )?;
                if out.oid.is_none() {
                    return Err(CoreError::InvalidInput("보관할 고친 파일이 없어요"));
                }
                Ok(Outcome::Stashed)
            }
            ("take-out", ActionInput::TakeOut(input)) => match &input.source {
                TakeOutSource::Stash { oid } => {
                    let oid = Oid::parse(oid).ok_or(CoreError::InvalidInput("없는 항목이에요"))?;
                    // 목록에 있는 항목만 꺼낸다(화면이 아무 해시나 줄 수 없다).
                    if !git.stash_list(repo)?.iter().any(|e| e.oid == oid) {
                        return Err(CoreError::InvalidInput("없는 항목이에요"));
                    }
                    crate::parked::refuse_unsafe(git, repo, &oid)?;
                    git.run(repo, &GitOp::Unpark { oid: oid.clone() })?;
                    git.run(repo, &GitOp::StashDropOid { oid })?;
                    Ok(Outcome::TookOut)
                }
                TakeOutSource::Parked { branch } => take_out_parked(git, repo, ctx.key, branch),
                TakeOutSource::Ref { refname } => take_out_ref(git, repo, refname),
            },
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}
