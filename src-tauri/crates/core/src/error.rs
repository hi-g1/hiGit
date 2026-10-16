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

    /// 원격 저장소가 없다.
    #[error("원격 저장소가 없습니다")]
    NoRemote,

    /// 고친 것을 버렸지만 백업 ref를 남기지 못했다. 버린 내용은 임시 보관 목록에 남아 있다.
    #[error("백업을 남기지 못했습니다")]
    BackupFailed,

    /// 충돌 파일이 화면이 본 것과 달라졌다(곳 수가 다름 등). 다시 읽고 골라야 한다.
    #[error("충돌 파일이 바뀌었습니다")]
    ConflictChanged,

    /// 이번 버전에 넣은 파일이 있어 하지 않았다(합치기·되돌리는 새 버전에 섞이지 않게).
    #[error("이번 버전에 넣은 파일이 있습니다")]
    StagedChanges,

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

    /// 바뀌거나 빠질 버전 중 올린 것이 있다(올린 기록은 바꾸지 않는다, D-097).
    #[error("이미 올린 버전이 있습니다")]
    PublishedHistory,

    /// 추적 중인 파일을 고쳤거나 넣은 파일이 있다.
    #[error("고친 파일이 있습니다")]
    HasChanges,

    /// 같은 이름의 태그가 있다.
    #[error("같은 이름의 태그가 있습니다")]
    TagExists,

    /// 지금 브랜치나 기본 브랜치라 지우지 않는다.
    #[error("지울 수 없는 브랜치입니다")]
    BranchProtected,

    /// 주석 태그(태그 객체)라 지우지 않는다(D-104).
    #[error("주석 태그입니다")]
    AnnotatedTag,

    /// 버전에 남길 이름·이메일이 없다(git을 부르기 전에 알아챔).
    #[error("이름과 이메일이 없습니다")]
    IdentityMissing,

    /// 하려는 결과가 지금과 같다(빈 복사·이미 취소된 변경, 바뀐 것 없는 고치기).
    #[error("바뀌는 것이 없습니다")]
    NoChange,

    /// 복제 주소가 규칙에 맞지 않는다(D-035, D-115).
    #[error("복제할 수 없는 주소입니다")]
    CloneUrlInvalid,

    /// 복제할 폴더 이름이 규칙에 맞지 않는다.
    #[error("쓸 수 없는 폴더 이름입니다")]
    CloneNameInvalid,

    /// 복제할 폴더가 이미 있고 비어 있지 않다(또는 파일·링크다).
    #[error("같은 이름의 폴더가 이미 있습니다")]
    CloneTargetExists,

    /// git이 보안 수정 기준보다 낮아 복제하지 않았다(사람이 알고 진행하면 된다, D-050).
    #[error("git이 보안 수정 기준보다 낮습니다")]
    GitOutdated,

    #[error("미리보기를 먼저 확인해야 합니다")]
    PreviewRequired,

    #[error("미리보기가 만료되었거나 실행하려는 내용과 다릅니다")]
    PreviewMismatch,

    #[error("git 작업이 실패했습니다")]
    Git(#[from] GitError),

    /// GitHub 연결·API 실패(D-109). 설명은 서버·전송이 준 짧은 글(비밀은 지운 뒤).
    #[error("GitHub 작업이 실패했습니다")]
    Github {
        kind: GithubFailure,
        detail: Option<String>,
    },
}

/// GitHub 연결·API 실패 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GithubFailure {
    /// client id가 없어 연결할 수 없다(준비 안 됨).
    NotConfigured,
    /// 연결되어 있지 않다.
    NotConnected,
    /// 토큰을 더는 쓸 수 없어 연결을 끊었다(401).
    Expired,
    Forbidden,
    RateLimited,
    /// 조직의 SSO 승인이 필요하다.
    SsoRequired,
    NotFound,
    Moved,
    Unreachable,
    /// GitHub 쪽 문제(5xx).
    Unavailable,
    BadResponse,
    /// 키체인을 쓰지 못했다(잠김·거절·오류).
    Keychain,
    Cancelled,
    PrAlreadyExists,
    PrNoDifference,
    PrInvalid,
    /// 지금 브랜치가 올리는 원격이 github.com이 아니다.
    NotGithubRemote,
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
            CoreError::Github { detail, .. } => detail.clone(),
            _ => None,
        }
    }
}
