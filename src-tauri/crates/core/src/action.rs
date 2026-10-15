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

    /// HEAD(지금 브랜치가 가리키는 버전)를 바꿀 수 있는지. 거짓이면 `caution` 이상이어도
    /// 실행기가 HEAD 백업 ref를 만들지 않는다(작업 내역에 쓸모없는 줄이 쌓이지 않게, D-072).
    fn moves_head(&self) -> bool {
        true
    }

    /// 실행 전후 값을 journal에 남길 다른 ref(`refs/heads/…`, `refs/stash`). 되돌리기가 이 ref들을
    /// 이전 값으로 돌린다(D-072).
    fn tracked_refs(&self, _input: &ActionInput) -> Vec<String> {
        Vec::new()
    }
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
    /// 브랜치를 갈아탔다(맡겨 두기 결과 포함).
    Switched(crate::actions::SwitchReport),
    BranchCreated {
        name: String,
    },
    /// 원격을 확인했다.
    Fetched,
    /// 가져오기·올리기·동기화 결과.
    Synced(crate::actions::SyncReport),
    /// 제외할 파일에 넣었다.
    Ignored {
        count: u32,
    },
    /// 합쳤다(또는 "main에 합치기"로 앞으로 감았다).
    Merged {
        branch: String,
        fast_forward: bool,
    },
    /// 겹쳐서 합치는 중으로 멈췄다. 상태는 저장소(`MERGE_HEAD`)에 있다.
    MergeConflicts {
        paths: Vec<String>,
    },
}

/// 실행 결과를 작업 목록(journal)에 어떻게 남길지. 액션의 `undo` 종류보다 우선한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalMode {
    /// 액션의 `undo` 종류대로.
    Journal,
    /// 되돌릴 수 없는 상태가 되었다. 경계 줄을 남긴다(D-065).
    Barrier,
    /// 아무것도 남기지 않는다(합치기를 시작만 하고 멈춘 경우 등).
    Skip,
    /// 브랜치·index가 그대로여도 남긴다(`.gitignore`처럼 상태에 담지 않는 것을 바꾼 경우).
    Always,
}

impl Outcome {
    /// 이 결과를 journal에 어떻게 남길지. 합치는 중으로 끝나는 결과들이 쓴다(반복 2).
    pub fn journal_mode(&self) -> JournalMode {
        match self {
            Outcome::Ignored { .. } => JournalMode::Always,
            // 합치기를 시작만 했다. 마치기·그만두기가 따로 기록한다.
            Outcome::MergeConflicts { .. } => JournalMode::Skip,
            _ => JournalMode::Journal,
        }
    }
}

/// 액션이 실행될 때 받는 것.
pub struct ExecCtx<'a> {
    pub git: &'a dyn GitBackend,
    pub state: &'a RepoState,
}
