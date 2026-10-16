//! 앱 설정 화면 타입(D-127).

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsView {
    /// "올린 버전도 바꾸기". 켜면 덮어쓰며 올리기도 보인다.
    pub allow_published_rewrite: bool,
}

/// 바꿀 칸만 담는다. 없는 칸은 그대로.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsPatch {
    #[serde(default)]
    pub allow_published_rewrite: Option<bool>,
}
