//! 앱 설정 `settings.json`(D-127, D-148). 앱 데이터에만 있어 받은 저장소가 바꿀 수 없다.
//!
//! 읽기는 링크가 아니고 4KB 이하·알 수 없는 칸 없음·`v == 1`일 때만, 아니면 기본값. 쓰기는
//! 임시 파일 뒤 바꿔 넣기(`platform::write_private_file`). 쓰지 못하면 이번 실행 동안만 메모리.
//! 화면 설정 칸(언어·글자 크기·화면 모드·용어 설명·줄 바꿈·칸 폭)은 없으면 기본값으로 읽어 옛 파일도 읽힌다.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};

use crate::view::{
    AppSettingsPatch, AppSettingsView, DisplayHintsView, FontSizeSetting, LanguageSetting,
    ThemeSetting,
};

const FILE: &str = "settings.json";
const MAX_BYTES: u64 = 4096;
const VERSION: u32 = 1;

/// 칸 폭(rem)으로 받는 범위. 밖이면 기본 폭(`None`)으로 본다.
const PANE_REM: std::ops::RangeInclusive<f32> = 10.0..=80.0;

fn pane(rem: Option<f32>) -> Option<f32> {
    rem.filter(|r| r.is_finite() && PANE_REM.contains(r))
}

/// 실행마다 `ExecCtx`에 주는 설정 복사본. 실행기는 `allow_published_rewrite`만 읽는다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppSettings {
    /// "올린 버전도 바꾸기"(D-127). 켜면 올린 버전의 기록도 고치고 덮어쓰며 올릴 수 있다.
    pub allow_published_rewrite: bool,
    pub language: LanguageSetting,
    pub font_size: FontSizeSetting,
    pub theme: ThemeSetting,
    pub term_help: bool,
    pub diff_wrap: bool,
    /// 칸 나눔선으로 정한 왼쪽 칸 폭(rem, D-164). 없으면 화면의 기본 폭.
    pub workspace_pane: Option<f32>,
    pub conflict_pane: Option<f32>,
    pub files_pane: Option<f32>,
    /// "올리기와 가져오기에 이 연결 쓰기"(D-187). 끄면 git에 hiGit 로그인을 붙이지 않는다.
    pub git_helper: bool,
    /// "처음 하는 일 안내"(D-205)와 이미 닫은 안내 비트(`TipId::bit`).
    pub tips_on: bool,
    pub tips_seen: u8,
    /// 폴더 오른쪽 클릭 메뉴를 사람이 켜거나 껐는지(D-214). 없으면 아직 정하지 않았다(릴리스 빌드는 처음 켤 때 켠다).
    /// 실제 등록 상태는 OS에서 읽는다. 화면 설정 칸(`AppSettingsPatch`)으로는 바꾸지 않는다.
    pub shell_menu: Option<bool>,
}

/// 알려진 안내 비트. 모르는 비트는 읽을 때 버린다.
const TIP_BITS: u8 = 0b1111;

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            allow_published_rewrite: false,
            language: LanguageSetting::default(),
            font_size: FontSizeSetting::default(),
            theme: ThemeSetting::default(),
            term_help: true,
            diff_wrap: true,
            workspace_pane: None,
            conflict_pane: None,
            files_pane: None,
            git_helper: true,
            tips_on: true,
            tips_seen: 0,
            shell_menu: None,
        }
    }
}

fn yes() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

