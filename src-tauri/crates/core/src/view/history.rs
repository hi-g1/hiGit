//! 버전 기록 화면의 조회: 기록 정리 계획(D-100, D-106), 버전 상세(D-105).

use serde::Serialize;
use specta::Type;

/// 기록 정리 카드 하나(새 버전 먼저).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RewriteCardView {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
    /// 전체 메시지(메시지 고치기의 처음 값).
    pub message: String,
    pub author: String,
    /// 작성 시각(unix 초).
    #[specta(type = u32)]
    pub time: i64,
}

/// 기록 정리를 할 수 없는 까닭. 실행할 때 core가 같은 검사를 다시 한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RewriteBlocked {
    /// 특정 버전을 보는 중(브랜치 없음).
    Detached,
    /// 고른 버전이 지금 브랜치 첫 부모 선 위의 연속 구간이 아니다.
    NotOnBranchLine,
    /// 고른 구간부터 지금 위치까지 합친 버전이 있다.
    HasMerge,
    /// 첫 버전(부모 없음)이 들어 있다.
    IncludesFirstVersion,
    /// 바뀔 버전 중 올린 것이 있다(D-097).
    Published,
    /// 고른 버전이 50개를 넘거나 지금 위치까지가 너무 멀다.
    TooMany,
    /// 추적 중인 파일을 고쳤다.
    HasChanges,
    /// 합치기·다른 정리 등 진행 중인 작업이 있다.
    OtherOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RewritePlanView {
    /// 고른 버전들(새 것 먼저). 구간을 알 수 없으면 비어 있다.
    pub cards: Vec<RewriteCardView>,
    pub blocked: Option<RewriteBlocked>,
    /// 정리하면 옛 버전을 가리키게 될 다른 브랜치·태그 수(막지는 않는다).
    pub stale_labels: u32,
}

/// 버전 하나에서 바뀐 파일.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommitFileView {
    pub path: String,
    pub orig_path: Option<String>,
    pub change: super::ChangeKind,
}

/// 버전 상세(이때 파일 보기, 마지막 버전 고치기의 처음 값, D-105).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetailView {
    pub oid: String,
    pub short_oid: String,
    /// 전체 메시지(64KB까지).
    pub message: String,
    pub author: String,
    /// 작성 시각(unix 초).
    #[specta(type = u32)]
    pub time: i64,
    pub parents: Vec<String>,
    /// 첫 부모와 비교해 바뀐 파일(500개까지).
    pub files: Vec<CommitFileView>,
    /// 500개를 넘어 보내지 않은 파일 수.
    pub more_files: u32,
}

impl From<git::CommitFile> for CommitFileView {
    fn from(f: git::CommitFile) -> Self {
        use git::Change as C;
        Self {
            path: f.path,
            orig_path: f.orig_path,
            change: match f.change {
                C::Added | C::Copied => super::ChangeKind::Added,
                C::Deleted => super::ChangeKind::Deleted,
                C::Renamed => super::ChangeKind::Renamed,
                C::TypeChanged => super::ChangeKind::TypeChanged,
                C::Unmerged => super::ChangeKind::Conflicted,
                C::Untracked => super::ChangeKind::Untracked,
                C::Modified | C::Unmodified => super::ChangeKind::Modified,
            },
        }
    }
}
