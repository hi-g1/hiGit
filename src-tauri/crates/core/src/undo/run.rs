//! 되돌리기·다시 하기 실행(D-042, D-065). journal의 마지막 줄을 그 액션의 `undo` 종류대로 되돌린다.

use git::{GitBackend, GitOp, Oid, RefExpect, RepoPath};

use super::journal::{self, Body, Journal};
use super::{UndoKind, UndoState, backup};
use crate::actions::switch::{refuse_during_operation, switch_with_parking};
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
    /// 올린 버전이라 되돌리는 새 버전을 만들었다(D-073).
    pub reverted: bool,
}

/// 종류별 되돌리기가 한 일.
enum Undid {
    /// 브랜치·index·ref를 이전 상태로 돌렸다. 다시 하기가 된다.
    Restored,
    /// 올린 버전이라 되돌리는 새 버전을 만들었다. 그 앞 줄은 더 되돌리지 않는다.
    Reverted,
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
    // 합치기·충돌 중에는 되돌리지 않는다(되돌리기가 그 작업을 망친다).
    refuse_during_operation(git, repo)?;
    let (mut applied, _) = stacks(journal.read());
    let entry = applied.pop().ok_or(CoreError::NothingToUndo)?;
    let now = UndoState::capture(git, repo)?;
    // 앱 밖에서 버전이나 브랜치가 바뀌었으면 되돌리지 않는다(복구 화면으로 안내).
    if now.head != entry.after.head || now.branch != entry.after.branch {
        return Err(CoreError::UndoStale);
    }
    let undid = match entry.kind {
        // HEAD는 그대로이고 다른 ref만 옮긴 작업("main에 합치기")은 아래 ref 되돌리기만 한다.
        UndoKind::RestoreRef if entry.before.head == entry.after.head => Undid::Restored,
        UndoKind::RestoreRef => undo_commit(git, repo, &entry)?,
        UndoKind::RestoreTree => undo_tree(git, repo, &entry)?,
        UndoKind::RestoreIndex => {
            restore_index(git, repo, &now, &entry.after, &entry.before)?;
            Undid::Restored
        }
        UndoKind::SwitchBack => {
            undo_switch(git, repo, &entry)?;
            Undid::Restored
        }
        UndoKind::DeleteCreated => {
            undo_create(git, repo, &entry)?;
            Undid::Restored
        }
        UndoKind::Inverse if entry.action == "stash" => {
            undo_stash(git, repo, &entry)?;
            Undid::Restored
        }
        UndoKind::Inverse if entry.action == "ignore" => {
            undo_ignore(git, repo, &now, &entry)?;
            Undid::Restored
        }
        UndoKind::Inverse if entry.action == "branch-rename" => {
            undo_branch_rename(git, repo, &entry)?;
            Undid::Restored
        }
        // 올리기·가져오기·동기화는 되돌리지 않는다. 올린 것은 기록을 바꾸지 않는다(D-042, D-073).
        UndoKind::Inverse if entry.action == "pull" => return Err(CoreError::NothingToUndo),
        UndoKind::Inverse => return Err(CoreError::UndoPushed),
        _ => return Err(CoreError::NothingToUndo),
    };
    let reverted = matches!(undid, Undid::Reverted);
    // 임시 보관 되돌리기는 항목을 꺼내고 목록에서 빼는 것으로 `refs/stash`를, 이름 바꾸기
    // 되돌리기는 이름을 되돌리는 것으로 두 ref를 이미 되돌렸다.
    if !reverted && entry.action != "stash" && entry.action != "branch-rename" {
        restore_refs(git, repo, &entry.after.refs, &entry.before.refs)?;
    }
    if entry.action == "branch-delete" {
        // 지우면서 사라진 upstream 설정을 같은 이름의 원격 브랜치가 하나뿐일 때만 다시 잇는다.
        // 잇지 못해도 브랜치는 되살아났다.
        let _ = reconnect_upstream(git, repo, &entry);
    }
    journal.append(&Body::Undo {
        target: entry.id.clone(),
        at: journal::now(),
    })?;
    if reverted {
        // HEAD가 새 버전이 되었다. 그 앞 줄과 다시 하기는 더 쓰지 않는다.
        journal.append(&Body::Barrier {
            at: journal::now(),
            action: "revert".into(),
        })?;
    }
    let mut result = outcome(&entry, entry.before.branch.clone());
    result.reverted = reverted;
    Ok(result)
}

