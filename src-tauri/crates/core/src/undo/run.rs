//! 되돌리기·다시 하기 실행(D-042, D-065). journal의 마지막 줄을 그 액션의 `undo` 종류대로 되돌린다.

use git::{GitBackend, GitOp, Oid, RefExpect, RepoPath};

use super::journal::{self, Body, Journal};
use super::{UndoKind, UndoState, backup};
use crate::view::ActionInput;
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
        // 갈아타기·새 브랜치 되돌리기는 맡겨 두는 갈아타기 단계에서 채운다.
        _ => return Err(CoreError::NothingToUndo),
    }
    journal.append(&Body::Undo {
        target: entry.id.clone(),
        at: journal::now(),
    })?;
    Ok(outcome(&entry))
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
        _ => return Err(CoreError::NothingToUndo),
    }
    journal.append(&Body::Redo {
        target: entry.id.clone(),
        at: journal::now(),
    })?;
    Ok(outcome(&entry))
}

fn outcome(entry: &Entry) -> UndoOutcome {
    let message = match &entry.input {
        ActionInput::Commit(c) => Some(c.message.clone()),
        _ => None,
    };
    UndoOutcome {
        action: entry.action.clone(),
        kind: entry.kind,
        branch: entry.after.branch.clone(),
        message,
    }
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
