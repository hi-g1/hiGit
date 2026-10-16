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
