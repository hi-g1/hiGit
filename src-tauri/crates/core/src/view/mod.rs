//! 화면에 보내는 타입(D-019). 바인딩용 derive(serde, specta)는 이 모듈에만 붙인다.
//! `git` 크레이트 타입은 여기서 옮겨 담고, 화면에 그대로 노출하지 않는다.

use std::path::Path;

mod branch;
mod conflict;
mod diff;
mod input;
mod map;
mod merge;
mod recovery;
mod secret;
mod stash;

pub use branch::{BranchListView, BranchNameCheck, BranchRow, NameProblem};
pub use conflict::{
    ConflictContext, ConflictFileKind, ConflictFileSummary, ConflictFileView, ConflictHunkView,
    ConflictSide, ConflictStateView, SideState,
};
pub use diff::{DiffLineView, DiffRequest, FileDiffView, HunkView, LineKindView};
pub use input::{
    ActionInput, BranchCreateInput, CommitInput, ConflictFinishInput, FetchInput, FileChoice,
    FileResolution, HunkChoice, MergeInput, MergeIntoInput, PathsInput, RestoreInput, SwitchInput,
    SyncInput, TakeOutInput, TakeOutSource,
};
pub use map::{HomeMapView, MapCommit};
pub use merge::{MergeHintKind, MergeHintView};
pub use recovery::{RecoveryItem, RecoverySource, RecoveryView, RestoreKind};
pub use secret::{SecretCheckRequest, SecretCheckView, SecretFinding, SecretRule};
pub use stash::{StashItem, StashListView};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ActionMeta, CoreError, Outcome, Risk, UnavailableReason};

/// 앱 시작 시 확인한 환경.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub git_version: String,
    pub git_security: GitSecurity,
    /// 배포판 패키지 git이면 패키지 버전(예: `1:2.34.1-1ubuntu1.17`).
    pub git_package_version: Option<String>,
}

/// Git이 보안 수정 기준을 넘는지(D-050). `outdated`면 화면이 업데이트를 안내한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum GitSecurity {
    Ok,
    Outdated,
    DistroManaged,
}

impl From<crate::git_security::SecurityJudgement> for GitSecurity {
    fn from(j: crate::git_security::SecurityJudgement) -> Self {
        use crate::git_security::SecurityJudgement as J;
        match j {
            J::Ok => GitSecurity::Ok,
            J::Outdated => GitSecurity::Outdated,
            J::DistroManaged => GitSecurity::DistroManaged,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    /// 저장소 최상위 폴더의 절대 경로.
    pub path: String,
    /// 폴더 이름(화면 제목용).
    pub name: String,
}

impl RepoInfo {
    pub(crate) fn from_path(path: &Path) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned()),
        }
    }
}

// ---- 조회: 바뀐 내용 ----

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusView {
    pub branch: BranchView,
    pub files: Vec<ChangedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchView {
    /// 현재 갈래 이름. 특정 저장을 직접 보고 있으면 `null`.
    pub name: Option<String>,
    pub has_commits: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    /// 저장소 기준 경로. 구분자는 OS와 상관없이 `/`.
    pub path: String,
    /// 이름 바꿈의 원래 경로.
    pub orig_path: Option<String>,
    pub change: ChangeKind,
    /// 다음 저장에 넣은(stage) 변경이 있는지.
    pub staged: bool,
}

/// 화면에 보여줄 변경 종류 하나.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    TypeChanged,
    Conflicted,
    Untracked,
}

impl From<git::Status> for StatusView {
    fn from(s: git::Status) -> Self {
        Self {
            branch: BranchView {
                name: s.branch.head,
                has_commits: s.branch.oid.is_some(),
                upstream: s.branch.upstream,
                ahead: s.branch.ahead,
                behind: s.branch.behind,
            },
            files: s.entries.into_iter().map(ChangedFile::from).collect(),
        }
    }
}