/// 다시 하면 지금 버전들이 기록에서 빠지는 액션(D-097).
const REWRITES_HISTORY: &[&str] = &["amend", "reset", "rebase-i"];

pub(crate) fn redo(
    git: &dyn GitBackend,
    repo: &RepoPath,
    journal: &Journal,
) -> Result<UndoOutcome> {
    refuse_during_operation(git, repo)?;
    let (_, mut undone) = stacks(journal.read());
    let entry = undone.pop().ok_or(CoreError::NothingToUndo)?;
    let now = UndoState::capture(git, repo)?;
    if now.head != entry.before.head || now.branch != entry.before.branch {
        return Err(CoreError::UndoStale);
    }
    // 기록을 바꾸는 작업을 다시 하면 그사이 올린 버전이 기록에서 빠질 수 있다(D-097).
    if REWRITES_HISTORY.contains(&entry.action.as_str())
        && let (Some(before), Some(after)) = (entry.before.head_oid(), entry.after.head_oid())
    {
        crate::guard::refuse_published(git, repo, &before, &after)?;
    }
    match entry.kind {
        UndoKind::RestoreRef if entry.before.head == entry.after.head => {}
        UndoKind::RestoreRef => redo_commit(git, repo, &entry)?,
        UndoKind::RestoreTree => redo_tree(git, repo, &entry)?,
        UndoKind::RestoreIndex => {
            restore_index(git, repo, &now, &entry.before, &entry.after)?;
        }
        UndoKind::SwitchBack => redo_switch(git, repo, &entry)?,
        UndoKind::DeleteCreated => redo_create(git, repo, &entry)?,
        _ => return Err(CoreError::NothingToUndo),
    }
    restore_refs(git, repo, &entry.before.refs, &entry.after.refs)?;
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
        reverted: false,
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

/// 다른 ref를 `from` 값에서 `to` 값으로 돌린다. 그사이 바뀌었으면(`from`과 다름) 실패한다.
fn restore_refs(
    git: &dyn GitBackend,
    repo: &RepoPath,
    from: &[(String, Option<String>)],
    to: &[(String, Option<String>)],
) -> Result<()> {
    let oid = |v: &Option<String>| v.as_deref().and_then(Oid::parse);
    for (name, target) in to {
        let current = from
            .iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, v)| oid(v));
        let target = oid(target);
        if current == target {
            continue;
        }
        let expect = match &current {
            Some(c) => RefExpect::Is(c.clone()),
            None => RefExpect::Absent,
        };
        match (target, current) {
            (Some(t), _) => git.update_ref(repo, name, &t, &expect)?,
            (None, Some(c)) => git.delete_ref(repo, name, &RefExpect::Is(c))?,
            (None, None) => {}
        }
    }
    Ok(())
}

/// 올린 버전이면 참. 올린 버전은 기록을 바꾸는 방법으로 되돌리지 않는다(D-042). upstream만이
/// 아니라 모든 원격 추적 브랜치를 보고, 판정하지 못하면 올린 것으로 본다(D-097).
fn is_pushed(git: &dyn GitBackend, repo: &RepoPath, oid: &Oid) -> Result<bool> {
    crate::guard::is_published(git, repo, oid)
}

