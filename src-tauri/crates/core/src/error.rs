use git::{GitError, GitVersion};

use crate::UnavailableReason;

pub type Result<T> = std::result::Result<T, CoreError>;

/// core가 화면에 넘기는 오류. 한국어 문구는 화면이 종류별로 고르고,
/// git 원문은 [`CoreError::detail`]로 "자세히 보기"에 보여준다.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("git이 설치되어 있지 않습니다")]
    GitMissing,

    #[error("git {found}은 지원하지 않는 버전입니다")]
    GitTooOld { found: GitVersion },

    #[error("폴더를 열 수 없습니다")]
    FolderNotFound(#[source] std::io::Error),

    #[error("Git으로 관리하는 폴더가 아닙니다")]
    NotARepo,

    #[error("알 수 없는 액션입니다: {0}")]
    UnknownAction(String),

    #[error("같은 id의 액션이 이미 있습니다: {0}")]
    DuplicateAction(&'static str),

    #[error("지금은 쓸 수 없는 액션입니다")]
    Unavailable(UnavailableReason),

    /// 입력이 규칙에 맞지 않는다(빈 메시지, 저장소 밖 경로 등). 이유는 "자세히 보기"용.
    #[error("입력이 올바르지 않습니다: {0}")]
    InvalidInput(&'static str),

    /// 되돌릴(다시 할) 작업이 없다.
    #[error("되돌릴 작업이 없습니다")]
    NothingToUndo,

    /// 앱 밖에서 버전이나 브랜치가 바뀌어 되돌리지 않았다.
    #[error("앱 밖에서 바뀌어 되돌리지 않았습니다")]
    UndoStale,

    /// 이미 올린 버전이라 되돌리지 않았다(올린 기록은 바꾸지 않는다, D-042).
    #[error("이미 올린 버전입니다")]
    UndoPushed,

    /// 실행되는 설정이 있는 저장소라 신뢰 확인이 먼저 필요하다(D-039).
    #[error("신뢰 확인이 필요합니다")]
    NeedsTrust,

    /// 합치기·리베이스 중이라 하지 않았다.
    #[error("진행 중인 작업이 있습니다")]
    OperationInProgress,

    /// 갈아타기가 실패한 뒤 맡긴 파일을 원래대로 꺼내지 못했다. 파일은 맡긴 ref에 남아 있다.
    #[error("맡긴 파일을 원래대로 꺼내지 못했습니다")]
    ParkBlocked(String),

    /// 같은 이름(대소문자 무시)의 브랜치가 있다.
    #[error("같은 이름의 브랜치가 있습니다")]
    BranchExists,

    /// journal을 쓰지 못했다(디스크, 권한).
    #[error("작업 기록을 쓰지 못했습니다: {0}")]
    Journal(String),

    /// 프로젝트 열기를 거치지 않은 경로다(D-037).
    #[error("열지 않은 프로젝트입니다")]
    NotOpened,

    /// 같은 저장소에서 다른 작업이 실행 중이다.
    #[error("다른 작업이 실행 중입니다")]
    Busy,

    /// 파일 변경 감시를 시작하지 못했다(감시 한도 등).
    #[error("파일 변경을 감시할 수 없습니다")]
    WatchUnavailable(String),

    /// 화면이 액션과 맞지 않는 입력 변형을 보냈다(화면 버그).
    #[error("액션에 맞지 않는 입력입니다: {0}")]
    InputMismatch(&'static str),

    #[error("미리보기를 먼저 확인해야 합니다")]
    PreviewRequired,

    #[error("미리보기가 만료되었거나 실행하려는 내용과 다릅니다")]
    PreviewMismatch,

    /// 백업·되돌리기가 생기기 전까지 caution 이상은 실행하지 않는다(D-011).
    #[error("아직 지원하지 않는 작업입니다")]
    NotYetSupported,

    #[error("git 작업이 실패했습니다")]
    Git(#[from] GitError),
}

impl CoreError {
    /// "자세히 보기"에 보여줄 원문. 없으면 `None`.
    pub fn detail(&self) -> Option<String> {
        match self {
            CoreError::Git(e) => Some(e.detail()),
            CoreError::FolderNotFound(e) => Some(e.to_string()),
            CoreError::GitTooOld { found } => Some(format!("git version {found}")),
            CoreError::WatchUnavailable(reason) => Some(reason.clone()),
            CoreError::ParkBlocked(detail) => Some(detail.clone()),
            _ => None,
        }
    }
}
