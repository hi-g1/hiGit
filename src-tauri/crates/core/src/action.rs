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

    /// 실행 전에 잡은 상태를 journal에 남기기 전에 고친다(멈췄다 이어가는 작업이 시작 때 상태를
    /// 남길 때). 기본은 그대로.
    fn journal_before(
        &self,
        _ctx: &ExecCtx<'_>,
        _input: &ActionInput,
        captured: crate::undo::UndoState,
    ) -> crate::undo::UndoState {
        captured
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
    /// 마지막 버전을 고쳤다.
    Amended {
        short_oid: String,
        summary: String,
    },
    Reverted {
        short_oid: String,
        summary: String,
    },
    Picked {
        short_oid: String,
        summary: String,
    },
    /// 지금 브랜치를 옛 버전으로 옮겼다. 기록에서 빠진 버전 수.
    Reset {
        dropped: u32,
    },
    /// 기록을 정리했다. 남은·뺀·묶어 합친 버전 수.
    Rewritten {
        kept: u32,
        dropped: u32,
        joined: u32,
    },
    /// 앱이 시작한 작업이 충돌·hook으로 멈췄다(표식이 남아 있다, D-101).
    OperationConflicts {
        context: crate::view::ConflictContext,
        paths: Vec<String>,
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
    /// 합치기를 마쳤다.
    ConflictFinished,
    /// 합치기를 그만뒀다.
    ConflictAborted,
    /// 임시 보관했다.
    Stashed,
    /// 보관해 둔 것을 꺼냈다.
    TookOut,
    /// 꺼내다 겹친 맡긴 파일을 충돌 해결 화면에서 풀었다(고친 파일로 남음, D-083).
    ParkedResolved,
    /// 고친 것을 버렸다(버린 내용은 백업 ref에).
    Discarded {
        count: u32,
    },
    /// 작업 내역의 버전으로 되돌렸다.
    Restored,
    /// 버려진 잠금 파일을 지웠다.
    LocksCleared {
        count: u32,
    },
    /// 남은 맡긴 항목을 맡긴 ref로 옮겼다.
    ParkedRecovered {
        count: u32,
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
            Outcome::MergeConflicts { .. } | Outcome::OperationConflicts { .. } => {
                JournalMode::Skip
            }
            Outcome::Synced(r) if r.merge_started => JournalMode::Skip,
            // 꺼내기는 HEAD를 바꾸지 않아 앞선 되돌리기를 막지 않는다.
            Outcome::TookOut => JournalMode::Skip,
            // HEAD·브랜치를 바꾸지 않는다. 경계를 남기면 앞선 되돌리기가 까닭 없이 막힌다(C13).
            Outcome::LocksCleared { .. } | Outcome::ParkedRecovered { .. } => JournalMode::Skip,
            // 맡긴 ref를 지웠다. 그 앞의 갈아타기를 되돌리면 맡긴 파일이 엉키므로 경계를 둔다.
            Outcome::ParkedResolved => JournalMode::Barrier,
            _ => JournalMode::Journal,
        }
    }
}

/// 액션이 실행될 때 받는 것.
pub struct ExecCtx<'a> {
    pub git: &'a dyn GitBackend,
    pub state: &'a RepoState,
    /// 앱이 시작한 작업의 표식을 서명하는 키(D-102). 앱에서는 journal 키와 같다.
    pub key: &'a crate::undo::JournalKey,
}
