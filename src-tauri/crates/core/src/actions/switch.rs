//! 브랜치 갈아타기(D-047, D-054). 고친 파일은 떠나는 브랜치에 맡겨 두고, 돌아오면 꺼낸다.
//!
//! 순서(계획 005 "검토 반영" H8·H9): 맡기기 → ref 쓰기 → stash 목록에서 빼기 → 갈아타기 →
//! 도착한 브랜치에 맡긴 것이 있으면 겹치지 않을 때만 꺼내기. 갈아타기가 실패하면 곧바로 원래대로 꺼낸다.

use git::{GitBackend, GitOp, MergeCheck, Oid, RefExpect, RepoPath};

use super::{names, needs_repo, repo_of};
use crate::undo::backup;
use crate::view::{ActionInput, SwitchInput};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk, parked,
};

/// 맡긴 ref 이름 한 조각의 한도(파일 시스템 이름 한도 255바이트 안쪽).
const MAX_PARKED_NAME_BYTES: usize = 200;

pub struct SwitchAction {
    meta: ActionMeta,
}

impl SwitchAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "switch",
                glossary_id: "switch",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::SwitchBack,
            },
        }
    }
}

impl Default for SwitchAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for SwitchAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Switch(input) = input else {
            return Err(CoreError::InputMismatch("switch"));
        };
        let report = switch_with_parking(ctx.git, repo_of(ctx)?, input)?;
        Ok(Outcome::Switched(report))
    }
}

/// 갈아타기 결과. 화면 토스트와 되돌리기에 쓴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchReport {
    pub branch: String,
    /// 떠난 브랜치에 맡긴 파일(브랜치, stash 해시).
    pub parked: Option<(String, Oid)>,
    /// 도착한 브랜치에 맡겨 두었던 파일을 꺼냈다.
    pub unparked: bool,
    /// 도착한 브랜치에 맡긴 파일이 지금 버전과 겹쳐 꺼내지 않았다(D-054).
    pub unpark_blocked: bool,
}

/// 맡겨 두는 갈아타기. 되돌리기·다시 하기도 이 흐름을 쓴다.
pub(crate) fn switch_with_parking(
    git: &dyn GitBackend,
    repo: &RepoPath,
    input: &SwitchInput,
) -> Result<SwitchReport> {
    names::validate_ref_like(&input.branch)?;
    if let Some(remote) = &input.remote {
        names::validate_ref_like(remote)?;
    }
    refuse_during_operation(git, repo)?;
    let current = git.current_branch(repo)?;
    if input.remote.is_none() && current.as_deref() == Some(input.branch.as_str()) {
        return Ok(SwitchReport {
            branch: input.branch.clone(),
            parked: None,
            unparked: false,
            unpark_blocked: false,
        });
    }

    // 1. 떠나는 브랜치에 맡긴다. 버전이 없거나(첫 저장 전) 특정 버전을 보는 중이면 맡기지 않는다.
    let parked = match (&current, git.resolve_ref(repo, "HEAD")?) {
        (Some(branch), Some(_)) if !git.status(repo)?.entries.is_empty() => {
            park(git, repo, branch)?.map(|(oid, undo)| (branch.clone(), oid, undo))
        }
        _ => None,
    };

    // 2. 갈아탄다. 실패하면 맡긴 것을 같은 브랜치에 곧바로 꺼내 원래대로 돌린다.
    let op = match &input.remote {
        Some(remote) => GitOp::SwitchTrack {
            local: input.branch.clone(),
            remote_branch: format!("{remote}/{}", input.branch),
        },
        None => GitOp::Switch {
            branch: input.branch.clone(),
        },
    };
    if let Err(e) = git.run(repo, &op) {
        if let Some((_, oid, undo)) = &parked {
            match git.run(repo, &GitOp::Unpark { oid: oid.clone() }) {
                Ok(_) => undo.revert(git, repo)?,
                Err(unpark) => return Err(CoreError::ParkBlocked(unpark.detail())),
            }
        }
        return Err(e.into());
    }

    // 3. 도착한 브랜치에 맡겨 둔 것이 있으면 겹치지 않을 때만 꺼낸다(D-054).
    // 갈아타기는 이미 끝났으므로 여기서 나는 오류는 "꺼내지 못함"으로 알린다. 오류로 올리면
    // 작업 목록에 남지 않아 되돌리기가 없어진다. 맡긴 파일은 ref에 그대로 있다.
    let (unparked, unpark_blocked) =
        unpark_arrived(git, repo, &input.branch).unwrap_or((false, true));
    Ok(SwitchReport {
        branch: input.branch.clone(),
        parked: parked.map(|(b, oid, _)| (b, oid)),
        unparked,
        unpark_blocked,
    })
}

/// 맡긴 ref를 쓰기 전 상태. 갈아타기가 실패하면 되돌린다.
pub(crate) enum ParkUndo {
    Delete { name: String, oid: Oid },
    Restore { name: String, old: Oid, oid: Oid },
}

impl ParkUndo {
    fn revert(&self, git: &dyn GitBackend, repo: &RepoPath) -> Result<()> {
        match self {
            ParkUndo::Delete { name, oid } => {
                git.delete_ref(repo, name, &RefExpect::Is(oid.clone()))?
            }
            ParkUndo::Restore { name, old, oid } => {
                git.update_ref(repo, name, old, &RefExpect::Is(oid.clone()))?
            }
        }
        Ok(())
    }
}