impl From<git::StatusEntry> for ChangedFile {
    fn from(e: git::StatusEntry) -> Self {
        use git::{Change as C, EntryKind as K};
        let change = match e.kind {
            K::Untracked => ChangeKind::Untracked,
            K::Unmerged => ChangeKind::Conflicted,
            K::Renamed => ChangeKind::Renamed,
            K::Ordinary => {
                // 작업 폴더 쪽 변화를 먼저 보여주고, 없으면 stage된 쪽.
                let c = if e.worktree != C::Unmodified {
                    e.worktree
                } else {
                    e.index
                };
                match c {
                    C::Added | C::Copied => ChangeKind::Added,
                    C::Deleted => ChangeKind::Deleted,
                    C::TypeChanged => ChangeKind::TypeChanged,
                    C::Renamed => ChangeKind::Renamed,
                    C::Unmerged => ChangeKind::Conflicted,
                    C::Untracked => ChangeKind::Untracked,
                    C::Modified | C::Unmodified => ChangeKind::Modified,
                }
            }
        };
        Self {
            path: e.path,
            orig_path: e.orig_path,
            change,
            staged: !matches!(e.index, git::Change::Unmodified | git::Change::Untracked),
        }
    }
}

// ---- 조회: 저장 기록 ----

#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogQuery {
    pub skip: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogPage {
    pub commits: Vec<CommitRow>,
    /// 이 쪽 뒤에 저장이 더 있는지.
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommitRow {
    pub oid: String,
    pub short_oid: String,
    pub parents: Vec<String>,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    /// 작성 시각(unix 초). TS에서는 number. specta는 i64를 bigint로, f64를 `number | null`로
    /// 내보내므로 TS 타입만 u32(number)로 지정한다. 값은 serde가 숫자 그대로 보낸다.
    #[specta(type = u32)]
    pub author_time: i64,
}

impl From<git::Commit> for CommitRow {
    fn from(c: git::Commit) -> Self {
        Self {
            oid: c.oid,
            short_oid: c.short_oid,
            parents: c.parents,
            summary: c.summary,
            author_name: c.author_name,
            author_email: c.author_email,
            author_time: c.author_time,
        }
    }
}

// ---- 액션 ----

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ActionMetaView {
    pub id: String,
    pub glossary_id: String,
    pub risk: RiskView,
    pub undo: UndoKindView,
}

/// 되돌리기·다시 하기 결과.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub action: String,
    pub kind: UndoKindView,
    pub branch: Option<String>,
    /// 버전 저장을 되돌렸으면 그때 쓴 메시지. 화면이 이름 칸에 다시 채운다.
    pub message: Option<String>,
    /// 올린 버전이라 기록을 지우지 않고 되돌리는 새 버전을 만들었다(D-073). 다시 하기는 없다.
    pub reverted: bool,
}

impl From<crate::undo::UndoOutcome> for UndoResult {
    fn from(o: crate::undo::UndoOutcome) -> Self {
        Self {
            action: o.action,
            kind: o.kind.into(),
            branch: o.branch,
            message: o.message,
            reverted: o.reverted,
        }
    }
}

/// 되돌리는 방법(D-042). 화면은 `irreversible`이면 실행 전에 확인을 받는다(D-043).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum UndoKindView {
    Nothing,
    RestoreRef,
    RestoreIndex,
    SwitchBack,
    RestoreTree,
    DeleteCreated,
    Inverse,
    Irreversible,
    Barrier,
}

