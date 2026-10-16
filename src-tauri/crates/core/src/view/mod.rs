//! 화면에 보내는 타입(D-019). 바인딩용 derive(serde, specta)는 이 모듈에만 붙인다.
//! `git` 크레이트 타입은 여기서 옮겨 담고, 화면에 그대로 노출하지 않는다.

use std::path::Path;

mod branch;
mod conflict;
mod diff;
mod force_push;
mod github;
mod history;
mod hosting;
mod identity;
mod image;
mod input;
mod map;
mod merge;
mod pending;
mod publish;
mod recovery;
mod remote_branch;
mod secret;
mod settings;
mod stash;

pub use branch::{
    BranchListView, BranchNameCheck, BranchRow, CarryBlock, NameProblem, SwitchPlanView,
};
pub use conflict::{
    ConflictContext, ConflictDraftView, ConflictFileKind, ConflictFileSummary, ConflictFileView,
    ConflictHunkView, ConflictSide, ConflictStateView, DraftFile, DraftPick, SideState,
};
pub use diff::{DiffLineView, DiffRequest, FileDiffView, HunkView, LineKindView};
pub use force_push::{DroppedVersion, ForcePushBlock, ForcePushPlanView};
pub use github::{
    CloneDefaultsView, EmailCheckState, EmailCheckView, GitHubRepoListView, GitHubRepoRowView,
    GitHubStatusView, HostingPage, LoginChallengeView, LoginPollView, ScopeAdd,
};
pub use history::{
    CommitDetailView, CommitFileView, RewriteBlocked, RewriteCardView, RewritePlanView,
};
pub use hosting::{
    ChecksView, HostingContextView, HostingProviderView, MergeMethodView, MergeableView,
    PrCommentView, PrCommentsView, PrDetailView, PrDraftView, PrListRequest, PrListView,
    PrRepoView, PrRowView, PrStateFilterView, PrStateView, ReviewsView,
};
pub use identity::IdentityView;
pub use image::{ImageBlob, ImageMime, ImagePairView, ImageRequest};
pub use input::{
    ActionInput, AmendInput, BranchCreateInput, BranchNameInput, BranchRenameInput, CloneInput,
    CommitInput, ConflictFinishInput, DirtyChoice, FetchInput, FileChoice, FileResolution,
    ForcePushInput, HunkChoice, IdentityInput, MergeInput, MergeIntoInput, OidInput, PathsInput,
    PrCommentInput, PrCreateInput, PrMergeInput, PrNumberInput, PublishInput, RemoteBranchInput,
    RemoteBranchRenameInput, RemoteTagInput, RestoreInput, RewriteInput, RewriteKindInput,
    RewriteStepInput, SwitchInput, SyncInput, TagDeleteInput, TagInput, TagPushInput, TakeOutInput,
    TakeOutSource, Visibility,
};
pub use map::{HomeMapView, MapCommit};
pub use merge::{MergeHintKind, MergeHintView};
pub use pending::{OtherOperation, PendingWorkView};
pub use publish::PublishDraftView;
pub use recovery::{RecoveryItem, RecoverySource, RecoveryView, RestoreKind, UndoHint};
pub use remote_branch::{RemoteBranchBlock, RemoteBranchPlanView};
pub use secret::{SecretCheckRequest, SecretCheckView, SecretFinding, SecretRule};
pub use settings::{
    AppSettingsPatch, AppSettingsView, DisplayHintsView, FontSizeSetting, LanguageSetting,
    ThemeSetting, TipId, TipsSeenView,
};
pub use stash::{StashItem, StashListView};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ActionMeta, CoreError, Outcome, Risk, UnavailableReason};

/// 오래 걸리는 작업의 진행(1초마다, D-080). 파일 이름이나 원격 주소는 담지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpProgress {
    pub repo: String,
    /// 도는 액션 id.
    pub action: String,
    pub elapsed_secs: u32,
    /// 마지막 출력 뒤 시간(원격 작업). 로컬 작업은 경과와 같다.
    pub idle_secs: u32,
    /// Git이 준 진행(D-172). 단계와 개수만이고 파일 이름·주소는 없다.
    pub phase: Option<ProgressPhaseView>,
    pub done: Option<u32>,
    pub total: Option<u32>,
}

