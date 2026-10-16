#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use common::FakeGit;
use higit_core::view::AppSettingsPatch;
use higit_core::{Core, GitBackend};

fn core(dir: &std::path::Path) -> Core {
    Core::with_backend(Arc::new(FakeGit::new()) as Arc<dyn GitBackend>)
        .unwrap()
        .with_settings_dir_for_tests(dir)
}

#[test]
fn published_rewrite_switch_is_off_until_set_and_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let first = core(dir.path());
    assert!(!first.settings().allow_published_rewrite);
    let on = first.set_settings(&AppSettingsPatch {
        allow_published_rewrite: Some(true),
        ..AppSettingsPatch::default()
    });
    assert!(on.allow_published_rewrite);
    assert!(core(dir.path()).settings().allow_published_rewrite);
    let off = first.set_settings(&AppSettingsPatch {
        allow_published_rewrite: Some(false),
        ..AppSettingsPatch::default()
    });
    assert!(!off.allow_published_rewrite);
    assert!(!core(dir.path()).settings().allow_published_rewrite);
}

#[test]
fn default_core_keeps_settings_in_memory() {
    let core = Core::with_backend(Arc::new(FakeGit::new()) as Arc<dyn GitBackend>).unwrap();
    assert!(!core.settings().allow_published_rewrite);
    assert!(
        core.set_settings(&AppSettingsPatch {
            allow_published_rewrite: Some(true),
            ..AppSettingsPatch::default()
        })
        .allow_published_rewrite
    );
}

#[test]
fn git_helper_is_on_by_default_and_for_old_files() {
    let dir = tempfile::tempdir().unwrap();
    // 칸이 없는 옛 파일은 켜짐으로 읽는다(D-187).
    std::fs::write(
        dir.path().join("settings.json"),
        br#"{"v":1,"allow_published_rewrite":false}"#,
    )
    .unwrap();
    let first = core(dir.path());
    assert!(first.settings().git_helper);
    let off = first.set_settings(&AppSettingsPatch {
        git_helper: Some(false),
        ..AppSettingsPatch::default()
    });
    assert!(!off.git_helper);
    assert!(!core(dir.path()).settings().git_helper);
}

/// 처음 하는 일 안내(D-205): 옛 파일은 켬·안 봄, 닫은 안내는 비트로 남고 다시 보기로 지운다.
#[test]
fn tips_default_on_remember_seen_and_reset() {
    use higit_core::view::TipId;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("settings.json"),
        br#"{"v":1,"allow_published_rewrite":false}"#,
    )
    .unwrap();
    let first = core(dir.path());
    let s = first.settings();
    assert!(s.tips_on);
    assert!(!s.tips_seen.first_branch && !s.tips_seen.first_undo);
    // 한 칸 패치는 다른 칸을 바꾸지 않는다.
    let seen = first.set_settings(&AppSettingsPatch {
        tip_seen: Some(TipId::FirstSync),
        ..AppSettingsPatch::default()
    });
    assert!(seen.tips_seen.first_sync && !seen.tips_seen.first_conflict);
    assert!(seen.tips_on && seen.git_helper && seen.term_help);
    first.set_settings(&AppSettingsPatch {
        tip_seen: Some(TipId::FirstUndo),
        ..AppSettingsPatch::default()
    });
    let again = core(dir.path()).settings();
    assert!(again.tips_seen.first_sync && again.tips_seen.first_undo);
    // 끄기와 다시 보기.
    let off = first.set_settings(&AppSettingsPatch {
        tips_on: Some(false),
        ..AppSettingsPatch::default()
    });
    assert!(!off.tips_on && off.tips_seen.first_sync);
    let reset = first.set_settings(&AppSettingsPatch {
        reset_tips: true,
        ..AppSettingsPatch::default()
    });
    assert!(!reset.tips_seen.first_sync && !reset.tips_seen.first_undo);
    assert!(!reset.tips_on, "다시 보기는 켬·끔을 바꾸지 않는다");
}

#[test]
fn unknown_tip_bits_are_dropped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("settings.json"),
        br#"{"v":1,"allow_published_rewrite":false,"tips_seen":255}"#,
    )
    .unwrap();
    let s = core(dir.path()).settings();
    assert!(s.tips_seen.first_branch && s.tips_seen.first_undo);
    // 모르는 비트는 다음 쓰기에서 남지 않는다.
    core(dir.path()).set_settings(&AppSettingsPatch {
        diff_wrap: Some(false),
        ..AppSettingsPatch::default()
    });
    let text = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
    assert!(text.contains(r#""tips_seen":15"#), "{text}");
}

/// 되돌리기 키를 저장하지 못하면 화면에 알린다(D-210). 시험용 Core는 앱 데이터 폴더가 없어 임시 키다.
#[test]
fn a_temporary_undo_key_is_reported() {
    use higit_core::view::UndoKeyState;
    let core = Core::with_backend(Arc::new(FakeGit::new()) as Arc<dyn GitBackend>).unwrap();
    let status = core.app_status();
    assert_eq!(status.undo_key, UndoKeyState::Temporary);
    assert!(status.undo_key_detail.is_some());
    let json = serde_json::to_value(&status).unwrap();
    assert_eq!(json["undoKey"], "temporary");
}