fn is_zero(v: &u8) -> bool {
    *v == 0
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFile {
    v: u32,
    allow_published_rewrite: bool,
    #[serde(default)]
    language: LanguageSetting,
    #[serde(default)]
    font_size: FontSizeSetting,
    #[serde(default)]
    theme: ThemeSetting,
    #[serde(default = "yes")]
    term_help: bool,
    #[serde(default = "yes")]
    diff_wrap: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    workspace_pane: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conflict_pane: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    files_pane: Option<f32>,
    #[serde(default = "yes")]
    git_helper: bool,
    /// 기본값이면 쓰지 않아 옛 파일과 글자가 같다.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    tips_on: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    tips_seen: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shell_menu: Option<bool>,
}

impl From<AppSettings> for SettingsFile {
    fn from(s: AppSettings) -> Self {
        Self {
            v: VERSION,
            allow_published_rewrite: s.allow_published_rewrite,
            language: s.language,
            font_size: s.font_size,
            theme: s.theme,
            term_help: s.term_help,
            diff_wrap: s.diff_wrap,
            workspace_pane: s.workspace_pane,
            conflict_pane: s.conflict_pane,
            files_pane: s.files_pane,
            git_helper: s.git_helper,
            tips_on: s.tips_on,
            tips_seen: s.tips_seen,
            shell_menu: s.shell_menu,
        }
    }
}

impl From<SettingsFile> for AppSettings {
    fn from(f: SettingsFile) -> Self {
        Self {
            allow_published_rewrite: f.allow_published_rewrite,
            language: f.language,
            font_size: f.font_size,
            theme: f.theme,
            term_help: f.term_help,
            diff_wrap: f.diff_wrap,
            workspace_pane: pane(f.workspace_pane),
            conflict_pane: pane(f.conflict_pane),
            files_pane: pane(f.files_pane),
            git_helper: f.git_helper,
            tips_on: f.tips_on,
            tips_seen: f.tips_seen & TIP_BITS,
            shell_menu: f.shell_menu,
        }
    }
}

pub(crate) struct SettingsStore {
    path: Option<PathBuf>,
    current: Mutex<AppSettings>,
}

impl SettingsStore {
    pub(crate) fn open(dir: &Path) -> Self {
        let path = dir.join(FILE);
        let current = read(&path).unwrap_or_default();
        Self {
            path: Some(path),
            current: Mutex::new(current),
        }
    }

    pub(crate) fn in_memory() -> Self {
        Self {
            path: None,
            current: Mutex::new(AppSettings::default()),
        }
    }

    pub(crate) fn get(&self) -> AppSettings {
        *self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 알려진 칸만 바꾼다. 값이 같으면 쓰지 않는다. 파일을 못 써도 이번 실행 동안은 바뀐 값을 쓴다.
    pub(crate) fn set(&self, patch: &AppSettingsPatch) -> AppSettings {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        let next = AppSettings {
            allow_published_rewrite: patch
                .allow_published_rewrite
                .unwrap_or(current.allow_published_rewrite),
            language: patch.language.unwrap_or(current.language),
            font_size: patch.font_size.unwrap_or(current.font_size),
            theme: patch.theme.unwrap_or(current.theme),
            term_help: patch.term_help.unwrap_or(current.term_help),
            diff_wrap: patch.diff_wrap.unwrap_or(current.diff_wrap),
            // 범위 밖 값은 무시하고 지금 값을 둔다.
            workspace_pane: pane(patch.workspace_pane).or(current.workspace_pane),
            conflict_pane: pane(patch.conflict_pane).or(current.conflict_pane),
            files_pane: pane(patch.files_pane).or(current.files_pane),
            git_helper: patch.git_helper.unwrap_or(current.git_helper),
            tips_on: patch.tips_on.unwrap_or(current.tips_on),
            tips_seen: if patch.reset_tips {
                0
            } else {
                current.tips_seen | patch.tip_seen.map_or(0, |t| t.bit())
            },
            shell_menu: current.shell_menu,
        };
        self.replace(&mut current, next);
        next
    }

    /// 오른쪽 클릭 메뉴를 사람이 정한 값(D-214).
    pub(crate) fn set_shell_menu(&self, on: Option<bool>) {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        let next = AppSettings {
            shell_menu: on,
            ..*current
        };
        self.replace(&mut current, next);
    }

    fn replace(&self, current: &mut AppSettings, next: AppSettings) {
        if next != *current {
            *current = next;
            if let Some(path) = &self.path
                && let Ok(bytes) = serde_json::to_vec(&SettingsFile::from(next))
            {
                let _ = platform::write_private_file(path, &bytes);
            }
        }
    }
}

fn read(path: &Path) -> Option<AppSettings> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let file: SettingsFile = serde_json::from_slice(&bytes).ok()?;
    (file.v == VERSION).then(|| file.into())
}

impl From<AppSettings> for AppSettingsView {
    fn from(s: AppSettings) -> Self {
        Self {
            allow_published_rewrite: s.allow_published_rewrite,
            language: s.language,
            font_size: s.font_size,
            theme: s.theme,
            term_help: s.term_help,
            diff_wrap: s.diff_wrap,
            workspace_pane: s.workspace_pane,
            conflict_pane: s.conflict_pane,
            files_pane: s.files_pane,
            git_helper: s.git_helper,
            tips_on: s.tips_on,
            tips_seen: s.tips_seen.into(),
        }
    }
}

/// OS 글자 크기·배율·다크(D-147). Core 없이 돈다(Git이 없어도 "Git 받기" 화면이 OS 배율을 따른다).
pub fn display_hints() -> DisplayHintsView {
    let hints = platform::display_hints();
    DisplayHintsView {
        body_px: hints.body_px,
        text_scale_percent: hints.text_scale_percent,
        dark: hints.dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn_on() -> AppSettingsPatch {
        AppSettingsPatch {
            allow_published_rewrite: Some(true),
            ..AppSettingsPatch::default()
        }
    }

    #[test]
    fn defaults() {
        let s = AppSettings::default();
        assert!(!s.allow_published_rewrite);
        assert_eq!(s.language, LanguageSetting::System);
        assert_eq!(s.font_size, FontSizeSetting::Normal);
        assert_eq!(s.theme, ThemeSetting::System);
        assert!(s.term_help);
        assert!(s.diff_wrap);
        assert_eq!(SettingsStore::in_memory().get(), s);
    }

    #[test]
    fn defaults_off_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::open(dir.path());
        assert!(!store.get().allow_published_rewrite);
        assert!(store.set(&turn_on()).allow_published_rewrite);
        let again = SettingsStore::open(dir.path());
        assert!(again.get().allow_published_rewrite);
        let text = std::fs::read_to_string(dir.path().join(FILE)).unwrap();
        assert_eq!(
            text,
            r#"{"v":1,"allow_published_rewrite":true,"language":"system","font_size":"normal","theme":"system","term_help":true,"diff_wrap":true,"git_helper":true}"#
        );
        // 빈 패치는 아무것도 바꾸지 않는다.
        assert!(
            again
                .set(&AppSettingsPatch::default())
                .allow_published_rewrite
        );
    }

    #[test]
    fn display_fields_persist_and_patch_one_at_a_time() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::open(dir.path());
        store.set(&AppSettingsPatch {
            language: Some(LanguageSetting::En),
            font_size: Some(FontSizeSetting::Xlarge),
            theme: Some(ThemeSetting::Dark),
            term_help: Some(false),
            diff_wrap: Some(false),
            ..AppSettingsPatch::default()
        });
        let got = SettingsStore::open(dir.path()).get();
        assert_eq!(got.language, LanguageSetting::En);
        assert_eq!(got.font_size, FontSizeSetting::Xlarge);
        assert_eq!(got.theme, ThemeSetting::Dark);
        assert!(!got.term_help);
        assert!(!got.diff_wrap);
        // 한 칸만 바꿔도 나머지는 그대로.
        let one = store.set(&AppSettingsPatch {
            font_size: Some(FontSizeSetting::Small),
            ..AppSettingsPatch::default()
        });
        assert_eq!(one.font_size, FontSizeSetting::Small);
        assert_eq!(one.language, LanguageSetting::En);
        assert!(!one.allow_published_rewrite);
        let text = std::fs::read_to_string(dir.path().join(FILE)).unwrap();
        assert!(
            text.contains(r#""language":"en","font_size":"small","theme":"dark""#),
            "{text}"
        );
    }

    #[test]
    fn pane_widths_persist_per_screen_and_ignore_odd_values() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::open(dir.path());
        store.set(&AppSettingsPatch {
            workspace_pane: Some(24.5),
            conflict_pane: Some(18.0),
            ..AppSettingsPatch::default()
        });
        let got = SettingsStore::open(dir.path()).get();
        assert_eq!(got.workspace_pane, Some(24.5));
        assert_eq!(got.conflict_pane, Some(18.0));
        assert_eq!(got.files_pane, None);
        // 범위 밖·숫자 아님은 무시하고 지금 값을 둔다.
        for odd in [f32::NAN, f32::INFINITY, 2.0, 500.0] {
            let next = store.set(&AppSettingsPatch {
                workspace_pane: Some(odd),
                ..AppSettingsPatch::default()
            });
            assert_eq!(next.workspace_pane, Some(24.5), "{odd}");
        }
        // 파일에 범위 밖 값이 있으면 그 칸만 기본 폭.
        std::fs::write(
            dir.path().join(FILE),
            r#"{"v":1,"allow_published_rewrite":true,"workspace_pane":3.0,"files_pane":20.0}"#,
        )
        .unwrap();
        let got = SettingsStore::open(dir.path()).get();
        assert!(got.allow_published_rewrite);
        assert_eq!(got.workspace_pane, None);
        assert_eq!(got.files_pane, Some(20.0));
    }

    #[test]
    fn old_v1_file_reads_new_fields_as_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(FILE),
            r#"{"v":1,"allow_published_rewrite":true}"#,
        )
        .unwrap();
        let got = SettingsStore::open(dir.path()).get();
        assert!(got.allow_published_rewrite);
        assert!(got.term_help);
        assert!(got.diff_wrap);
        assert_eq!(got.font_size, FontSizeSetting::Normal);
    }

    #[test]
    fn odd_files_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE);
        let big = format!(
            r#"{{"v":1,"allow_published_rewrite":true,"x":"{}"}}"#,
            "a".repeat(5000)
        );
        for bad in [
            r#"{"v":2,"allow_published_rewrite":true}"#,
            r#"{"v":1,"allow_published_rewrite":true,"extra":1}"#,
            r#"{"v":1,"allow_published_rewrite":true,"font_size":"huge"}"#,
            r#"{"v":1,"allow_published_rewrite":true,"language":"fr"}"#,
            r#"{"v":1,"allow_published_rewrite":true,"term_help":"yes"}"#,
            r#"{"v":1}"#,
            "not json",
            big.as_str(),
        ] {
            std::fs::write(&path, bad).unwrap();
            assert_eq!(
                SettingsStore::open(dir.path()).get(),
                AppSettings::default(),
                "{bad}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn link_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real.json");
        std::fs::write(&real, r#"{"v":1,"allow_published_rewrite":true}"#).unwrap();
        std::os::unix::fs::symlink(&real, dir.path().join(FILE)).unwrap();
        assert!(
            !SettingsStore::open(dir.path())
                .get()
                .allow_published_rewrite
        );
    }

    #[test]
    fn write_failure_keeps_value_in_memory() {
        let dir = tempfile::tempdir().unwrap();
        // 폴더가 없어 쓸 수 없는 경로.
        let store = SettingsStore::open(&dir.path().join("없음"));
        assert!(store.set(&turn_on()).allow_published_rewrite);
        assert!(store.get().allow_published_rewrite);
        let memory = SettingsStore::in_memory();
        assert!(memory.set(&turn_on()).allow_published_rewrite);
    }

    #[test]
    fn display_hints_pass_through_platform() {
        let hints = display_hints();
        assert_eq!(hints.body_px, platform::BODY_PX);
        assert!((50..=300).contains(&hints.text_scale_percent));
    }
}
