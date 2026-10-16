//! 작업 내역 모으기(D-085). 다섯 원천을 최신 먼저 200개로 합친다. 모든 원천은 믿지 않는 입력이다.

use std::collections::HashMap;

use git::{GitBackend, RepoPath};

use crate::Result;
use crate::undo::journal::{Body, Journal};
use crate::view::{RecoveryItem, RecoverySource, RestoreKind};

const MAX_ITEMS: usize = 200;
/// 서명이 맞지 않는 journal 줄(받은 저장소, 다른 앱 데이터)은 이만큼만 보인다.
const MAX_UNVERIFIED: usize = 50;
/// 같은 버전·이 초 안의 reflog 줄은 앱 작업 줄에 합친다.
const SAME_MOMENT: i64 = 2;

/// stash 모양(부모 둘 이상, 둘째 부모 메시지가 `index on `)이면 꺼낼 것, 아니면 버전.
pub(crate) fn kind_of(git: &dyn GitBackend, repo: &RepoPath, oid: &str) -> RestoreKind {
    let second = git
        .log(
            repo,
            &git::LogQuery {
                skip: 0,
                limit: 1,
                rev: Some(format!("{oid}^2")),
            },
        )
        .ok()
        .and_then(|c| c.into_iter().next());
    match second {
        Some(c) if c.summary.starts_with("index on ") => RestoreKind::Changes,
        _ => RestoreKind::Version,
    }
}

