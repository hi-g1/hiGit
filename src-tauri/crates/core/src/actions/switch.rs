//! 브랜치 갈아타기(D-047, D-054, D-195). 고친 파일은 떠나는 브랜치에 맡겨 두고 돌아오면 꺼내거나,
//! 도착할 브랜치로 가져간다.
//!
//! 맡기기 순서(계획 005 "검토 반영" H8·H9): 맡기기 → ref 쓰기 → stash 목록에서 빼기 → 갈아타기 →
//! 도착한 브랜치에 맡긴 것이 있으면 겹치지 않을 때만 꺼내기. 갈아타기가 실패하면 곧바로 원래대로 꺼낸다.
//! 가져가기 순서: 겹침 검사 → stash로 만들어 백업 ref에 남기기 → 목록에서 빼기 → 갈아타기 → 적용.

use git::{FailureKind, GitBackend, GitOp, MergeCheck, Oid, RefExpect, RepoPath, StatusEntry};

use super::{names, needs_repo, repo_of};
use crate::undo::backup;
use crate::view::{ActionInput, CarryBlock, DirtyChoice, SwitchInput};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk, parked,
};

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
    /// 두고 가거나 가져간 고친 파일 수.
    pub files: u32,
    /// 고친 파일을 도착한 브랜치로 가져왔다(D-195).
    pub carried: bool,
}

impl SwitchReport {
    fn stayed(branch: &str) -> Self {
        Self {
            branch: branch.to_string(),
            parked: None,
            unparked: false,
            unpark_blocked: false,
            files: 0,
            carried: false,
        }
    }
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
    // `--end-of-options`를 넣지 못하는 git에서도 이름이 옵션으로 읽히지 않게 git 규칙으로 한 번 더 본다.
    if !git.check_branch_name(&input.branch)? {
        return Err(CoreError::InvalidInput("쓸 수 없는 브랜치 이름이에요"));
    }
    refuse_during_operation(git, repo)?;
    let current = git.current_branch(repo)?;
    if input.remote.is_none() && current.as_deref() == Some(input.branch.as_str()) {
        return Ok(SwitchReport::stayed(&input.branch));
    }
    let op = match &input.remote {
        Some(remote) => GitOp::SwitchTrack {
            local: input.branch.clone(),
            remote_branch: format!("{remote}/{}", input.branch),
        },
        None => GitOp::Switch {
            branch: input.branch.clone(),
        },
    };

    // 버전이 없거나(첫 저장 전) 특정 버전을 보는 중이면 맡기지도 가져가지도 않는다.
    let entries = git.status(repo)?.entries;
    let leaving = match (&current, git.resolve_ref(repo, "HEAD")?) {
        (Some(branch), Some(head)) if !entries.is_empty() => Some((branch.clone(), head)),
        _ => None,
    };
    let files = u32::try_from(entries.len()).unwrap_or(u32::MAX);
    if let Some((from, head)) = &leaving
        && input.dirty == Some(DirtyChoice::Carry)
    {
        return carry(git, repo, input, &op, from, head, &entries, files);
    }

    // 1. 떠나는 브랜치에 맡긴다.
    let parked = match &leaving {
        Some((branch, _)) => {
            park(git, repo, branch)?.map(|(oid, undo)| (branch.clone(), oid, undo))
        }
        None => None,
    };

    // 2. 갈아탄다. 실패하면 맡긴 것을 같은 브랜치에 곧바로 꺼내 원래대로 돌린다.
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
        files: if parked.is_some() { files } else { 0 },
        parked: parked.map(|(b, oid, _)| (b, oid)),
        unparked,
        unpark_blocked,
        carried: false,
    })
}

/// 고친 파일을 들고 갈아탄다(D-195). 겹치면 갈아타기 전에 `CarryBlocked`이고 작업 폴더는 그대로다.
/// 적용이 실패하면 갈아타기를 되돌리고 원래 브랜치에 다시 꺼낸다. stash는 처음부터 백업 ref에 있다.
#[allow(clippy::too_many_arguments)]
fn carry(
    git: &dyn GitBackend,
    repo: &RepoPath,
    input: &SwitchInput,
    op: &GitOp,
    from: &str,
    head: &Oid,
    entries: &[StatusEntry],
    files: u32,
) -> Result<SwitchReport> {
    if let CarryCheck::Blocked { paths, .. } = carry_check(
        git,
        repo,
        &input.branch,
        input.remote.as_deref(),
        head,
        entries,
    )? {
        return Err(CoreError::CarryBlocked(paths.join("\n")));
    }
    let message = format!("higit carry {from}");
    let Some(oid) = git.run(repo, &GitOp::Park { message })?.oid else {
        // 그사이 고친 파일이 없어졌다.
        git.run(repo, op)?;
        return Ok(SwitchReport::stayed(&input.branch));
    };
    // 1. 백업 ref에 남기고 stash 목록에서 뺀다. 못 하면 곧바로 꺼내 원래대로 돌린다.
    let kept = backup::backup_oid(git, repo, &oid, &format!("switch-carry-{}", input.branch))
        .and_then(|_| {
            git.run(repo, &GitOp::StashDropIf { oid: oid.clone() })
                .map(|_| ())
                .map_err(CoreError::from)
        });
    if let Err(e) = kept {
        put_back(git, repo, &oid)?;
        let _ = git.run(repo, &GitOp::StashDropIf { oid });
        return Err(e);
    }
    // 2. 갈아탄다. 실패하면 같은 브랜치에 곧바로 꺼낸다.
    if let Err(e) = git.run(repo, op) {
        put_back(git, repo, &oid)?;
        return Err(e.into());
    }
    // 3. 같은 것을 적용한다. 실패하면(겹침, 새 파일이 이미 있음) 깨끗한 상태로 되돌아와 있다.
    if let Err(e) = git.run(repo, &GitOp::Unpark { oid: oid.clone() }) {
        go_back(git, repo, input, from)?;
        put_back(git, repo, &oid)?;
        return Err(match e.kind() {
            FailureKind::UnparkConflict | FailureKind::WouldOverwrite => {
                CoreError::CarryBlocked(e.detail())
            }
            _ => e.into(),
        });
    }
    Ok(SwitchReport {
        branch: input.branch.clone(),
        parked: None,
        unparked: false,
        unpark_blocked: false,
        files,
        carried: true,
    })
}

