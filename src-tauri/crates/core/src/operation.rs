//! 앱이 시작한 복사·되돌리기·기록 정리의 맥락(D-101, D-102). 시작 전에 서명한 표식
//! (`operation.json`)을 쓰고, 멈춘 뒤에는 표식과 git 상태가 서로 맞을 때만 앱이 이어서 다룬다.
//! 하나라도 어긋나면 "다른 도구의 작업"으로 본다.

use git::{GitBackend, Oid, RepoPath};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::marker::{self, Name};
use crate::undo::JournalKey;

/// 표식에 담는 버전 수 상한(기록 정리: 고른 50개 + 그 위 1000개).
const MAX_OIDS: usize = 1050;
/// 표식에 담는 고친 메시지 전체 크기 상한.
const MAX_MESSAGES_BYTES: usize = 24 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OpKind {
    CherryPick,
    Revert,
    Rebase,
}

impl OpKind {
    /// 충돌 해결 화면의 맥락.
    pub(crate) fn context(self) -> crate::view::ConflictContext {
        use crate::view::ConflictContext as C;
        match self {
            OpKind::CherryPick => C::CherryPick,
            OpKind::Revert => C::Revert,
            OpKind::Rebase => C::Rebase,
        }
    }

    pub(crate) fn git(self) -> git::OperationKind {
        match self {
            OpKind::CherryPick => git::OperationKind::CherryPick,
            OpKind::Revert => git::OperationKind::Revert,
            OpKind::Rebase => git::OperationKind::Rebase,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpMarker {
    pub kind: OpKind,
    /// 시작할 때의 브랜치.
    pub branch: String,
    /// 시작할 때의 지금 위치.
    pub head_before: String,
    /// 기록 정리의 바탕(정리할 첫 버전의 부모).
    #[serde(default)]
    pub base: Option<String>,
    /// 복사·되돌리기는 고른 버전 하나, 기록 정리는 할 일 목록에 넣은 버전 모두(아래부터).
    pub oids: Vec<String>,
    pub started_at: i64,
    /// 기록 정리에서 메시지를 고칠 버전(옛 해시)과 새 메시지. `edit`으로 멈추면 이것으로 고친다.
    /// 껐다 켜도 이어 가도록 표식에 담는다(D-100).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<(String, String)>,
}

impl OpMarker {
    /// 표식은 서명이 맞아도 믿지 않는 입력처럼 형식을 본다(D-034).
    fn valid(&self) -> bool {
        let oid_ok = |s: &str| Oid::parse(s).is_some();
        crate::actions::names::validate_ref_like(&self.branch).is_ok()
            && oid_ok(&self.head_before)
            && self.base.as_deref().is_none_or(oid_ok)
            && !self.oids.is_empty()
            && self.oids.len() <= MAX_OIDS
            && self.oids.iter().all(|o| oid_ok(o))
            && (self.kind == OpKind::Rebase) == self.base.is_some()
            && self.messages.iter().all(|(o, _)| oid_ok(o))
            && self.messages.iter().map(|(_, m)| m.len()).sum::<usize>() <= MAX_MESSAGES_BYTES
    }

    /// `edit`으로 멈춘 버전(옛 해시)의 새 메시지. git은 할 일 목록의 해시를 줄여 적을 수 있어
    /// 앞부분(7자 이상)으로 찾는다.
    pub(crate) fn message_for(&self, oid: &str) -> Option<&str> {
        if oid.len() < 7 || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        self.messages
            .iter()
            .find(|(o, _)| o.starts_with(oid))
            .map(|(_, m)| m.as_str())
    }
}

pub(crate) fn write(git_dir: &std::path::Path, key: &JournalKey, op: &OpMarker) -> Result<()> {
    marker::write(git_dir, Name::Operation, key, op)
}

/// 서명과 형식이 맞는 표식(git 상태와 맞는지는 `verify_state`).
pub(crate) fn read(git_dir: &std::path::Path, key: &JournalKey) -> Option<OpMarker> {
    marker::read::<OpMarker>(git_dir, Name::Operation, key).filter(OpMarker::valid)
}

pub(crate) fn clear(git_dir: &std::path::Path) {
    marker::clear(git_dir, Name::Operation);
}

/// 표식이 지금 git 상태와 맞는지. 복사·되돌리기는 같은 브랜치에서 `CHERRY_PICK_HEAD`·`REVERT_HEAD`가
/// 고른 버전이고, 기록 정리는 `rebase-merge/`의 브랜치·시작 위치·바탕이 표식과 같아야 한다.
pub(crate) fn verify_state(git: &dyn GitBackend, repo: &RepoPath, op: &OpMarker) -> Result<bool> {
    let first = op.oids.first().and_then(|o| Oid::parse(o));
    let head_is = |name: &str| -> Result<bool> {
        let head = git.resolve_ref(repo, name)?;
        Ok(head.is_some() && head == first)
    };
    match op.kind {
        OpKind::CherryPick | OpKind::Revert => {
            let pick_head = if op.kind == OpKind::CherryPick {
                "CHERRY_PICK_HEAD"
            } else {
                "REVERT_HEAD"
            };
            Ok(
                git.current_branch(repo)?.as_deref() == Some(op.branch.as_str())
                    && head_is(pick_head)?,
            )
        }
        OpKind::Rebase => {
            let Some(state) = git.rebase_state(repo)? else {
                return Ok(false);
            };
            Ok(state.head_name == format!("refs/heads/{}", op.branch)
                && state.orig_head.as_ref().map(Oid::as_str) == Some(op.head_before.as_str())
                && state.onto.as_ref().map(Oid::as_str) == op.base.as_deref())
        }
    }
}

/// 표식과 맞는 앱의 작업이 멈춰 있으면 그 표식.
pub(crate) fn current(
    git: &dyn GitBackend,
    repo: &RepoPath,
    key: &JournalKey,
) -> Result<Option<OpMarker>> {
    let Some(op) = read(&git.git_dir(repo)?, key) else {
        return Ok(None);
    };
    Ok(verify_state(git, repo, &op)?.then_some(op))
}

/// 남은 표식을 정리한다: 표식은 있는데 멈춘 작업이 하나도 없으면 지운다. 앱 작업이 돌지 않을
/// 때만 부른다(조회는 지우지 않는다, D-102).
pub(crate) fn tidy(git: &dyn GitBackend, repo: &RepoPath) -> Result<()> {
    let git_dir = git.git_dir(repo)?;
    if marker::exists(&git_dir, Name::Operation)
        && crate::conflict::other_operation(&git_dir).is_none()
    {
        clear(&git_dir);
    }
    Ok(())
}