/// 고친 파일을 `refs/higit/parked/<브랜치>`에 맡긴다. 맡길 것이 없으면 `None`.
fn park(git: &dyn GitBackend, repo: &RepoPath, branch: &str) -> Result<Option<(Oid, ParkUndo)>> {
    let name = parked::ref_name(branch);
    if name.len() - parked::PARKED_PREFIX.len() > MAX_PARKED_NAME_BYTES {
        return Err(CoreError::InvalidInput(
            "브랜치 이름이 너무 길어 고친 파일을 맡길 수 없어요",
        ));
    }
    let message = format!("higit parked {branch}");
    let Some(oid) = git.run(repo, &GitOp::Park { message })?.oid else {
        return Ok(None);
    };
    match keep_parked(git, repo, branch, &name, &oid) {
        Ok(undo) => Ok(Some((oid, undo))),
        Err(e) => {
            // 작업 폴더는 이미 비었다. 맡긴 파일을 곧바로 꺼내 원래대로 돌리고 갈아타지 않는다.
            let detail = e.detail().unwrap_or_default();
            match git.run(repo, &GitOp::Unpark { oid: oid.clone() }) {
                Ok(_) => {
                    let _ = git.run(repo, &GitOp::StashDropIf { oid });
                    Err(CoreError::ParkBlocked(detail))
                }
                // 꺼내지도 못하면 stash 목록에 남아 있다(파일을 잃지 않는다).
                Err(unpark) => Err(CoreError::ParkBlocked(unpark.detail())),
            }
        }
    }
}

/// 만든 stash를 맡긴 ref에 쓰고 stash 목록에서 뺀다. 실패하면 ref를 되돌린다.
fn keep_parked(
    git: &dyn GitBackend,
    repo: &RepoPath,
    branch: &str,
    name: &str,
    oid: &Oid,
) -> Result<ParkUndo> {
    let name = name.to_string();
    // ref를 먼저 쓰고 나서 stash 목록에서 뺀다. 쓰기가 실패해도 목록에 남아 파일을 잃지 않는다.
    let undo = match git.resolve_ref(repo, &name)? {
        Some(old) => {
            // 이미 맡긴 것이 있으면(앞서 꺼내지 못함) 백업 ref로 옮겨 두고 새로 쓴다.
            backup::backup_oid(git, repo, &old, &format!("parked-{branch}"))?;
            git.update_ref(repo, &name, oid, &RefExpect::Is(old.clone()))?;
            ParkUndo::Restore {
                name,
                old,
                oid: oid.clone(),
            }
        }
        None => {
            git.update_ref(repo, &name, oid, &RefExpect::Absent)?;
            ParkUndo::Delete {
                name,
                oid: oid.clone(),
            }
        }
    };
    if let Err(e) = git.run(repo, &GitOp::StashDropIf { oid: oid.clone() }) {
        let _ = undo.revert(git, repo);
        return Err(e.into());
    }
    Ok(undo)
}

/// 도착한 브랜치의 맡긴 파일을 꺼낸다. (꺼냄, 겹쳐서 못 꺼냄).
fn unpark_arrived(git: &dyn GitBackend, repo: &RepoPath, branch: &str) -> Result<(bool, bool)> {
    let name = parked::ref_name(branch);
    let Some(oid) = git.resolve_ref(repo, &name)? else {
        return Ok((false, false));
    };
    let head = git.resolve_ref(repo, "HEAD")?;
    let base = git.resolve_ref(repo, &format!("{oid}^1"))?;
    // 맡길 때와 같은 버전이면 바로, 아니면 겹침 검사가 깨끗할 때만 꺼낸다.
    let clean = match (&base, &head) {
        (Some(b), Some(h)) if b == h => true,
        (Some(b), Some(_)) => {
            git.merge_check(repo, Some(b.as_str()), "HEAD", oid.as_str())? == MergeCheck::Clean
        }
        _ => false,
    };
    if !clean {
        return Ok((false, true));
    }
    match git.run(repo, &GitOp::Unpark { oid: oid.clone() }) {
        Ok(_) => {
            git.delete_ref(repo, &name, &RefExpect::Is(oid))?;
            Ok((true, false))
        }
        // 겹치거나, 갈아탄 뒤에도 고친 파일이 남아 있으면(특정 버전에서 온 경우 등) 두고 알린다.
        Err(e)
            if matches!(
                e.kind(),
                git::FailureKind::UnparkConflict | git::FailureKind::WouldOverwrite
            ) =>
        {
            Ok((false, true))
        }
        Err(e) => Err(e.into()),
    }
}

/// 합치기·리베이스·cherry-pick 중에는 갈아타지 않는다(맡기기와 꺼내기 실패 정리가 그 작업을 망친다).
pub(crate) fn refuse_during_operation(git: &dyn GitBackend, repo: &RepoPath) -> Result<()> {
    let dir = git.git_dir(repo)?;
    let busy = [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
    ]
    .iter()
    .any(|f| dir.join(f).exists());
    if busy {
        return Err(CoreError::OperationInProgress);
    }
    Ok(())
}