/// 진행 단계(D-172).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ProgressPhaseView {
    Count,
    Receive,
    Resolve,
    Send,
}

/// 앱 시작 시 확인한 환경.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub git_version: String,
    pub git_security: GitSecurity,
    /// 배포판 패키지 git이면 패키지 버전(예: `1:2.34.1-1ubuntu1.17`).
    pub git_package_version: Option<String>,
    /// 찾은 git 실행 파일의 경로(설정 "고급", D-160). 시험용 Core는 빈 글자.
    pub git_path: String,
    /// 되돌리기 서명 키를 이 컴퓨터에 저장했는지(D-210). `temporary`면 화면이 알린다.
    pub undo_key: UndoKeyState,
    /// 저장하지 못했으면 "자세히 보기"에 보일 앱 데이터 폴더(홈은 가림).
    pub undo_key_detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum UndoKeyState {
    /// 다음 실행에도 같은 키.
    Saved,
    /// 이번 실행에서만 쓰는 키. 다시 켜면 이전 작업은 Ctrl+Z로 되돌리지 못한다.
    Temporary,
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

/// 버전 기록 범위(D-088).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LogScope {
    /// 지금 위치와 로컬 브랜치 끝 모두.
    #[default]
    All,
    /// 지금 위치의 조상만.
    Current,
}

#[derive(Debug, Clone, Default, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogQuery {
    pub skip: u32,
    pub limit: u32,
    #[serde(default)]
    pub scope: LogScope,
    /// 다음 쪽이면 첫 쪽이 준 `tips`를 그대로. 없으면 첫 쪽(끝점을 새로 정한다).
    #[serde(default)]
    pub tips: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogPage {
    /// 위상 순서(자식이 부모보다 먼저).
    pub commits: Vec<CommitRow>,
    /// 이 쪽 뒤에 저장이 더 있는지.
    pub has_more: bool,
    /// 이 쪽 버전에 붙은 로컬 브랜치와 태그 이름.
    pub labels: Vec<RefLabelView>,
    /// 이 기록의 끝점(해시). 다음 쪽 요청에 그대로 되돌려 보낸다.
    pub tips: Vec<String>,
    /// 아래는 첫 쪽에만 채운다(다음 쪽은 비어 있다).
    /// 끝점이 200개를 넘어 그리지 않은 브랜치 수.
    pub tips_cut: u32,
    /// 지금 위치(HEAD)의 해시.
    pub head: Option<String>,
    pub current_branch: Option<String>,
    pub default_branch: Option<String>,
    /// 기본 브랜치 끝의 해시(노선도 0번 레인).
    pub default_tip: Option<String>,
    /// 지금 브랜치의 안 올린 버전(최대 500개). upstream이 없으면 비어 있다.
    pub unpushed: Vec<String>,
}