/// 합치기·되살리기 되돌리기: 지금 버전을 백업한 뒤 브랜치와 파일을 이전 버전으로(고친 파일은 둔다).
/// 뒤로 옮기면 빠지는 버전이 이미 올라가 있으면 되돌리는 새 버전으로 한다(D-073).
fn undo_tree(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<Undid> {
    let after = entry.after.head_oid().ok_or(CoreError::NothingToUndo)?;
    let before = entry.before.head_oid().ok_or(CoreError::NothingToUndo)?;
    // 앞으로 옮기는 되돌리기(되살리기를 되돌림)는 기록에서 빠지는 버전이 없다.
    let moving_back = !git.is_ancestor(repo, after.as_str(), before.as_str())?;
    if moving_back && is_pushed(git, repo, &after)? {
        return revert_pushed(git, repo, &after, &before);
    }
    let label = entry.after.branch.as_deref().unwrap_or("HEAD");
    backup::backup_oid(git, repo, &after, &format!("undo-{label}"))?;
    git.run(repo, &GitOp::ResetKeep { oid: before })?;
    Ok(Undid::Restored)
}

/// 올린 버전 `after`의 변경을 취소하는 새 버전을 만든다. `after`가 `before` 바로 위의 버전
/// (합치기 버전이면 첫 부모가 `before`)일 때만 한다. 넣은 파일이 있으면 하지 않는다.
fn revert_pushed(
    git: &dyn GitBackend,
    repo: &RepoPath,
    after: &Oid,
    before: &Oid,
) -> Result<Undid> {
    let first_parent = git.resolve_ref(repo, &format!("{after}^1"))?;
    if first_parent.as_ref() != Some(before) {
        return Err(CoreError::UndoPushed);
    }
    crate::guard::refuse_staged(git, repo)?;
    let merge = git.resolve_ref(repo, &format!("{after}^2"))?.is_some();
    git.run(
        repo,
        &GitOp::Revert {
            oid: after.clone(),
            mainline: merge.then_some(1),
        },
    )?;
    Ok(Undid::Reverted)
}

fn redo_tree(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let after = entry.after.head_oid().ok_or(CoreError::NothingToUndo)?;
    git.run(repo, &GitOp::ResetKeep { oid: after })?;
    Ok(())
}

/// 브랜치 이름 바꾸기 되돌리기: 새 이름이 그때 버전 그대로 있고 옛 이름이 비어 있을 때만 원래
/// 이름으로 되돌리고 맡긴 ref도 옮긴다. upstream은 되살리지 않는다(D-103).
fn undo_branch_rename(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let ActionInput::BranchRename(input) = &entry.input else {
        return Err(CoreError::NothingToUndo);
    };
    let (from, to) = (input.from.trim(), input.to.trim());
    let value = |state: &UndoState, name: &str| {
        state
            .refs
            .iter()
            .find(|(n, _)| n == &format!("refs/heads/{name}"))
            .and_then(|(_, v)| v.as_deref())
            .and_then(Oid::parse)
    };
    let renamed = value(&entry.after, to).ok_or(CoreError::NothingToUndo)?;
    if git.resolve_ref(repo, &format!("refs/heads/{to}"))? != Some(renamed)
        || git
            .resolve_ref(repo, &format!("refs/heads/{from}"))?
            .is_some()
    {
        return Err(CoreError::UndoStale);
    }
    git.run(
        repo,
        &GitOp::BranchRename {
            from: to.to_string(),
            to: from.to_string(),
            force: from.to_lowercase() == to.to_lowercase(),
        },
    )?;
    crate::parked::move_ref(git, repo, to, from)?;
    Ok(())
}

/// 되살린 브랜치를 같은 이름의 원격 추적 브랜치(하나뿐일 때)에 다시 잇는다.
fn reconnect_upstream(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let ActionInput::BranchDelete(input) = &entry.input else {
        return Ok(());
    };
    let name = input.name.trim();
    let suffix = format!("/{name}");
    let remotes = git.remotes(repo)?;
    let matches: Vec<String> = git
        .list_refs(repo, "refs/remotes/")?
        .into_iter()
        .filter_map(|r| r.name.strip_prefix("refs/remotes/").map(str::to_string))
        .filter(|short| {
            short.ends_with(&suffix)
                && remotes
                    .iter()
                    .any(|rm| short == &format!("{}{suffix}", rm.name))
        })
        .collect();
    if let [upstream] = matches.as_slice() {
        git.run(
            repo,
            &GitOp::SetUpstream {
                branch: name.to_string(),
                upstream: upstream.clone(),
            },
        )?;
    }
    Ok(())
}

/// 임시 보관 되돌리기: 그때 만든 항목이 아직 목록 맨 위이고 작업 폴더가 깨끗할 때만 꺼내고 뺀다.
fn undo_stash(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<()> {
    let created = entry
        .after
        .refs
        .iter()
        .find(|(name, _)| name == "refs/stash")
        .and_then(|(_, v)| v.as_deref())
        .and_then(Oid::parse)
        .ok_or(CoreError::NothingToUndo)?;
    if git.resolve_ref(repo, "refs/stash")? != Some(created.clone()) {
        return Err(CoreError::UndoStale);
    }
    git.run(
        repo,
        &GitOp::Unpark {
            oid: created.clone(),
        },
    )?;
    git.run(repo, &GitOp::StashDropOid { oid: created })?;
    Ok(())
}

/// 제외할 파일 되돌리기: 덧붙인 규칙 줄이 `.gitignore` 끝에 그대로 있고 index가 그때 그대로일
/// 때만 그 줄을 지우고, 빼냈던 새 파일을 다시 넣는다. 다시 하기는 없다(액션을 다시 누른다).
fn undo_ignore(
    git: &dyn GitBackend,
    repo: &RepoPath,
    now: &UndoState,
    entry: &Entry,
) -> Result<()> {
    use crate::actions::ignore;
    let ActionInput::Ignore(input) = &entry.input else {
        return Err(CoreError::NothingToUndo);
    };
    if now.index_tree != entry.after.index_tree {
        return Err(CoreError::UndoStale);
    }
    let lines: Vec<String> = input
        .paths
        .iter()
        .map(|p| ignore::rule_line(p))
        .collect::<Result<_>>()?;
    let text = ignore::read_rules(repo.as_path())?;
    let added = ignore::block(&lines, ignore::newline_of(&text));
    let Some(kept) = text.strip_suffix(&added) else {
        return Err(CoreError::UndoStale);
    };
    // 앱이 처음 만든 `.gitignore`였으면(비고 버전에 없음) 빈 파일을 남기지 않고 지운다.
    let tracked = git
        .index_entries(repo)?
        .iter()
        .any(|e| e.path == ".gitignore");
    if kept.is_empty() && !tracked {
        std::fs::remove_file(repo.as_path().join(".gitignore"))
            .map_err(|_| CoreError::InvalidInput("제외할 파일 목록을 바꿀 수 없어요"))?;
    } else {
        ignore::write_rules(repo.as_path(), kept)?;
    }
    if entry.before.index_tree != entry.after.index_tree
        && let Some(tree) = entry.before.index_tree.as_deref().and_then(Oid::parse)
    {
        git.run(repo, &GitOp::ReadTree { tree })?;
    }
    Ok(())
}

fn stage_all(entry: &Entry) -> bool {
    matches!(&entry.input, ActionInput::Commit(c) if c.stage_all)
}

/// 버전 저장 되돌리기: 브랜치를 이전 버전으로(고친 파일은 그대로). 올린 버전은 기록을 바꾸지
/// 않고 되돌리는 새 버전을 만든다(D-073).
fn undo_commit(git: &dyn GitBackend, repo: &RepoPath, entry: &Entry) -> Result<Undid> {
    let after = entry.after.head_oid().ok_or(CoreError::NothingToUndo)?;
    let branch = entry
        .after
        .branch
        .as_deref()
        .ok_or(CoreError::NothingToUndo)?;
    if is_pushed(git, repo, &after)? {
        let before = entry.before.head_oid().ok_or(CoreError::UndoPushed)?;
        return revert_pushed(git, repo, &after, &before);
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
    Ok(Undid::Restored)
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use super::*;
    use crate::undo::JournalKey;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// 버전 둘(a.txt를 고친 두 번째)이 있는 저장소. (폴더, 첫 버전, 둘째 버전)
    fn two_versions() -> (tempfile::TempDir, String, String) {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        git(dir, &["config", "core.autocrlf", "false"]);
        // 앱 쪽 git(되돌리는 새 버전)도 이름이 있어야 한다. CI에는 전역 이름이 없다.
        git(dir, &["config", "user.name", "t"]);
        git(dir, &["config", "user.email", "t@example.com"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        std::fs::write(dir.join("a.txt"), "1\n").unwrap();
        std::fs::write(dir.join("b.txt"), "b\n").unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "1"]);
        let first = git(dir, &["rev-parse", "HEAD"]);
        std::fs::write(dir.join("a.txt"), "2\n").unwrap();
        git(dir, &["commit", "-q", "-am", "2"]);
        let second = git(dir, &["rev-parse", "HEAD"]);
        (repo, first, second)
    }

    fn line(kind: UndoKind, before: UndoState, after: UndoState) -> Body {
        Body::Action {
            id: "1".into(),
            at: 1,
            action: "merge".into(),
            input: ActionInput::Init,
            before,
            after,
            kind,
        }
    }

    fn state(head: &str) -> UndoState {
        UndoState {
            head: Some(head.into()),
            branch: Some("main".into()),
            ..Default::default()
        }
    }

    fn setup(dir: &Path) -> (git::CliBackend, RepoPath, Journal) {
        let backend = git::CliBackend::new("git", Arc::new(|_| {}));
        let repo = RepoPath::new(dir);
        let journal = Journal::at(&dir.join(".git"), JournalKey::ephemeral());
        (backend, repo, journal)
    }

    #[test]
    fn tree_undo_keeps_unrelated_edits_and_redo_comes_back() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        let (backend, repo, journal) = setup(dir);
        journal
            .append(&line(UndoKind::RestoreTree, state(&first), state(&second)))
            .unwrap();
        std::fs::write(dir.join("b.txt"), "b 고침\n").unwrap();
        undo(&backend, &repo, &journal).unwrap();
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), first);
        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "1\n");
        assert_eq!(
            std::fs::read_to_string(dir.join("b.txt")).unwrap(),
            "b 고침\n"
        );
        // 되돌리기 전 버전은 백업 ref로 남는다.
        let backups = git(
            dir,
            &[
                "for-each-ref",
                "--format=%(objectname)",
                "refs/higit/backup/",
            ],
        );
        assert_eq!(backups, second);
        redo(&backend, &repo, &journal).unwrap();
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), second);
        assert_eq!(
            std::fs::read_to_string(dir.join("b.txt")).unwrap(),
            "b 고침\n"
        );
    }

    #[test]
    fn tree_undo_refuses_when_edits_overlap() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        let (backend, repo, journal) = setup(dir);
        journal
            .append(&line(UndoKind::RestoreTree, state(&first), state(&second)))
            .unwrap();
        std::fs::write(dir.join("a.txt"), "내 고침\n").unwrap();
        let err = undo(&backend, &repo, &journal).unwrap_err();
        assert_eq!(
            crate::view::ErrorView::from(&err).code,
            crate::view::ErrorCode::WouldOverwrite
        );
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), second);
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "내 고침\n"
        );
    }

    #[test]
    fn ref_only_undo_moves_the_other_branch_back() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        git(dir, &["branch", "기능", &first]);
        git(dir, &["update-ref", "refs/heads/기능", &second]);
        let (backend, repo, journal) = setup(dir);
        let with_ref = |head: &str, value: &str| UndoState {
            refs: vec![("refs/heads/기능".into(), Some(value.into()))],
            ..state(head)
        };
        journal
            .append(&line(
                UndoKind::RestoreRef,
                with_ref(&second, &first),
                with_ref(&second, &second),
            ))
            .unwrap();
        undo(&backend, &repo, &journal).unwrap();
        assert_eq!(git(dir, &["rev-parse", "refs/heads/기능"]), first);
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), second, "HEAD는 그대로");
        redo(&backend, &repo, &journal).unwrap();
        assert_eq!(git(dir, &["rev-parse", "refs/heads/기능"]), second);
    }

    #[test]
    fn redo_of_a_rewrite_refuses_when_dropped_versions_were_pushed() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        let (backend, repo, journal) = setup(dir);
        // "돌아가기"(second → first)를 되돌린 뒤라 지금은 second다.
        let reset = Body::Action {
            id: "1".into(),
            at: 1,
            action: "reset".into(),
            input: ActionInput::Init,
            before: state(&second),
            after: state(&first),
            kind: UndoKind::RestoreTree,
        };
        journal.append(&reset).unwrap();
        journal
            .append(&Body::Undo {
                target: "1".into(),
                at: 2,
            })
            .unwrap();
        // upstream이 아닌 원격 추적 브랜치여도 올린 것이다.
        git(dir, &["update-ref", "refs/remotes/origin/다른", &second]);
        let err = redo(&backend, &repo, &journal).unwrap_err();
        assert!(matches!(err, CoreError::PublishedHistory), "{err:?}");
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), second);
        git(dir, &["update-ref", "-d", "refs/remotes/origin/다른"]);
        redo(&backend, &repo, &journal).unwrap();
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), first);
    }

    #[test]
    fn undo_treats_any_remote_branch_as_pushed() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        let (backend, repo, journal) = setup(dir);
        git(dir, &["update-ref", "refs/remotes/origin/다른", &second]);
        journal
            .append(&line(UndoKind::RestoreTree, state(&first), state(&second)))
            .unwrap();
        let result = undo(&backend, &repo, &journal).unwrap();
        assert!(result.reverted, "되돌리는 새 버전을 만든다");
        assert_eq!(git(dir, &["rev-parse", "HEAD~1"]), second);
    }

    /// 원격에 올리고 upstream을 건다.
    fn push_to_bare(dir: &Path) -> tempfile::TempDir {
        let remote = tempfile::tempdir().unwrap();
        git(remote.path(), &["init", "-q", "--bare", "-b", "main"]);
        git(
            dir,
            &["remote", "add", "origin", &remote.path().to_string_lossy()],
        );
        git(dir, &["push", "-q", "-u", "origin", "main"]);
        remote
    }

    #[test]
    fn undoing_a_pushed_merge_reverts_against_the_first_parent() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        git(dir, &["switch", "-q", "-c", "기능", &first]);
        std::fs::write(dir.join("c.txt"), "c\n").unwrap();
        git(dir, &["add", "c.txt"]);
        git(dir, &["commit", "-q", "-m", "c"]);
        git(dir, &["switch", "-q", "main"]);
        git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
        let merged = git(dir, &["rev-parse", "HEAD"]);
        let _remote = push_to_bare(dir);
        let (backend, repo, journal) = setup(dir);
        journal
            .append(&line(UndoKind::RestoreTree, state(&second), state(&merged)))
            .unwrap();
        let result = undo(&backend, &repo, &journal).unwrap();
        assert!(result.reverted);
        assert_eq!(git(dir, &["rev-parse", "HEAD~1"]), merged);
        assert!(!dir.join("c.txt").exists(), "합쳐 온 파일이 빠진다");
        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "2\n");
    }

    #[test]
    fn undoing_a_move_back_to_an_older_version_just_moves_forward() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        let _remote = push_to_bare(dir);
        // 되살리기로 첫 버전에 돌아온 상태(올린 버전 위). 되돌리면 앞으로 간다.
        git(dir, &["reset", "-q", "--keep", &first]);
        let (backend, repo, journal) = setup(dir);
        journal
            .append(&line(UndoKind::RestoreTree, state(&second), state(&first)))
            .unwrap();
        let result = undo(&backend, &repo, &journal).unwrap();
        assert!(!result.reverted);
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), second);
    }

    #[test]
    fn no_undo_while_merging() {
        let (tmp, first, second) = two_versions();
        let dir = tmp.path();
        let (backend, repo, journal) = setup(dir);
        journal
            .append(&line(UndoKind::RestoreTree, state(&first), state(&second)))
            .unwrap();
        std::fs::write(dir.join(".git/MERGE_HEAD"), format!("{first}\n")).unwrap();
        let err = undo(&backend, &repo, &journal).unwrap_err();
        assert!(matches!(err, CoreError::OperationInProgress), "{err:?}");
    }
}
