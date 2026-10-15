//! 되돌리기·다시 하기 실행(D-042, D-065). journal의 마지막 줄을 그 액션의 `undo` 종류대로 되돌린다.

use git::{GitBackend, GitOp, Oid, RefExpect, RepoPath};

use super::journal::{self, Body, Journal};
use super::{UndoKind, UndoState, backup};
use crate::actions::switch::switch_with_parking;
use crate::parked;
use crate::view::{ActionInput, SwitchInput};
use crate::{CoreError, Result};

/// 되돌리기·다시 하기 결과. 화면은 토스트 문구와 이름 칸 채우기에 쓴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoOutcome {
    pub action: String,
    pub kind: UndoKind,
    pub branch: Option<String>,
    /// 버전 저장을 되돌렸으면 그때 쓴 메시지(이름 칸에 다시 채운다).
    pub message: Option<String>,
}

struct Entry {
    id: String,
    action: String,
    input: ActionInput,
    before: UndoState,
    after: UndoState,
    kind: UndoKind,
}

/// journal을 처음부터 따라가며 되돌릴 줄(마지막 적용)과 다시 할 줄(마지막 되돌림)을 찾는다.
/// 경계 줄을 만나면 그 앞은 모두 잊는다. 새 작업이 생기면 다시 하기는 사라진다.
fn stacks(lines: Vec<Body>) -> (Vec<Entry>, Vec<Entry>) {
    let mut applied: Vec<Entry> = Vec::new();
    let mut undone: Vec<Entry> = Vec::new();
    for line in lines {
        match line {
            Body::Action {
                id,
                action,
                input,
                before,
                after,
                kind,
                ..
            } => {
                applied.push(Entry {
                    id,
                    action,
                    input,
                    before,
                    after,
                    kind,
                });
                undone.clear();
            }
            Body::Barrier { .. } => {
                applied.clear();
                undone.clear();
            }
            Body::Undo { target, .. } => {
                if applied.last().is_some_and(|e| e.id == target)
                    && let Some(e) = applied.pop()
                {
                    undone.push(e);
                }
            }
            Body::Redo { target, .. } => {
                if undone.last().is_some_and(|e| e.id == target)
                    && let Some(e) = undone.pop()
                {
                    applied.push(e);
                }
            }
        }
    }
    (applied, undone)
}

pub(crate) fn undo(
    git: &dyn GitBackend,
    repo: &RepoPath,
    journal: &Journal,
) -> Result<UndoOutcome> {
    let (mut applied, _) = stacks(journal.read());
    let entry = applied.pop().ok_or(CoreError::NothingToUndo)?;
    let now = UndoState::capture(git, repo)?;
    // 앱 밖에서 버전이나 브랜치가 바뀌었으면 되돌리지 않는다(복구 화면으로 안내).
    if now.head != entry.after.head || now.branch != entry.after.branch {
        return Err(CoreError::UndoStale);
    }
    match entry.kind {
        UndoKind::RestoreRef => undo_commit(git, repo, &entry)?,
        UndoKind::RestoreIndex => {
            restore_index(git, repo, &now, &entry.after, &entry.before)?;
        }
        UndoKind::SwitchBack => undo_switch(git, repo, &entry)?,
        UndoKind::DeleteCreated => undo_create(git, repo, &entry)?,
        // 원격 작업은 1b에서 되돌리지 않는다. 올린 것은 기록을 바꾸지 않는다(D-042).
        UndoKind::Inverse if entry.action == "pull" => return Err(CoreError::NothingToUndo),
        UndoKind::Inverse => return Err(CoreError::UndoPushed),
        _ => return Err(CoreError::NothingToUndo),
    }
    journal.append(&Body::Undo {
        target: entry.id.clone(),
        at: journal::now(),
    })?;
    Ok(outcome(&entry, entry.before.branch.clone()))
}

pub(crate) fn redo(
    git: &dyn GitBackend,
    repo: &RepoPath,
    journal: &Journal,
) -> Result<UndoOutcome> {
    let (_, mut undone) = stacks(journal.read());
    let entry = undone.pop().ok_or(CoreError::NothingToUndo)?;
    let now = UndoState::capture(git, repo)?;
    if now.head != entry.before.head || now.branch != entry.before.branch {
        return Err(CoreError::UndoStale);
    }
    match entry.kind {
        UndoKind::RestoreRef => redo_commit(git, repo, &entry)?,
        UndoKind::RestoreIndex => {
            restore_index(git, repo, &now, &entry.before, &entry.after)?;
        }
        UndoKind::SwitchBack => redo_switch(git, repo, &entry)?,
        UndoKind::DeleteCreated => redo_create(git, repo, &entry)?,
        _ => return Err(CoreError::NothingToUndo),
    }
    journal.append(&Body::Redo {
        target: entry.id.clone(),
        at: journal::now(),
    })?;
    Ok(outcome(&entry, entry.after.branch.clone()))
}

fn outcome(entry: &Entry, branch: Option<String>) -> UndoOutcome {
    let message = match &entry.input {
        ActionInput::Commit(c) => Some(c.message.clone()),
        _ => None,
    };
    UndoOutcome {
        action: entry.action.clone(),
        kind: entry.kind,
        branch,
        message,
    }
}

/// 넣기·빼기 되돌리기와 다시 하기: index가 `from`의 트리 그대로일 때만 `to`의 트리로 바꾼다.
/// 작업 폴더는 그대로다. 앱 밖에서 `git add` 등으로 index가 바뀌었으면 하지 않는다.
fn restore_index(
    git: &dyn GitBackend,
    repo: &RepoPath,
    now: &UndoState,
    from: &UndoState,
    to: &UndoState,
) -> Result<()> {
    if now.index_tree.is_none() || now.index_tree != from.index_tree {
        return Err(CoreError::UndoStale);
    }
    let tree = to
        .index_tree
        .as_deref()
        .and_then(Oid::parse)
        .ok_or(CoreError::NothingToUndo)?;
    git.run(repo, &GitOp::ReadTree { tree })?;
    Ok(())
}