/// 버전 하나의 바뀐 파일·줄 수(노선도 선 굵기, D-089).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommitStatView {
    pub oid: String,
    pub files: u32,
    pub added: u32,
    pub removed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RefLabelView {
    pub oid: String,
    pub kind: RefLabelKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RefLabelKind {
    Branch,
    Tag,
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
    /// 입력이 필요한 액션(`ActionInput`의 그 변형에 칸이 있다). 명령 팔레트가 바로 하기와 화면 열기를 가른다(D-199).
    pub needs_input: bool,
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
            // 입력 없는 변형만 `{"id": …}`로 읽힌다(인접 태그 `id`·`input`).
            needs_input: serde_json::from_value::<ActionInput>(serde_json::json!({ "id": m.id }))
                .is_err(),
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
    Amended {
        short_oid: String,
        summary: String,
    },
    /// 고른 버전의 변경을 취소하는 새 버전을 만들었다.
    #[serde(rename_all = "camelCase")]
    Reverted {
        short_oid: String,
        summary: String,
    },
    /// 고른 버전을 지금 브랜치로 복사했다.
    #[serde(rename_all = "camelCase")]
    Picked {
        short_oid: String,
        summary: String,
    },
    /// 지금 브랜치를 옛 버전으로 옮겼다. 기록에서 빠진 버전 수.
    Reset {
        dropped: u32,
    },
    Tagged {
        name: String,
    },
    BranchDeleted {
        name: String,
    },
    BranchRenamed {
        from: String,
        to: String,
    },
    TagDeleted {
        name: String,
    },
    /// 기록을 정리했다. 남은·뺀·묶어 합친 버전 수.
    Rewritten {
        kept: u32,
        dropped: u32,
        joined: u32,
    },
    /// 복사·되돌리기·기록 정리가 멈췄다. 충돌 해결 화면으로 간다(충돌 0곳이면 hook 거부 등).
    OperationConflicts {
        context: ConflictContext,
        paths: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    Switched {
        branch: String,
        /// 떠난 브랜치에 맡긴 파일이 있으면 그 브랜치 이름.
        parked_on: Option<String>,
        unparked: bool,
        unpark_blocked: bool,
        /// 두고 가거나 가져간 고친 파일 수(D-195).
        files: u32,
        /// 고친 파일을 이 브랜치로 가져왔다.
        carried: bool,
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
    /// 기록 정리의 충돌 하나를 마쳤고 다음 버전에서 다시 겹쳤다. 화면은 다음 충돌을 연다.
    ConflictContinued {
        paths: Vec<String>,
    },
    /// 합치기를 그만뒀다.
    ConflictAborted,
    Stashed,
    TookOut,
    ParkedResolved,
    Discarded {
        count: u32,
    },
    Restored,
    LocksCleared {
        count: u32,
    },
    PrCreated {
        number: u32,
    },
    PrCommented {
        number: u32,
    },
    PrMerged {
        number: u32,
    },
    PrClosed {
        number: u32,
    },
    PrReopened {
        number: u32,
    },
    Forked {
        owner: String,
        name: String,
    },
    RemoteTagDeleted {
        name: String,
    },
    ParkedRecovered {
        count: u32,
    },
    IdentitySaved,
    ForcePushed {
        dropped: u32,
    },
    RemoteBranchDeleted {
        remote: String,
        branch: String,
    },
    RemoteBranchRenamed {
        remote: String,
        from: String,
        to: String,
    },
    TagPushed {
        name: String,
    },
    Published {
        owner: String,
        name: String,
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
            Outcome::Amended { short_oid, summary } => ActionResult::Amended { short_oid, summary },
            Outcome::Reverted { short_oid, summary } => {
                ActionResult::Reverted { short_oid, summary }
            }
            Outcome::Picked { short_oid, summary } => ActionResult::Picked { short_oid, summary },
            Outcome::Reset { dropped } => ActionResult::Reset { dropped },
            Outcome::Tagged { name } => ActionResult::Tagged { name },
            Outcome::BranchDeleted { name } => ActionResult::BranchDeleted { name },
            Outcome::BranchRenamed { from, to } => ActionResult::BranchRenamed { from, to },
            Outcome::TagDeleted { name } => ActionResult::TagDeleted { name },
            Outcome::Rewritten {
                kept,
                dropped,
                joined,
            } => ActionResult::Rewritten {
                kept,
                dropped,
                joined,
            },
            Outcome::OperationConflicts { context, paths } => {
                ActionResult::OperationConflicts { context, paths }
            }
            Outcome::Switched(r) => ActionResult::Switched {
                branch: r.branch,
                parked_on: r.parked.map(|(b, _)| b),
                unparked: r.unparked,
                unpark_blocked: r.unpark_blocked,
                files: r.files,
                carried: r.carried,
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
            Outcome::ConflictContinued { paths } => ActionResult::ConflictContinued { paths },
            Outcome::ConflictAborted => ActionResult::ConflictAborted,
            Outcome::Stashed => ActionResult::Stashed,
            Outcome::TookOut => ActionResult::TookOut,
            Outcome::ParkedResolved => ActionResult::ParkedResolved,
            Outcome::Discarded { count, .. } => ActionResult::Discarded { count },
            Outcome::Restored => ActionResult::Restored,
            Outcome::LocksCleared { count } => ActionResult::LocksCleared { count },
            Outcome::PrCreated { number } => ActionResult::PrCreated { number },
            Outcome::PrCommented { number } => ActionResult::PrCommented { number },
            Outcome::PrMerged { number } => ActionResult::PrMerged { number },
            Outcome::PrClosed { number } => ActionResult::PrClosed { number },
            Outcome::PrReopened { number } => ActionResult::PrReopened { number },
            Outcome::Forked { owner, name } => ActionResult::Forked { owner, name },
            Outcome::RemoteTagDeleted { name } => ActionResult::RemoteTagDeleted { name },
            Outcome::ParkedRecovered { count } => ActionResult::ParkedRecovered { count },
            Outcome::IdentitySaved => ActionResult::IdentitySaved,
            Outcome::ForcePushed { dropped } => ActionResult::ForcePushed { dropped },
            Outcome::RemoteBranchDeleted { remote, branch } => {
                ActionResult::RemoteBranchDeleted { remote, branch }
            }
            Outcome::RemoteBranchRenamed { remote, from, to } => {
                ActionResult::RemoteBranchRenamed { remote, from, to }
            }
            Outcome::TagPushed { name } => ActionResult::TagPushed { name },
            Outcome::Published { owner, name } => ActionResult::Published { owner, name },
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
    CarryBlocked,
    PinLimit,
    NeedsTrust,
    NoRemote,
    WatchUnavailable,
    PublishedHistory,
    /// 올린 버전의 기록을 바꿔 원격과 갈라져 동기화가 멈췄다(D-129).
    RewrotePublished,
    /// 설정 "올린 버전도 바꾸기"가 꺼져 있다(D-127).
    ForcePushDisabled,
    /// 원격 기본 브랜치는 덮어쓰거나 지우지 않는다(D-128).
    ForcePushProtected,
    /// 덮어쓸 것이 없다(그냥 올리면 된다).
    NothingToOverwrite,
    /// 원격 기본 브랜치는 지우거나 이름을 바꾸지 않는다(D-130).
    RemoteBranchProtected,
    /// 새 이름으로는 올렸지만 옛 이름을 지우지 못했다(D-130).
    RemoteRenamePartial,
    /// GitHub에 올리기: 이미 원격이 있다(D-132).
    PublishHasRemote,
    /// GitHub에 올리기: 저장한 버전이 없다.
    PublishNoVersions,
    /// GitHub에 같은 이름의 저장소가 있다.
    GithubRepoExists,
    /// GitHub가 저장소 만들기 입력을 받지 않았다(그 밖의 422, 계획 010 R10).
    GithubRepoInvalid,
    PrNotMergeable,
    PrHeadChanged,
    PrNotOpen,
    ForkNotReady,
    ForkRemoteTaken,
    ForkParentMismatch,
    GithubOrgForbidden,
    HasChanges,
    TagExists,
    BranchProtected,
    NoChange,
    AnnotatedTag,
    // GitHub 연결·API(D-109)
    GithubNotConfigured,
    GithubNotConnected,
    GithubExpired,
    GithubForbidden,
    GithubRateLimited,
    GithubSsoRequired,
    GithubNotFound,
    GithubMoved,
    GithubUnreachable,
    GithubUnavailable,
    GithubBadResponse,
    GithubKeychain,
    PrAlreadyExists,
    PrNoDifference,
    PrInvalid,
    /// 지금 브랜치가 올리는 원격이 github.com이 아니다.
    NotGithubRemote,
    /// 아직 올리지 않은 버전이 있거나 올린 적이 없다(검토 요청 전에 올린다).
    NotPushed,
    /// 검토 요청의 head와 대상이 같거나 head가 기본 브랜치다.
    PrSameBranch,
    // 아래는 git 실패 종류(D-021)
    NothingToCommit,
    IdentityMissing,
    InvalidBranchName,
    BranchExists,
    WouldOverwrite,
    NotFastForward,
    PushRejected,
    /// 덮어쓰며 올리기·원격 브랜치 지우기 사이에 원격이 바뀌었다(기대 값이 다름, D-128).
    RemoteMoved,
    /// 서버가 받기를 거절했다(보호 브랜치·hook).
    RemoteRefused,
    /// 원격에 같은 이름이 다른 버전으로 있다(태그 올리기).
    RemoteRefExists,
    /// 지우려는 원격 브랜치가 없다.
    RemoteRefMissing,
    /// 원격에 만들려는 브랜치 이름이 이미 있다(원격 브랜치 이름 바꾸기, core가 나눔).
    RemoteNameTaken,
    /// 원격의 같은 이름 태그가 다른 버전이다(D-182).
    RemoteTagDiffers,
    /// 원격 기본 브랜치라 이름을 똑같이 입력해야 한다(D-183).
    NameConfirmMismatch,
    NoUpstream,
    SshHostUnverified,
    SshAuthFailed,
    HttpAuthFailed,
    HttpForbidden,
    RemoteNotFound,
    GithubWorkflowScope,
    /// GitHub가 비공개 이메일이 든 버전을 막았다(GH007, D-186).
    PrivateEmailBlocked,
    /// GitHub가 100MB 넘는 파일이 든 버전을 막았다(GH001, D-194).
    FileTooLarge,
    /// 다른 프로그램이 파일을 쓰고 있다(D-194).
    FileInUse,
    /// 복제할 폴더가 이미 있고 비어 있지 않다.
    CloneTargetExists,
    CloneUrlInvalid,
    CloneNameInvalid,
    IdentityInvalid,
    GitOutdated,
    RemoteUnreachable,
    ProtocolNotAllowed,
    /// 올리는 곳을 바꾸는 설정을 끄고 연 저장소라 원격 작업을 하지 않았다(D-212).
    RemoteSettingsOff,
    UnparkConflict,
    TimedOut,
    MergeConflict,
    DubiousOwnership,
    Cancelled,
    Stalled,
    /// 다른 git 작업의 잠금 파일이 있다(D-081).
    IndexLocked,
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
            F::LeaseStale => ErrorCode::RemoteMoved,
            F::RemoteRefused => ErrorCode::RemoteRefused,
            F::PrivateEmailBlocked => ErrorCode::PrivateEmailBlocked,
            F::FileTooLarge => ErrorCode::FileTooLarge,
            F::FileInUse => ErrorCode::FileInUse,
            F::RemoteRefExists => ErrorCode::RemoteRefExists,
            F::RemoteRefMissing => ErrorCode::RemoteRefMissing,
            F::NoUpstream => ErrorCode::NoUpstream,
            F::SshHostUnverified => ErrorCode::SshHostUnverified,
            F::SshAuthFailed => ErrorCode::SshAuthFailed,
            F::HttpAuthFailed => ErrorCode::HttpAuthFailed,
            F::HttpForbidden => ErrorCode::HttpForbidden,
            F::RemoteNotFound => ErrorCode::RemoteNotFound,
            F::GithubWorkflowScope => ErrorCode::GithubWorkflowScope,
            F::TargetExists => ErrorCode::CloneTargetExists,
            F::RemoteUnreachable => ErrorCode::RemoteUnreachable,
            F::NotARepository => ErrorCode::NotARepo,
            F::DubiousOwnership => ErrorCode::DubiousOwnership,
            F::ProtocolNotAllowed => ErrorCode::ProtocolNotAllowed,
            F::RemoteSettingsOff => ErrorCode::RemoteSettingsOff,
            F::UnparkConflict => ErrorCode::UnparkConflict,
            F::TimedOut => ErrorCode::TimedOut,
            F::Cancelled => ErrorCode::Cancelled,
            F::Stalled => ErrorCode::Stalled,
            F::Locked => ErrorCode::IndexLocked,
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
            CoreError::CarryBlocked(_) => ErrorCode::CarryBlocked,
            CoreError::PinLimit => ErrorCode::PinLimit,
            CoreError::BranchExists => ErrorCode::BranchExists,
            CoreError::WatchUnavailable(_) => ErrorCode::WatchUnavailable,
            CoreError::PublishedHistory => ErrorCode::PublishedHistory,
            CoreError::RewrotePublished => ErrorCode::RewrotePublished,
            CoreError::ForcePushDisabled => ErrorCode::ForcePushDisabled,
            CoreError::ForcePushProtected => ErrorCode::ForcePushProtected,
            CoreError::NothingToOverwrite => ErrorCode::NothingToOverwrite,
            CoreError::RemoteBranchProtected => ErrorCode::RemoteBranchProtected,
            CoreError::RemoteNameTaken => ErrorCode::RemoteNameTaken,
            CoreError::RemoteTagDiffers => ErrorCode::RemoteTagDiffers,
            CoreError::NameConfirmMismatch => ErrorCode::NameConfirmMismatch,
            CoreError::PublishHasRemote => ErrorCode::PublishHasRemote,
            CoreError::PublishNoVersions => ErrorCode::PublishNoVersions,
            CoreError::RemoteRenamePartial(_) => ErrorCode::RemoteRenamePartial,
            CoreError::HasChanges => ErrorCode::HasChanges,
            CoreError::TagExists => ErrorCode::TagExists,
            CoreError::BranchProtected => ErrorCode::BranchProtected,
            CoreError::NoChange => ErrorCode::NoChange,
            CoreError::IdentityMissing => ErrorCode::IdentityMissing,
            CoreError::AnnotatedTag => ErrorCode::AnnotatedTag,
            CoreError::CloneUrlInvalid => ErrorCode::CloneUrlInvalid,
            CoreError::CloneNameInvalid => ErrorCode::CloneNameInvalid,
            CoreError::IdentityInvalid => ErrorCode::IdentityInvalid,
            CoreError::CloneTargetExists => ErrorCode::CloneTargetExists,
            CoreError::GitOutdated => ErrorCode::GitOutdated,
            CoreError::Git(e) => e.kind().into(),
            CoreError::Github { kind, .. } => github_code(*kind),
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

fn github_code(kind: crate::GithubFailure) -> ErrorCode {
    use crate::GithubFailure as G;
    match kind {
        G::NotConfigured => ErrorCode::GithubNotConfigured,
        G::NotConnected => ErrorCode::GithubNotConnected,
        G::Expired => ErrorCode::GithubExpired,
        G::Forbidden => ErrorCode::GithubForbidden,
        G::RateLimited => ErrorCode::GithubRateLimited,
        G::SsoRequired => ErrorCode::GithubSsoRequired,
        G::NotFound => ErrorCode::GithubNotFound,
        G::Moved => ErrorCode::GithubMoved,
        G::Unreachable => ErrorCode::GithubUnreachable,
        G::Unavailable => ErrorCode::GithubUnavailable,
        G::BadResponse => ErrorCode::GithubBadResponse,
        G::Keychain => ErrorCode::GithubKeychain,
        G::Cancelled => ErrorCode::Cancelled,
        G::PrAlreadyExists => ErrorCode::PrAlreadyExists,
        G::PrNoDifference => ErrorCode::PrNoDifference,
        G::PrInvalid => ErrorCode::PrInvalid,
        G::NotGithubRemote => ErrorCode::NotGithubRemote,
        G::NotPushed => ErrorCode::NotPushed,
        G::PrSameBranch => ErrorCode::PrSameBranch,
        G::RepoExists => ErrorCode::GithubRepoExists,
        G::RepoInvalid => ErrorCode::GithubRepoInvalid,
        G::PrNotMergeable => ErrorCode::PrNotMergeable,
        G::PrHeadChanged => ErrorCode::PrHeadChanged,
        G::PrNotOpen => ErrorCode::PrNotOpen,
        G::ForkNotReady => ErrorCode::ForkNotReady,
        G::ForkRemoteTaken => ErrorCode::ForkRemoteTaken,
        G::ForkParentMismatch => ErrorCode::ForkParentMismatch,
        G::OrgForbidden => ErrorCode::GithubOrgForbidden,
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
    /// 실행되는 설정이나 올리는 곳을 바꾸는 설정이 있어 믿을지 물어야 한다(D-039, D-212). `keys`는 설정
    /// 이름과 `hooks/이름`, `redirects`는 올리는 곳·연결을 바꾸는 설정. `can_disable`이 거짓이면 덮어써서
    /// 끌 수 없는 키가 있어 "끄고 열기"를 보이지 않는다.
    #[serde(rename_all = "camelCase")]
    NeedsTrust {
        folder: String,
        keys: Vec<String>,
        redirects: Vec<TrustRedirect>,
        can_disable: bool,
    },
}

/// 올리는 곳·연결을 바꾸는 설정 하나(D-212). 주소는 호스트만, 경로는 가린 것. 값을 보이지 않는 키는 `target`이 없다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct TrustRedirect {
    pub key: String,
    pub target: Option<String>,
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
    /// 고정(D-203).
    pub pinned: bool,
    /// 고정한 브랜치 이름. 화면이 지금 브랜치 목록에 있는 것만 보인다.
    pub pinned_branches: Vec<String>,
}

impl From<crate::recent::RecentEntry> for RecentProject {
    fn from(e: crate::recent::RecentEntry) -> Self {
        Self {
            path: e.path,
            name: e.name,
            last_opened_at: e.last_opened_at,
            pinned: e.pinned,
            pinned_branches: e.pinned_branches,
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
