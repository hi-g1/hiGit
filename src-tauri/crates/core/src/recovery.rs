//! 작업 내역 모으기(D-085). 다섯 원천을 최신 먼저 200개로 합친다. 모든 원천은 믿지 않는 입력이다.

use std::collections::HashMap;

use git::{GitBackend, RepoPath};

use crate::Result;
use crate::undo::journal::{Body, Journal};
use crate::view::{RecoveryItem, RecoverySource, RestoreKind};

const MAX_ITEMS: usize = 200;
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
    for (body, verified) in lines {
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
        if let Some(o) = &oid
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
            restore: kind_of(git, repo, &r.oid),
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
    Ok(items)
}
