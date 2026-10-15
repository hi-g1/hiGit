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
    let (mut applied, _) = stacks(journal.read());
    let entry = applied.pop().ok_or(CoreError::NothingToUndo)?;
    // 합치기·충돌 중에는 되돌리지 않는다(되돌리기가 그 작업을 망친다).
    refuse_during_operation(git, repo)?;
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
        // 올리기·가져오기·동기화는 되돌리지 않는다. 올린 것은 기록을 바꾸지 않는다(D-042, D-073).
        UndoKind::Inverse if entry.action == "pull" => return Err(CoreError::NothingToUndo),
        UndoKind::Inverse => return Err(CoreError::UndoPushed),
        _ => return Err(CoreError::NothingToUndo),
    };
    let reverted = matches!(undid, Undid::Reverted);
    if !reverted {
        restore_refs(git, repo, &entry.after.refs, &entry.before.refs)?;
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

pub(crate) fn redo(
    git: &dyn GitBackend,
    repo: &RepoPath,
    journal: &Journal,
) -> Result<UndoOutcome> {
    let (_, mut undone) = stacks(journal.read());
    let entry = undone.pop().ok_or(CoreError::NothingToUndo)?;
    refuse_during_operation(git, repo)?;
    let now = UndoState::capture(git, repo)?;
    if now.head != entry.before.head || now.branch != entry.before.branch {
        return Err(CoreError::UndoStale);
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

/// 올린 버전이면 참. 올린 버전은 기록을 바꾸는 방법으로 되돌리지 않는다(D-042).
fn is_pushed(git: &dyn GitBackend, repo: &RepoPath, oid: &Oid) -> Result<bool> {
    // upstream이 없으면 `@{upstream}`이 오류를 내므로 status의 upstream 이름으로 찾는다.
    if let Some(upstream) = git.status(repo)?.branch.upstream
        && let Some(remote) = git.resolve_ref(repo, &format!("refs/remotes/{upstream}"))?
    {
        return Ok(git.is_ancestor(repo, oid.as_str(), remote.as_str())?);
    }
    Ok(false)
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
    let staged = git
        .status(repo)?
        .entries
        .iter()
        .any(|e| !matches!(e.index, git::Change::Unmodified | git::Change::Untracked));
    if staged {
        return Err(CoreError::StagedChanges);
    }
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
