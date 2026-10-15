//! 되돌리기 바탕(D-042, D-043, D-063, D-065). `docs/architecture.md` "되돌리기와 UndoJournal".
//!
//! 실행기가 액션 전후의 상태를 journal에 남기고, 백업 ref로 버전을 살려 둔다.

pub(crate) mod backup;
pub(crate) mod journal;
mod key;
pub(crate) mod run;

use git::{GitBackend, Oid, RepoPath};
use serde::{Deserialize, Serialize};

pub use key::JournalKey;
pub use run::UndoOutcome;

use crate::Result;
use crate::actions::names;

/// 되돌리는 방법. 레지스트리의 액션마다 정한다(`ActionMeta::undo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UndoKind {
    /// 되돌릴 것이 없다(원격 확인, 관리 시작). journal에 남기지 않는다.
    Nothing,
    /// HEAD·브랜치를 이전 버전으로. 고친 파일은 그대로(버전 저장).
    RestoreRef,
    /// index를 이전 트리로(이번 버전에 넣기·빼기, D-072). 충돌 중이라 트리가 없으면 경계 줄.
    RestoreIndex,
    /// 이전 브랜치로 갈아타고 맡겨 둔 파일을 꺼낸다(브랜치 갈아타기).
    SwitchBack,
    /// 브랜치와 파일을 이전 버전으로 옮기되 고친 파일은 둔다(합치기·되살리기, `reset --keep`, D-072).
    RestoreTree,
    /// 만든 ref를 지운다(새 브랜치).
    DeleteCreated,
    /// 기록을 바꾸지 않는 반대 동작(원격에 올린 뒤, revert). 1b에는 없다.
    Inverse,
    /// 되돌릴 수 없다. 실행 전 확인을 받는다(D-043).
    Irreversible,
    /// 되돌릴 수 없지만 확인 없이 실행하고 경계 줄만 남긴다(D-072).
    Barrier,
}

impl UndoKind {
    /// journal에 되돌릴 수 있는 줄로 남기는지.
    pub(crate) fn is_recorded(self) -> bool {
        matches!(
            self,
            UndoKind::RestoreRef
                | UndoKind::RestoreIndex
                | UndoKind::RestoreTree
                | UndoKind::SwitchBack
                | UndoKind::DeleteCreated
                | UndoKind::Inverse
        )
    }

    /// 되돌릴 수 없어서 경계 줄을 남기는지(그 앞 줄은 되돌리지 않는다, D-065).
    pub(crate) fn is_barrier(self) -> bool {
        matches!(self, UndoKind::Irreversible | UndoKind::Barrier)
    }
}

/// 되돌리는 데 필요한 저장소 상태. journal에 그대로 들어간다.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoState {
    pub head: Option<String>,
    pub branch: Option<String>,
    /// (브랜치, 맡긴 stash 해시). 지금은 아무도 채우지 않는다(맡긴 ref는 갈아타기가 직접 다룬다).
    /// 이 값이 든 옛 줄을 읽을 수 있게 남겨 두고 읽을 때 검사한다.
    #[serde(default)]
    pub parked: Vec<(String, String)>,
    /// index 트리. "모두 넣고 저장"을 되돌릴 때 index도 돌린다(D-062).
    #[serde(default)]
    pub index_tree: Option<String>,
    /// 액션이 옮기는 다른 ref의 값(이름, 해시. 없으면 `None`). "main에 합치기" 등이 채운다.
    /// 비어 있으면 쓰지 않는다: 옛 줄을 다시 직렬화해도 같은 글자여야 서명이 맞는다(D-072).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<(String, Option<String>)>,
}

impl UndoState {
    pub(crate) fn capture(git: &dyn GitBackend, repo: &RepoPath) -> Result<Self> {
        Self::capture_with(git, repo, &[])
    }

    /// `tracked`의 ref 값도 함께 담는다.
    pub(crate) fn capture_with(
        git: &dyn GitBackend,
        repo: &RepoPath,
        tracked: &[String],
    ) -> Result<Self> {
        let mut refs = Vec::new();
        for name in tracked {
            refs.push((
                name.clone(),
                git.resolve_ref(repo, name)?.map(|o| o.to_string()),
            ));
        }
        Ok(Self {
            head: git.resolve_ref(repo, "HEAD")?.map(|o| o.to_string()),
            branch: git.current_branch(repo)?,
            parked: Vec::new(),
            index_tree: git.write_tree(repo)?.map(|o| o.to_string()),
            refs,
        })
    }

    /// journal에서 읽은 값은 믿지 않는 입력이다(D-034). 해시와 이름 형식을 확인한다.
    pub(crate) fn is_valid(&self) -> bool {
        let oid_ok = |o: &Option<String>| o.as_deref().is_none_or(|s| Oid::parse(s).is_some());
        let name_ok =
            |n: &str| names::validate_ref_like(n).is_ok() && !n.contains("..") && !n.contains(' ');
        // 되돌리기가 옮길 수 있는 ref는 로컬 브랜치와 stash뿐이다.
        let ref_ok = |n: &str| {
            n == "refs/stash"
                || n.strip_prefix("refs/heads/")
                    .is_some_and(|b| !b.is_empty() && name_ok(b))
        };
        oid_ok(&self.head)
            && oid_ok(&self.index_tree)
            && self.refs.iter().all(|(n, o)| ref_ok(n) && oid_ok(o))
            && self.branch.as_deref().is_none_or(name_ok)
            && self
                .parked
                .iter()
                .all(|(b, o)| name_ok(b) && Oid::parse(o).is_some())
    }

    pub(crate) fn head_oid(&self) -> Option<Oid> {
        self.head.as_deref().and_then(Oid::parse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_validation() {
        let good = UndoState {
            head: Some("a".repeat(40)),
            branch: Some("기능/한글".into()),
            parked: vec![("main".into(), "b".repeat(40))],
            index_tree: None,
            refs: vec![("refs/heads/main".into(), Some("c".repeat(40)))],
        };
        assert!(good.is_valid());
        for bad in [
            UndoState {
                branch: Some("-x".into()),
                ..good.clone()
            },
            UndoState {
                head: Some("zz".into()),
                ..good.clone()
            },
            UndoState {
                parked: vec![("main".into(), "nope".into())],
                ..good.clone()
            },
            UndoState {
                refs: vec![("refs/remotes/origin/main".into(), None)],
                ..good.clone()
            },
            UndoState {
                refs: vec![("refs/heads/-x".into(), None)],
                ..good.clone()
            },
            UndoState {
                refs: vec![("refs/heads/main".into(), Some("zz".into()))],
                ..good.clone()
            },
        ] {
            assert!(!bad.is_valid(), "{bad:?}");
        }
    }
}
