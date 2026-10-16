//! 앱 설정 화면 타입(D-127, D-148).

use serde::{Deserialize, Serialize};
use specta::Type;

/// 화면 언어(D-139). 시스템 따라가기는 화면이 웹뷰 언어로 정한다(D-149).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LanguageSetting {
    #[default]
    System,
    Ko,
    En,
}

/// 글자 크기 단계(D-138, D-146).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FontSizeSetting {
    Small,
    #[default]
    Normal,
    Large,
    Xlarge,
}

/// 화면 모드(D-145, D-151).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ThemeSetting {
    #[default]
    System,
    Light,
    Dark,
}

/// 처음 하는 일 안내(D-168, D-205).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TipId {
    FirstBranch,
    FirstConflict,
    FirstSync,
    FirstUndo,
}

impl TipId {
    /// `settings.json`의 `tips_seen` 비트.
    pub fn bit(self) -> u8 {
        match self {
            TipId::FirstBranch => 1,
            TipId::FirstConflict => 2,
            TipId::FirstSync => 4,
            TipId::FirstUndo => 8,
        }
    }
}

/// 이미 닫은 안내.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TipsSeenView {
    pub first_branch: bool,
    pub first_conflict: bool,
    pub first_sync: bool,
    pub first_undo: bool,
}

impl From<u8> for TipsSeenView {
    fn from(bits: u8) -> Self {
        let has = |t: TipId| bits & t.bit() != 0;
        Self {
            first_branch: has(TipId::FirstBranch),
            first_conflict: has(TipId::FirstConflict),
            first_sync: has(TipId::FirstSync),
            first_undo: has(TipId::FirstUndo),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsView {
    /// "올린 버전도 바꾸기". 켜면 덮어쓰며 올리기도 보인다.
    pub allow_published_rewrite: bool,
    pub language: LanguageSetting,
    pub font_size: FontSizeSetting,
    pub theme: ThemeSetting,
    /// "용어 설명 보기"(D-140). 끄면 용어 툴팁이 없다.
    pub term_help: bool,
    /// 비교 칸 "줄 바꿈"(D-144).
    pub diff_wrap: bool,
    /// 칸 나눔선으로 정한 왼쪽 칸 폭(rem, D-164). null이면 화면의 기본 폭.
    pub workspace_pane: Option<f32>,
    pub conflict_pane: Option<f32>,
    pub files_pane: Option<f32>,
    /// "올리기와 가져오기에 이 연결 쓰기"(D-187).
    pub git_helper: bool,
    /// "처음 하는 일 안내"(D-205).
    pub tips_on: bool,
    pub tips_seen: TipsSeenView,
}

/// 바꿀 칸만 담는다. 없는 칸은 그대로. 고정 열거형·참거짓·칸 폭 숫자뿐이다.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsPatch {
    #[serde(default)]
    pub allow_published_rewrite: Option<bool>,
    #[serde(default)]
    pub language: Option<LanguageSetting>,
    #[serde(default)]
    pub font_size: Option<FontSizeSetting>,
    #[serde(default)]
    pub theme: Option<ThemeSetting>,
    #[serde(default)]
    pub term_help: Option<bool>,
    #[serde(default)]
    pub diff_wrap: Option<bool>,
    /// rem. 10~80 밖이면 무시한다.
    #[serde(default)]
    pub workspace_pane: Option<f32>,
    #[serde(default)]
    pub conflict_pane: Option<f32>,
    #[serde(default)]
    pub files_pane: Option<f32>,
    #[serde(default)]
    pub git_helper: Option<bool>,
    #[serde(default)]
    pub tips_on: Option<bool>,
    /// 이 안내를 닫았다(다시 뜨지 않는다).
    #[serde(default)]
    pub tip_seen: Option<TipId>,
    /// 모든 안내를 다시 보인다("다시 보기").
    #[serde(default)]
    #[specta(optional)]
    pub reset_tips: bool,
}

/// 화면 배율을 정하는 OS 값(D-147). 화면이 루트 글자 크기를 계산한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DisplayHintsView {
    /// OS 기본 본문 글자 크기(px). Windows 14, Ubuntu 15.
    pub body_px: u8,
    /// OS 글자 배율(%). 모르면 100.
    pub text_scale_percent: u16,
    /// OS가 다크 모드인지. 모르면 null(화면은 CSS에 맡긴다).
    pub dark: Option<bool>,
}

/// 폴더 오른쪽 클릭 메뉴 종류(D-214). 화면은 이것으로 설명 문구를 고르고 `unsupported`면 행을 숨긴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ShellMenuKindView {
    Explorer,
    NautilusScripts,
    Unsupported,
}

/// 오른쪽 클릭 메뉴의 지금 상태. `installed`는 실제 등록을 읽은 것이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ShellMenuView {
    pub kind: ShellMenuKindView,
    pub installed: bool,
}