impl From<crate::undo::UndoKind> for UndoKindView {
    fn from(k: crate::undo::UndoKind) -> Self {
        use crate::undo::UndoKind as K;
        match k {
            K::Nothing => Self::Nothing,
            K::RestoreRef => Self::RestoreRef,
            K::RestoreIndex => Self::RestoreIndex,
            K::SwitchBack => Self::SwitchBack,
            K::RestoreTree => Self::RestoreTree,
            K::DeleteCreated => Self::DeleteCreated,
            K::Inverse => Self::Inverse,
            K::Irreversible => Self::Irreversible,
            K::Barrier => Self::Barrier,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RiskView {
    Safe,
    Normal,
    Caution,
    Danger,
}

impl From<&ActionMeta> for ActionMetaView {
    fn from(m: &ActionMeta) -> Self {
        Self {
            id: m.id.to_string(),
            glossary_id: m.glossary_id.to_string(),
            risk: match m.risk {
                Risk::Safe => RiskView::Safe,
                Risk::Normal => RiskView::Normal,
                Risk::Caution => RiskView::Caution,
                Risk::Danger => RiskView::Danger,
            },
            undo: m.undo.into(),
        }
    }
}

/// 미리보기와 실행 때 돌려줄 증표.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewView {
    pub token: String,
}

/// 액션 실행 결과.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ActionResult {
    RepoCreated {
        repo: RepoInfo,
    },
    Staged {
        count: u32,
    },
    Unstaged {
        count: u32,
    },
    #[serde(rename_all = "camelCase")]
    Committed {
        short_oid: String,
        summary: String,
    },
    #[serde(rename_all = "camelCase")]
    Switched {
        branch: String,
        /// 떠난 브랜치에 맡긴 파일이 있으면 그 브랜치 이름.
        parked_on: Option<String>,
        unparked: bool,
        unpark_blocked: bool,
    },
    BranchCreated {
        name: String,
    },
    Fetched,
    #[serde(rename_all = "camelCase")]
    Synced {
        received: u32,
        sent: u32,
        merged: bool,
        /// 겹친 파일. 비어 있지 않으면 올리지 않았다.
        overlap: Vec<String>,
        /// 겹쳐서 합치기를 시작했다. 화면은 충돌 해결 화면으로 간다(D-069).
        merge_started: bool,
    },
    Ignored {
        count: u32,
    },
    /// 합쳤다. `fast_forward`면 새 버전 없이 앞으로 감았다.
    #[serde(rename_all = "camelCase")]
    Merged {
        branch: String,
        fast_forward: bool,
    },
    /// 겹쳐서 합치는 중으로 멈췄다. 충돌 해결 화면으로 간다(D-069).
    MergeConflicts {
        paths: Vec<String>,
    },
    /// 합치기를 마쳤다(합친 버전을 만들었다).
    ConflictFinished,
    /// 합치기를 그만뒀다.
    ConflictAborted,
    Stashed,
    TookOut,
    ParkedResolved,
    Discarded {
        count: u32,
    },
    Restored,
}

impl From<Outcome> for ActionResult {
    fn from(o: Outcome) -> Self {
        match o {
            Outcome::RepoCreated(repo) => ActionResult::RepoCreated {
                repo: RepoInfo::from_path(repo.as_path()),
            },
            Outcome::Staged { count } => ActionResult::Staged { count },
            Outcome::Unstaged { count } => ActionResult::Unstaged { count },
            Outcome::Committed { short_oid, summary } => {
                ActionResult::Committed { short_oid, summary }
            }
            Outcome::Switched(r) => ActionResult::Switched {
                branch: r.branch,
                parked_on: r.parked.map(|(b, _)| b),
                unparked: r.unparked,
                unpark_blocked: r.unpark_blocked,
            },
            Outcome::BranchCreated { name } => ActionResult::BranchCreated { name },
            Outcome::Fetched => ActionResult::Fetched,
            Outcome::Synced(r) => ActionResult::Synced {
                received: r.received,
                sent: r.sent,
                merged: r.merged,
                overlap: r.overlap,
                merge_started: r.merge_started,
            },
            Outcome::Ignored { count } => ActionResult::Ignored { count },
            Outcome::Merged {
                branch,
                fast_forward,
            } => ActionResult::Merged {
                branch,
                fast_forward,
            },
            Outcome::MergeConflicts { paths } => ActionResult::MergeConflicts { paths },
            Outcome::ConflictFinished => ActionResult::ConflictFinished,
            Outcome::ConflictAborted => ActionResult::ConflictAborted,
            Outcome::Stashed => ActionResult::Stashed,
            Outcome::TookOut => ActionResult::TookOut,
            Outcome::ParkedResolved => ActionResult::ParkedResolved,
            Outcome::Discarded { count } => ActionResult::Discarded { count },
            Outcome::Restored => ActionResult::Restored,
        }
    }
}

// ---- 오류 ----

/// 화면에 보내는 오류. 문구는 화면이 `code`로 고르고, `detail`은 "자세히 보기"의 원문이다.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ErrorView {
    pub code: ErrorCode,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    GitMissing,
    GitTooOld,
    FolderNotFound,
    NotARepo,
    AlreadyRepo,
    UnknownAction,
    PreviewRequired,
    PreviewMismatch,
    InvalidInput,
    Busy,
    NotOpened,
    NothingToUndo,
    UndoStale,
    UndoPushed,
    StagedChanges,
    ConflictChanged,
    BackupFailed,
    OperationInProgress,
    ParkBlocked,
    NeedsTrust,
    NoRemote,
    WatchUnavailable,
    // 아래는 git 실패 종류(D-021)
    NothingToCommit,
    IdentityMissing,
    InvalidBranchName,
    BranchExists,
    WouldOverwrite,
    NotFastForward,
    PushRejected,
    NoUpstream,
    SshHostUnverified,
    SshAuthFailed,
    HttpAuthFailed,
    RemoteUnreachable,
    ProtocolNotAllowed,
    UnparkConflict,
    TimedOut,
    MergeConflict,
    DubiousOwnership,
    /// 분류하지 못한 git 실패. 원문은 "자세히 보기".
    GitFailed,
    Internal,
}

