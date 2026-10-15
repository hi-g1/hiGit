//! 화면에 보내는 타입(D-019). 바인딩용 derive(serde, specta)는 이 모듈에만 붙인다.
//! `git` 크레이트 타입은 여기서 옮겨 담고, 화면에 그대로 노출하지 않는다.

use std::path::Path;

mod diff;
mod input;

pub use diff::{DiffLineView, DiffRequest, FileDiffView, HunkView, LineKindView};
pub use input::{ActionInput, CommitInput, PathsInput};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ActionMeta, CoreError, Outcome, Risk, UnavailableReason};

/// 앱 시작 시 확인한 환경.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub git_version: String,
}

/// 사용자가 고른 폴더. 저장소가 아니면 `repo`가 비어 있다.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderInfo {
    pub folder: String,
    pub repo: Option<RepoInfo>,
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

#[derive(Debug, Clone, Serialize, Type)]
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
    NotYetSupported,
    InvalidInput,
    Busy,
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
            CoreError::NotYetSupported => ErrorCode::NotYetSupported,
            CoreError::InvalidInput(_) => ErrorCode::InvalidInput,
            CoreError::Busy => ErrorCode::Busy,
            CoreError::Git(e) => e.kind().into(),
            CoreError::DuplicateAction(_) | CoreError::InputMismatch(_) => ErrorCode::Internal,
        };
        let detail = e.detail().or_else(|| match code {
            ErrorCode::Internal | ErrorCode::UnknownAction | ErrorCode::InvalidInput => {
                Some(e.to_string())
            }
            _ => None,
        });
        Self { code, detail }
    }
}

impl From<CoreError> for ErrorView {
    fn from(e: CoreError) -> Self {
        Self::from(&e)
    }
}
