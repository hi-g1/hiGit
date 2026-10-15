use git::{GitBackend, RepoPath};

use crate::view::ActionInput;
use crate::{RepoState, Result};

/// 상태를 바꾸는 기능 하나(D-010). 조회는 Action이 아니라 Query다.
pub trait Action: Send + Sync {
    fn meta(&self) -> &ActionMeta;

    /// 지금 쓸 수 있는지. 못 쓰면 화면이 쉬운 말로 보여줄 이유.
    fn available(&self, state: &RepoState) -> Availability;

    /// 실행하면 무엇이 바뀌는지. 되돌릴 수 없는 액션은 실행기가 이 단계를 강제한다(D-043).
    fn preview(&self, _ctx: &ExecCtx<'_>, _input: &ActionInput) -> Result<Preview> {
        Ok(Preview::default())
    }

    /// `input`의 변형이 이 액션 것이 아니면 [`crate::CoreError::InputMismatch`].
    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMeta {
    pub id: &'static str,
    /// 용어표 id. 쉬운 이름·원래 이름·설명은 glossary에서 온다(D-006).
    pub glossary_id: &'static str,
    pub risk: Risk,
    /// 되돌리는 방법(D-042). `Irreversible`이면 실행 전에 확인을 받는다(D-043).
    pub undo: crate::undo::UndoKind,
}

/// 위험도. 순서가 있다: `Safe < Normal < Caution < Danger`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Risk {
    Safe,
    Normal,
    Caution,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Available,
    Unavailable(UnavailableReason),
}

/// 액션을 쓸 수 없는 이유. 문구는 화면이 이 값으로 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableReason {
    AlreadyRepo,
    NotARepo,
}

/// 미리보기 내용. 바뀔 파일·옮겨질 갈래 같은 내용은 caution 액션이 생기는 반복 2에서 채운다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Preview {}

/// 액션 실행 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 폴더가 저장소가 되었다(init).
    RepoCreated(RepoPath),
    Staged {
        count: u32,
    },
    Unstaged {
        count: u32,
    },
    Committed {
        short_oid: String,
        summary: String,
    },
}

/// 액션이 실행될 때 받는 것.
pub struct ExecCtx<'a> {
    pub git: &'a dyn GitBackend,
    pub state: &'a RepoState,
}
