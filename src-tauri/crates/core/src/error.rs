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

    /// 고친 파일이 도착할 브랜치와 겹쳐 가져가지 않았다(D-195). 갈아타지 않았고 작업 폴더는 그대로다.
    /// 겹친 파일(5개까지, 줄마다 하나).
    #[error("고친 파일이 겹쳐 가져가지 않았습니다")]
    CarryBlocked(String),

    /// 고정 한도(프로젝트 10개, 브랜치 10개, D-203).
    #[error("더 고정할 수 없습니다")]
    PinLimit,

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

    /// 폴더 오른쪽 클릭 메뉴를 만들거나 지우지 못했다(D-214). 내용은 io 오류 종류.
    #[error("오른쪽 클릭 메뉴를 바꾸지 못했습니다")]
    ShellMenu(String),

    /// 화면이 액션과 맞지 않는 입력 변형을 보냈다(화면 버그).
    #[error("액션에 맞지 않는 입력입니다: {0}")]
    InputMismatch(&'static str),

    /// 바뀌거나 빠질 버전 중 올린 것이 있다(올린 기록은 바꾸지 않는다, D-097).
    #[error("이미 올린 버전이 있습니다")]
    PublishedHistory,

    /// 설정 "올린 버전도 바꾸기"가 꺼져 있어 덮어쓰며 올리지 않는다(D-127).
    #[error("덮어쓰며 올리기가 꺼져 있습니다")]
    ForcePushDisabled,

    /// 원격 기본 브랜치는 덮어쓰거나 지우지 않는다(D-128, D-130).
    #[error("원격 기본 브랜치입니다")]
    ForcePushProtected,

    /// 원격에 내 기록에 없는 버전이 없어 덮어쓸 것이 없다(그냥 올리면 된다).
    #[error("덮어쓸 것이 없습니다")]
    NothingToOverwrite,

    /// 원격 기본 브랜치는 지우거나 이름을 바꾸지 않는다(D-130).
    #[error("원격 기본 브랜치입니다")]
    RemoteBranchProtected,

    /// GitHub에 올리기: 이미 원격이 있다(D-132).
    #[error("이미 원격이 있습니다")]
    PublishHasRemote,

    /// GitHub에 올리기: 저장한 버전이 없다.
    #[error("저장한 버전이 없습니다")]
    PublishNoVersions,

    /// 원격 기본 브랜치로 보여 이름을 똑같이 입력해야 한다(D-183).
    #[error("브랜치 이름이 맞지 않습니다")]
    NameConfirmMismatch,

    /// 원격의 같은 이름 태그가 내 태그와 다른 버전이다(D-182).
    #[error("원격 태그가 다른 버전입니다")]
    RemoteTagDiffers,

    /// 원격에 만들려는 브랜치 이름이 이미 있다(이름 바꾸기, D-130).
    #[error("원격에 같은 이름이 있습니다")]
    RemoteNameTaken,

    /// 새 이름으로는 올렸지만 옛 이름을 지우지 못했다(D-130). 원문은 "자세히 보기".
    #[error("새 이름으로 올렸지만 옛 이름은 남았습니다")]
    RemoteRenamePartial(String),

    /// 올린 버전의 기록을 바꿔 원격과 갈라졌다. 원격의 옛 버전을 다시 들여오지 않게 멈춘다(D-129).
    #[error("올린 기록을 바꿔 원격과 갈라졌습니다")]
    RewrotePublished,

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

    /// 이름·이메일이 규칙에 맞지 않는다(D-125). 값은 담지 않는다.
    #[error("이름과 이메일을 확인해 주세요")]
    IdentityInvalid,

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
    /// 아직 올리지 않은 버전이 있거나 올린 적이 없다.
    NotPushed,
    /// head와 대상이 같거나 head가 기본 브랜치다.
    PrSameBranch,
    /// 같은 이름의 저장소가 이미 있다(GitHub에 올리기, D-132).
    RepoExists,
    /// 저장소 만들기 입력을 GitHub가 받지 않았다(그 밖의 422).
    RepoInvalid,
    /// 지금은 합칠 수 없다(405, 허용되지 않은 방법, D-179).
    PrNotMergeable,
    /// 화면이 본 뒤 새 버전이 올라왔다(409, D-179).
    PrHeadChanged,
    /// 열린(닫기)·닫힌(다시 열기) 검토 요청이 아니거나 다시 열 수 없다(422, D-180).
    PrNotOpen,
    /// 이 조직에 저장소를 만들 수 없다(앱 접근 제한·권한 없음, D-184).
    OrgForbidden,
    /// 포크가 아직 준비되지 않았다(다시 누르면 이어서 된다, D-181).
    ForkNotReady,
    /// 원격 `fork`가 이미 다른 주소를 가리킨다.
    ForkRemoteTaken,
    /// 포크의 원래 저장소가 `origin`이 아니다.
    ForkParentMismatch,
}

impl CoreError {
    /// "자세히 보기"에 보여줄 원문. 없으면 `None`.
    pub fn detail(&self) -> Option<String> {
        match self {
            CoreError::Git(e) => Some(e.detail()),
            CoreError::FolderNotFound(e) => Some(e.to_string()),
            CoreError::GitTooOld { found } => Some(format!("git version {found}")),
            CoreError::WatchUnavailable(reason) => Some(reason.clone()),
            CoreError::ShellMenu(detail) => Some(detail.clone()),
            CoreError::ParkBlocked(detail) => Some(detail.clone()),
            CoreError::CarryBlocked(detail) if !detail.is_empty() => Some(detail.clone()),
            CoreError::Github { detail, .. } => detail.clone(),
            CoreError::RemoteRenamePartial(detail) => Some(detail.clone()),
            _ => None,
        }
    }
}