fn stage_all(entry: &Entry) -> bool {
    matches!(&entry.input, ActionInput::Commit(c) if c.stage_all)
}

/// 버전 저장 되돌리기: 브랜치를 이전 버전으로(고친 파일은 그대로). 올린 버전은 바꾸지 않는다.
fn undo_commit(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let after = entry.after.head_oid().ok_or(CoreError::NothingToUndo)?;
    let branch = entry
        .after
        .branch
        .as_deref()
        .ok_or(CoreError::NothingToUndo)?;
    // upstream이 없으면 `@{upstream}`이 오류를 내므로 status의 upstream 이름으로 찾는다.
    if let Some(upstream) = git.status(repo)?.branch.upstream
        && let Some(oid) = git.resolve_ref(repo, &format!("refs/remotes/{upstream}"))?
        && git.is_ancestor(repo, after.as_str(), oid.as_str())?
    {
        return Err(CoreError::UndoPushed);
    }
    // 되돌리기 전에 지금 버전을 백업 ref로 살려 둔다. 못 남기면 되돌리지 않는다.
    backup::backup_oid(git, repo, &after, &format!("undo-{branch}"))?;
    match entry.before.head_oid() {
        Some(before) => {
            git.run(repo, &GitOp::ResetSoft { oid: before })?;
        }
        // 첫 버전이었다: 브랜치 ref를 지우면 버전이 없는 상태로 돌아간다(index는 그대로).
        None => git.delete_ref(repo, &format!("refs/heads/{branch}"), &RefExpect::Is(after))?,
    }
    if stage_all(entry)
        && let Some(tree) = entry.before.index_tree.as_deref().and_then(Oid::parse)
    {
        git.run(repo, &GitOp::ReadTree { tree })?;
    }
    Ok(())
}

/// 다시 하기: 액션을 다시 실행하지 않고 되돌리기 전의 버전으로 브랜치를 되살린다(D-065).
fn redo_commit(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let after = entry.after.head_oid().ok_or(CoreError::NothingToUndo)?;
    let branch = entry
        .after
        .branch
        .as_deref()
        .ok_or(CoreError::NothingToUndo)?;
    let expect = match entry.before.head_oid() {
        Some(before) => RefExpect::Is(before),
        None => RefExpect::Absent,
    };
    git.update_ref(repo, &format!("refs/heads/{branch}"), &after, &expect)?;
    if stage_all(entry)
        && let Some(tree) = entry.after.index_tree.as_deref().and_then(Oid::parse)
    {
        git.run(repo, &GitOp::ReadTree { tree })?;
    }
    Ok(())
}

/// 갈아타기 되돌리기: 같은 맡겨 두는 흐름으로 이전 브랜치로 돌아간다. 지금 고친 파일은 도착했던
/// 브랜치에 맡겨지고, 떠났던 브랜치의 맡긴 파일이 다시 나온다(D-047).
fn undo_switch(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let ActionInput::Switch(input) = &entry.input else {
        return Err(CoreError::NothingToUndo);
    };
    let back = entry
        .before
        .branch
        .clone()
        .ok_or(CoreError::NothingToUndo)?;
    switch_with_parking(
        git,
        repo,
        &SwitchInput {
            branch: back,
            remote: None,
        },
    )?;
    // 원격 브랜치에서 새로 만든 로컬 브랜치는 지운다. 그 위에 맡긴 파일이 생겼으면 남긴다(D-054).
    if input.remote.is_some()
        && let Some(after) = entry.after.head_oid()
        && git
            .resolve_ref(repo, &parked::ref_name(&input.branch))?
            .is_none()
    {
        git.delete_ref(
            repo,
            &format!("refs/heads/{}", input.branch),
            &RefExpect::Is(after),
        )?;
    }
    Ok(())
}

fn redo_switch(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let ActionInput::Switch(input) = &entry.input else {
        return Err(CoreError::NothingToUndo);
    };
    // 되돌릴 때 로컬 브랜치를 남겼으면 원격에서 다시 만들지 않고 그 브랜치로 간다.
    let local_exists = git
        .resolve_ref(repo, &format!("refs/heads/{}", input.branch))?
        .is_some();
    let input = SwitchInput {
        branch: input.branch.clone(),
        remote: input.remote.clone().filter(|_| !local_exists),
    };
    switch_with_parking(git, repo, &input).map(|_| ())
}

/// 새 브랜치 되돌리기: 이전 브랜치로 돌아가고(고친 파일은 따라옴) 만든 브랜치를 지운다.
fn undo_create(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let ActionInput::BranchCreate(input) = &entry.input else {
        return Err(CoreError::NothingToUndo);
    };
    let back = entry
        .before
        .branch
        .clone()
        .ok_or(CoreError::NothingToUndo)?;
    // 버전이 없는 저장소의 새 브랜치는 ref가 없어 되돌릴 것이 없다.
    let created = entry.after.head_oid().ok_or(CoreError::NothingToUndo)?;
    git.run(repo, &GitOp::Switch { branch: back })?;
    git.delete_ref(
        repo,
        &format!("refs/heads/{}", input.name.trim()),
        &RefExpect::Is(created),
    )?;
    Ok(())
}

fn redo_create(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let ActionInput::BranchCreate(input) = &entry.input else {
        return Err(CoreError::NothingToUndo);
    };
    git.run(
        repo,
        &GitOp::CreateAndSwitch {
            name: input.name.trim().to_string(),
        },
    )
    .map(|_| ())
    .map_err(Into::into)
}
