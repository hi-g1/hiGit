//! 임시 보관 목록(D-076).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StashItem {
    /// 꺼낼 때 그대로 돌려보내는 값.
    pub oid: String,
    /// 보관할 때 메시지(앱이 만든 것은 `higit stash <브랜치>`).
    pub message: String,
    /// 앱이 만든 항목이면 그때의 브랜치.
    pub branch: Option<String>,
    /// 만든 시각(unix 초).
    #[specta(type = u32)]
    pub at: i64,
    /// 든 파일 수(고친 파일과 새 파일).
    pub files: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StashListView {
    pub items: Vec<StashItem>,
}