/// 가져가려던 파일을 지금 브랜치에 꺼낸다. 못 꺼내도 백업 ref나 stash 목록에 남아 있다.
fn put_back(git: &dyn GitBackend, repo: &RepoPath, oid: &Oid) -> Result<()> {
    git.run(repo, &GitOp::Unpark { oid: oid.clone() })
        .map(|_| ())
        .map_err(|e| CoreError::ParkBlocked(e.detail()))
}

/// 적용하지 못한 가져가기에서 떠난 브랜치로 돌아간다. 원격에서 새로 만든 로컬 브랜치는 지운다.
fn go_back(git: &dyn GitBackend, repo: &RepoPath, input: &SwitchInput, from: &str) -> Result<()> {
    let created = match &input.remote {
        Some(_) => git.resolve_ref(repo, "HEAD")?,
        None => None,
    };
    git.run(
        repo,
        &GitOp::Switch {
            branch: from.to_string(),
        },
    )
    .map_err(|e| CoreError::ParkBlocked(e.detail()))?;
    if let Some(tip) = created {
        git.delete_ref(
            repo,
            &format!("refs/heads/{}", input.branch),
            &RefExpect::Is(tip),
        )?;
    }
    Ok(())
}

/// 가져가기 미리 보기 결과.
pub(crate) enum CarryCheck {
    Clear,
    Blocked {
        reason: CarryBlock,
        /// 겹치는 파일(5개까지).
        paths: Vec<String>,
        total: u32,
    },
}

/// 미리 계산하는 고친 파일 상한. 넘으면 스냅샷을 만들지 않고 가져가기를 끈다.
const CARRY_MAX_FILES: usize = 500;
const CARRY_MAX_BYTES: u64 = 50 * 1024 * 1024;
/// 까닭에 보여 줄 파일 수.
const CARRY_SHOW: usize = 5;

/// 고친 파일을 `branch`(원격이면 `<remote>/<branch>`)로 가져갈 수 있는지 본다. ref·index·작업 폴더는
/// 그대로다(스냅샷은 어디에도 매달리지 않은 객체만 만든다).
pub(crate) fn carry_check(
    git: &dyn GitBackend,
    repo: &RepoPath,
    branch: &str,
    remote: Option<&str>,
    head: &Oid,
    entries: &[StatusEntry],
) -> Result<CarryCheck> {
    let blocked = |reason, mut paths: Vec<String>| {
        let total = u32::try_from(paths.len()).unwrap_or(u32::MAX);
        paths.truncate(CARRY_SHOW);
        CarryCheck::Blocked {
            reason,
            paths,
            total,
        }
    };
    // 도착할 브랜치에 두고 간 파일이 있으면 가져간 파일과 섞인다.
    if let Some(oid) = git.resolve_ref(repo, &parked::ref_name(branch))? {
        let paths = git.parked_paths(repo, &oid).unwrap_or_default();
        return Ok(blocked(CarryBlock::ParkedThere, paths));
    }
    let mut paths: Vec<String> = entries
        .iter()
        .flat_map(|e| std::iter::once(e.path.clone()).chain(e.orig_path.clone()))
        .collect();
    paths.sort();
    paths.dedup();
    let bytes: u64 = paths
        .iter()
        .filter_map(|p| std::fs::symlink_metadata(repo.as_path().join(p)).ok())
        .map(|m| m.len())
        .sum();
    if paths.len() > CARRY_MAX_FILES || bytes > CARRY_MAX_BYTES {
        return Ok(blocked(CarryBlock::TooBig, Vec::new()));
    }
    let tip_ref = match remote {
        Some(r) => format!("refs/remotes/{r}/{branch}"),
        None => format!("refs/heads/{branch}"),
    };
    // 도착할 곳이 없으면 갈아타기가 알린다. 같은 버전이면 겹칠 것이 없다.
    let Some(tip) = git.resolve_ref(repo, &tip_ref)? else {
        return Ok(CarryCheck::Clear);
    };
    if &tip == head {
        return Ok(CarryCheck::Clear);
    }
    let snapshot = git
        .run(
            repo,
            &GitOp::SnapshotFiles {
                paths,
                message: "higit carry check".into(),
            },
        )?
        .oid
        .ok_or(CoreError::BackupFailed)?;
    Ok(
        match git.merge_check(repo, Some(head.as_str()), tip.as_str(), snapshot.as_str())? {
            MergeCheck::Clean => CarryCheck::Clear,
            MergeCheck::Overlap { paths } => blocked(CarryBlock::Overlap, paths),
        },
    )
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
    if name.len() - parked::PARKED_PREFIX.len() > parked::MAX_NAME_BYTES {
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
