//! 합치기 제안(D-048, D-071). 앱이 방향 하나만 고른다.

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MergeHintKind {
    /// 제안할 것이 없다(기본 브랜치에 있음, 같음, 기본 브랜치 없음).
    None,
    /// 기본 브랜치에 새 버전이 있다: "main 가져와 합치기".
    Bring,
    /// 내 브랜치만 앞서 있다: 브랜치 목록의 "main에 합치기".
    Into,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MergeHintView {
    pub kind: MergeHintKind,
    /// 기본 브랜치 이름.
    pub branch: Option<String>,
    /// 기본 브랜치에만 있는 버전 수.
    pub behind: u32,
    /// 내 브랜치에만 있는 버전 수.
    pub ahead: u32,
    /// 가져와 합치면 겹치는 파일(5개까지). 겹침 검사를 건너뛰었으면 비어 있다.
    pub overlap: Vec<String>,
    pub overlap_count: u32,
}

impl MergeHintView {
    pub(crate) fn none() -> Self {
        Self {
            kind: MergeHintKind::None,
            branch: None,
            behind: 0,
            ahead: 0,
            overlap: Vec::new(),
            overlap_count: 0,
        }
    }
}