impl From<git::FailureKind> for ErrorCode {
    fn from(kind: git::FailureKind) -> Self {
        use git::FailureKind as F;
        match kind {
            F::NothingToCommit => ErrorCode::NothingToCommit,
            F::IdentityMissing => ErrorCode::IdentityMissing,
            F::InvalidRefName => ErrorCode::InvalidBranchName,
            F::BranchExists => ErrorCode::BranchExists,
            F::WouldOverwrite => ErrorCode::WouldOverwrite,
            F::NotFastForward => ErrorCode::NotFastForward,
            F::PushRejected => ErrorCode::PushRejected,
            F::NoUpstream => ErrorCode::NoUpstream,
            F::SshHostUnverified => ErrorCode::SshHostUnverified,
            F::SshAuthFailed => ErrorCode::SshAuthFailed,
            F::HttpAuthFailed => ErrorCode::HttpAuthFailed,
            F::RemoteUnreachable => ErrorCode::RemoteUnreachable,
            F::NotARepository => ErrorCode::NotARepo,
            F::DubiousOwnership => ErrorCode::DubiousOwnership,
            F::ProtocolNotAllowed => ErrorCode::ProtocolNotAllowed,
            F::UnparkConflict => ErrorCode::UnparkConflict,
            F::TimedOut => ErrorCode::TimedOut,
            F::MergeConflict => ErrorCode::MergeConflict,
            F::Other => ErrorCode::GitFailed,
        }
    }
}

impl From<&CoreError> for ErrorView {
    fn from(e: &CoreError) -> Self {
        let code = match e {
            CoreError::GitMissing => ErrorCode::GitMissing,
            CoreError::GitTooOld { .. } => ErrorCode::GitTooOld,
            CoreError::FolderNotFound(_) => ErrorCode::FolderNotFound,
            CoreError::NotARepo | CoreError::Unavailable(UnavailableReason::NotARepo) => {
                ErrorCode::NotARepo
            }
            CoreError::Unavailable(UnavailableReason::AlreadyRepo) => ErrorCode::AlreadyRepo,
            CoreError::UnknownAction(_) => ErrorCode::UnknownAction,
            CoreError::PreviewRequired => ErrorCode::PreviewRequired,
            CoreError::PreviewMismatch => ErrorCode::PreviewMismatch,
            CoreError::InvalidInput(_) => ErrorCode::InvalidInput,
            CoreError::Busy => ErrorCode::Busy,
            CoreError::NotOpened => ErrorCode::NotOpened,
            CoreError::NothingToUndo => ErrorCode::NothingToUndo,
            CoreError::UndoStale => ErrorCode::UndoStale,
            CoreError::UndoPushed => ErrorCode::UndoPushed,
            CoreError::StagedChanges => ErrorCode::StagedChanges,
            CoreError::ConflictChanged => ErrorCode::ConflictChanged,
            CoreError::BackupFailed => ErrorCode::BackupFailed,
            CoreError::OperationInProgress => ErrorCode::OperationInProgress,
            CoreError::NeedsTrust => ErrorCode::NeedsTrust,
            CoreError::NoRemote => ErrorCode::NoRemote,
            CoreError::ParkBlocked(_) => ErrorCode::ParkBlocked,
            CoreError::BranchExists => ErrorCode::BranchExists,
            CoreError::WatchUnavailable(_) => ErrorCode::WatchUnavailable,
            CoreError::Git(e) => e.kind().into(),
            CoreError::DuplicateAction(_) | CoreError::InputMismatch(_) | CoreError::Journal(_) => {
                ErrorCode::Internal
            }
        };
        let detail = e.detail().or_else(|| match code {
            ErrorCode::Internal | ErrorCode::UnknownAction | ErrorCode::InvalidInput => {
                Some(e.to_string())
            }
            _ => None,
        });
        // "자세히 보기"에 주소 속 자격 증명, 토큰, 홈 폴더가 나오지 않게 한 곳에서 가린다(D-040).
        let home = platform::home_dir().map(|h| h.to_string_lossy().into_owned());
        let detail = detail.map(|d| crate::redact::redact(&d, home.as_deref()));
        Self { code, detail }
    }
}

