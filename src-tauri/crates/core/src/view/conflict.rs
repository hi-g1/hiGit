//! 충돌 해결 화면 조회(D-028, D-069, D-086). git의 충돌 표시 문자는 담지 않는다.

use serde::{Deserialize, Serialize};
use specta::Type;

/// 어떤 작업이 충돌을 남겼는지.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConflictContext {
    /// 합치기(`MERGE_HEAD`).
    Merge,
}

/// 카드 이름표: "내 브랜치 · 사람 · 날짜 · 이름".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConflictSide {
    /// 브랜치 이름. 브랜치가 없으면 짧은 해시.
    pub label: String,
    pub author: String,
    /// 작성 시각(unix 초). TS에서는 number(`CommitRow.author_time`과 같은 방식).
    #[specta(type = u32)]
    pub at: i64,
    pub summary: String,
    pub short_oid: String,
}

/// `Text`는 곳마다 고르고, `Whole`은 파일 단위로 한쪽을 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConflictFileKind {
    Text,
    Whole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFileSummary {
    pub path: String,
    pub kind: ConflictFileKind,
    /// 고를 곳 수(`Whole`이면 1).
    pub hunks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConflictStateView {
    /// 합치는 중인지.
    pub active: bool,
    pub context: ConflictContext,
    pub ours: Option<ConflictSide>,
    pub theirs: Option<ConflictSide>,
    /// 아직 고르지 않은 충돌 파일(200개까지).
    pub files: Vec<ConflictFileSummary>,
    /// 200개를 넘어 보내지 않은 파일 수.
    pub more_files: u32,
    /// 문제없이 합쳐진 파일 수.
    pub clean_files: u32,
    /// 앱이 끝낼 수 없는 다른 작업(리베이스·cherry-pick·revert)이 충돌을 남겼다.
    pub other_operation: bool,
}

/// 한쪽 파일의 모양.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SideState {
    Text,
    Binary,
    /// 이 쪽에서는 지웠다.
    Deleted,
    Link {
        target: String,
    },
    /// 1MB가 넘는다.
    Large,
    Submodule,
}

/// 고를 곳 하나. 각 쪽 내용과 앞뒤 3줄. 줄 끝은 뺐다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConflictHunkView {
    pub index: u32,
    /// 내 쪽 파일에서 시작하는 줄(1부터).
    pub line: u32,
    pub ours: Vec<String>,
    pub base: Vec<String>,
    pub theirs: Vec<String>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFileView {
    pub path: String,
    pub kind: ConflictFileKind,
    pub hunks: Vec<ConflictHunkView>,
    pub ours_side: SideState,
    pub theirs_side: SideState,
    /// `Whole`일 때 각 쪽 앞 40줄.
    pub ours_preview: Vec<String>,
    pub theirs_preview: Vec<String>,
}