pub(crate) fn collect(
    git: &dyn GitBackend,
    repo: &RepoPath,
    journal: &Journal,
) -> Result<Vec<RecoveryItem>> {
    let reflog = git.reflog(repo, MAX_ITEMS as u32)?;
    let mut summaries: HashMap<String, String> = HashMap::new();
    for r in &reflog {
        summaries
            .entry(r.oid.to_string())
            .or_insert_with(|| r.summary.clone());
    }
    let mut items = Vec::new();

    // 앱 작업. 되돌린 줄은 표시한다(다시 하면 표시를 지운다).
    let lines = journal.read_all();
    let mut undone: HashMap<String, bool> = HashMap::new();
    for (body, _) in &lines {
        match body {
            Body::Undo { target, .. } => {
                undone.insert(target.clone(), true);
            }
            Body::Redo { target, .. } => {
                undone.insert(target.clone(), false);
            }
            _ => {}
        }
    }
    let mut journal_marks: Vec<(String, i64)> = Vec::new();
    // 서명이 맞지 않는 줄은 최근 것만 남긴다(파일 순서가 옛 것부터).
    let unverified = lines.iter().filter(|(_, v)| !v).count();
    let mut skip_unverified = unverified.saturating_sub(MAX_UNVERIFIED);
    for (body, verified) in lines {
        if !verified && skip_unverified > 0 {
            skip_unverified -= 1;
            continue;
        }
        let Body::Action {
            id,
            at,
            action,
            before,
            after,
            ..
        } = body
        else {
            continue;
        };
        let oid = after.head.clone();
        // HEAD를 옮긴 앱 작업만 같은 순간의 reflog 줄을 대신한다(임시 보관 등은 HEAD 그대로).
        // 서명이 맞지 않는 줄은 git의 기록을 가리지 못한다.
        if let Some(o) = &oid
            && verified
            && before.head != after.head
        {
            journal_marks.push((o.clone(), at));
        }
        items.push(RecoveryItem {
            id: format!("j:{id}"),
            at,
            source: RecoverySource::Journal,
            action: Some(action),
            label: after.branch.clone().unwrap_or_default(),
            summary: oid
                .as_ref()
                .and_then(|o| summaries.get(o).cloned())
                .unwrap_or_default(),
            restore: if verified && oid.is_some() {
                RestoreKind::Version
            } else {
                RestoreKind::None
            },
            oid,
            refname: None,
            undone: undone.get(&id).copied().unwrap_or(false),
        });
    }

    // git의 HEAD 기록. 앱 작업과 같은 순간이면 뺀다.
    for (i, r) in reflog.iter().enumerate() {
        let oid = r.oid.to_string();
        let dup = journal_marks
            .iter()
            .any(|(o, at)| *o == oid && (at - r.at).abs() <= SAME_MOMENT);
        if dup {
            continue;
        }
        items.push(RecoveryItem {
            id: format!("r:{i}"),
            at: r.at,
            source: RecoverySource::Reflog,
            action: None,
            label: r.subject.clone(),
            summary: r.summary.clone(),
            oid: Some(oid),
            refname: None,
            restore: RestoreKind::Version,
            undone: false,
        });
    }

    // 백업 ref. 종류는 이름이 아니라 객체 모양으로 가른다.
    for r in git.list_refs(repo, crate::undo::backup::BACKUP_PREFIX)? {
        let at = crate::undo::backup::created_at(&r.name)
            .and_then(|s| i64::try_from(s).ok())
            .unwrap_or(r.time);
        let label = r.name.rsplit('/').next().unwrap_or("").to_string();
        items.push(RecoveryItem {
            id: format!("b:{}", r.name),
            at,
            source: RecoverySource::Backup,
            action: None,
            label,
            summary: r.summary.clone(),
            // 종류(git log 한 번)는 목록을 자른 뒤 남은 것만 본다.
            restore: RestoreKind::None,
            oid: Some(r.oid.clone()),
            refname: Some(r.name.clone()),
            undone: false,
        });
    }

    // 맡겨 둔 파일.
    for r in git.list_refs(repo, crate::parked::PARKED_PREFIX)? {
        let Some(branch) = crate::parked::branch_of(&r.name) else {
            continue;
        };
        items.push(RecoveryItem {
            id: format!("p:{}", r.name),
            at: r.time,
            source: RecoverySource::Parked,
            action: None,
            label: branch,
            summary: String::new(),
            oid: Some(r.oid.clone()),
            refname: Some(r.name.clone()),
            restore: RestoreKind::Changes,
            undone: false,
        });
    }

    // 임시 보관(맡겨 두기가 잠깐 만든 항목은 뺀다).
    for s in git.stash_list(repo)? {
        let message = crate::actions::stash::stash_message(&s.subject).to_string();
        if message.starts_with(crate::actions::stash::PARKED_PREFIX) {
            continue;
        }
        items.push(RecoveryItem {
            id: format!("s:{}", s.oid),
            at: s.at,
            source: RecoverySource::Stash,
            action: None,
            label: message,
            summary: String::new(),
            oid: Some(s.oid.to_string()),
            refname: None,
            restore: RestoreKind::Changes,
            undone: false,
        });
    }

    items.sort_by_key(|i| std::cmp::Reverse(i.at));
    items.truncate(MAX_ITEMS);
    for item in items
        .iter_mut()
        .filter(|i| i.source == RecoverySource::Backup)
    {
        if let Some(oid) = &item.oid {
            item.restore = kind_of(git, repo, oid);
        }
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use super::*;
    use crate::undo::{JournalKey, UndoKind, UndoState};
    use crate::view::ActionInput;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
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

    /// 버전 둘. (폴더, 첫째, 둘째, 둘째를 만든 reflog 시각)
    fn two() -> (tempfile::TempDir, String, String, i64) {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        git(dir, &["commit", "-q", "--allow-empty", "-m", "처음"]);
        let first = git(dir, &["rev-parse", "HEAD"]);
        git(dir, &["commit", "-q", "--allow-empty", "-m", "둘"]);
        let second = git(dir, &["rev-parse", "HEAD"]);
        let backend = git::CliBackend::new("git", Arc::new(|_| {}));
        let at = backend.reflog(&RepoPath::new(dir), 1).unwrap()[0].at;
        (repo, first, second, at)
    }

    fn line(id: usize, at: i64, before: &str, after: &str) -> Body {
        let state = |h: &str| UndoState {
            head: Some(h.into()),
            branch: Some("main".into()),
            ..Default::default()
        };
        Body::Action {
            id: format!("u{id}"),
            at,
            action: "commit".into(),
            input: ActionInput::Init,
            before: state(before),
            after: state(after),
            kind: UndoKind::RestoreRef,
        }
    }

    #[test]
    fn unsigned_lines_do_not_hide_git_history() {
        let (tmp, first, second, at) = two();
        let dir = tmp.path();
        let git_dir = dir.join(".git");
        // 다른 키로 서명한 줄이 같은 버전·같은 순간을 주장한다.
        Journal::at(&git_dir, JournalKey::ephemeral())
            .append(&line(1, at, &first, &second))
            .unwrap();
        let backend = git::CliBackend::new("git", Arc::new(|_| {}));
        let items = collect(
            &backend,
            &RepoPath::new(dir),
            &Journal::at(&git_dir, JournalKey::ephemeral()),
        )
        .unwrap();
        assert!(
            items
                .iter()
                .any(|i| i.source == RecoverySource::Reflog
                    && i.oid.as_deref() == Some(second.as_str())),
            "{items:?}"
        );
        let unsigned = items
            .iter()
            .find(|i| i.source == RecoverySource::Journal)
            .unwrap();
        assert_eq!(unsigned.restore, RestoreKind::None);
    }

    #[test]
    fn only_fifty_unsigned_lines_show() {
        let (tmp, first, second, at) = two();
        let dir = tmp.path();
        let git_dir = dir.join(".git");
        let other = Journal::at(&git_dir, JournalKey::ephemeral());
        for i in 0..80 {
            other
                .append(&line(i, at - 1000 + i as i64, &first, &second))
                .unwrap();
        }
        let backend = git::CliBackend::new("git", Arc::new(|_| {}));
        let items = collect(
            &backend,
            &RepoPath::new(dir),
            &Journal::at(&git_dir, JournalKey::ephemeral()),
        )
        .unwrap();
        let shown: Vec<&RecoveryItem> = items
            .iter()
            .filter(|i| i.source == RecoverySource::Journal)
            .collect();
        assert_eq!(shown.len(), MAX_UNVERIFIED);
        // 최근 줄이 남는다.
        assert!(shown.iter().any(|i| i.id == "j:u79"));
        assert!(!shown.iter().any(|i| i.id == "j:u0"));
    }
}