impl From<CoreError> for ErrorView {
    fn from(e: CoreError) -> Self {
        Self::from(&e)
    }
}

// ---- 파일 변경 감시 이벤트 ----

/// 저장소 파일이 바뀌었다. `worktree`면 바뀐 내용·비교를, `git`이면 바뀐 내용·저장 기록을 다시 읽는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RepoChanged {
    pub repo: String,
    pub worktree: bool,
    pub git: bool,
}

/// 감시가 켜졌는지. 꺼져 있으면 화면은 창 포커스 때 새로 고침한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WatchStatus {
    pub repo: String,
    pub active: bool,
    pub reason: Option<String>,
}

// ---- 프로젝트 열기, 최근 프로젝트 (D-032) ----

/// 어떻게 여는지. 고른 폴더는 저장소가 아니면 버전 관리 시작 안내로, 최근 목록은 기록과 같은
/// 저장소일 때만 연다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum OpenHow {
    Picked,
    Recent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProjectOpenResult {
    Opened {
        repo: RepoInfo,
    },
    /// 폴더가 없다.
    Missing,
    /// 폴더는 있지만 (그 자리의) 저장소가 아니다.
    NotARepo {
        folder: String,
    },
    /// 폴더 주인이 다른 사용자라 git이 다루지 않는다. 사용자가 고르면 이 폴더만 믿는다(D-039).
    NotOwned {
        folder: String,
    },
    /// 실행되는 설정이 있어 믿을지 물어야 한다(D-039). `keys`는 설정 이름과 `hooks/이름`.
    /// `can_disable`이 거짓이면 덮어써서 끌 수 없는 키가 있어 "끄고 열기"를 보이지 않는다.
    #[serde(rename_all = "camelCase")]
    NeedsTrust {
        folder: String,
        keys: Vec<String>,
        can_disable: bool,
    },
}

/// 신뢰 확인에서 고른 것(D-039).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TrustChoice {
    /// 믿고 열기
    Trust,
    /// 끄고 열기
    Disable,
}

impl From<TrustChoice> for crate::trust::TrustDecision {
    fn from(c: TrustChoice) -> Self {
        match c {
            TrustChoice::Trust => Self::Trust,
            TrustChoice::Disable => Self::Disable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    /// 마지막으로 연 시각(unix 초).
    #[specta(type = u32)]
    pub last_opened_at: i64,
}

impl From<crate::recent::RecentEntry> for RecentProject {
    fn from(e: crate::recent::RecentEntry) -> Self {
        Self {
            path: e.path,
            name: e.name,
            last_opened_at: e.last_opened_at,
        }
    }
}

/// 시작 화면의 프로젝트 상태 요약(D-031). 문장은 화면(messages)이 만든다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummaryView {
    pub branch: BranchView,
    /// 충돌 파일을 포함한 바뀐 파일 수.
    pub changed_files: u32,
    pub conflicted_files: u32,
    /// 마지막 버전. 저장한 버전이 없으면 null.
    pub last_commit: Option<CommitRow>,
}
