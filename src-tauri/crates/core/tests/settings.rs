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

/// 오른쪽 클릭 메뉴 가짜 등록(D-214). 실제 OS는 건드리지 않는다.
mod shell_menu {
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use higit_core::shell_menu::ShellMenuPort;
    use higit_core::view::{ErrorCode, ErrorView, ShellMenuKindView};
    use platform::ShellMenuState;

    #[derive(Default)]
    struct Fake {
        /// 등록된 (실행 파일, 이름).
        installed: Mutex<Option<(PathBuf, String)>>,
        launcher: Mutex<PathBuf>,
        calls: Mutex<Vec<String>>,
    }

    impl Fake {
        fn new() -> Arc<Self> {
            let fake = Self::default();
            *fake.launcher.lock().unwrap() = PathBuf::from("/opt/higit/higit");
            Arc::new(fake)
        }
        fn calls(&self) -> Vec<String> {
            std::mem::take(&mut self.calls.lock().unwrap())
        }
    }

    impl ShellMenuPort for Fake {
        fn kind(&self) -> ShellMenuKindView {
            ShellMenuKindView::NautilusScripts
        }
        fn launcher(&self) -> Option<PathBuf> {
            Some(self.launcher.lock().unwrap().clone())
        }
        fn status(&self, launcher: &Path, label: &str) -> std::io::Result<ShellMenuState> {
            Ok(match &*self.installed.lock().unwrap() {
                None => ShellMenuState::NotInstalled,
                Some((l, n)) if l == launcher && n == label => ShellMenuState::Installed,
                Some(_) => ShellMenuState::Outdated,
            })
        }
        fn install(&self, launcher: &Path, label: &str) -> std::io::Result<()> {
            self.calls.lock().unwrap().push(format!("install {label}"));
            *self.installed.lock().unwrap() = Some((launcher.to_path_buf(), label.to_string()));
            Ok(())
        }
        fn uninstall(&self) -> std::io::Result<()> {
            self.calls.lock().unwrap().push("uninstall".into());
            *self.installed.lock().unwrap() = None;
            Ok(())
        }
    }

    fn core(dir: &Path, fake: &Arc<Fake>) -> higit_core::Core {
        super::core(dir).with_shell_menu_for_tests(fake.clone())
    }

    #[test]
    fn first_release_run_installs_and_remembers() {
        let dir = tempfile::tempdir().unwrap();
        let fake = Fake::new();
        // 개발 빌드는 아직 정하지 않았으면 아무것도 하지 않는다.
        let view = core(dir.path(), &fake)
            .shell_menu_sync("hiGit으로 열기", false)
            .unwrap();
        assert!(!view.installed);
        assert!(fake.calls().is_empty());
        // 릴리스 빌드는 처음 켤 때 등록하고 켬으로 남긴다.
        let view = core(dir.path(), &fake)
            .shell_menu_sync("hiGit으로 열기", true)
            .unwrap();
        assert!(view.installed);
        assert_eq!(view.kind, ShellMenuKindView::NautilusScripts);
        assert_eq!(fake.calls(), ["install hiGit으로 열기"]);
        // 같은 실행 파일·이름이면 다시 등록하지 않는다.
        core(dir.path(), &fake)
            .shell_menu_sync("hiGit으로 열기", true)
            .unwrap();
        assert!(fake.calls().is_empty());
    }

    #[test]
    fn on_reinstalls_when_the_app_or_label_changes() {
        let dir = tempfile::tempdir().unwrap();
        let fake = Fake::new();
        core(dir.path(), &fake)
            .shell_menu_sync("hiGit으로 열기", true)
            .unwrap();
        fake.calls();
        *fake.launcher.lock().unwrap() = PathBuf::from("/새 곳/higit");
        // 개발 빌드여도 사람이 켠 메뉴는 고친다.
        core(dir.path(), &fake)
            .shell_menu_sync("hiGit으로 열기", false)
            .unwrap();
        assert_eq!(fake.calls(), ["install hiGit으로 열기"]);
        core(dir.path(), &fake)
            .shell_menu_sync("Open in hiGit", false)
            .unwrap();
        assert_eq!(fake.calls(), ["install Open in hiGit"]);
    }

    #[test]
    fn turning_off_sticks_across_restarts() {
        let dir = tempfile::tempdir().unwrap();
        let fake = Fake::new();
        let c = core(dir.path(), &fake);
        assert!(c.shell_menu_set(true, "열기").unwrap().installed);
        assert!(!c.shell_menu_set(false, "열기").unwrap().installed);
        assert_eq!(fake.calls(), ["install 열기", "uninstall"]);
        let view = core(dir.path(), &fake)
            .shell_menu_sync("열기", true)
            .unwrap();
        assert!(!view.installed);
        assert!(fake.calls().is_empty(), "끈 메뉴는 다시 켜지 않는다");
        // 화면 설정 칸은 오른쪽 클릭 메뉴 값을 지우지 않는다.
        core(dir.path(), &fake).set_settings(&higit_core::view::AppSettingsPatch {
            term_help: Some(false),
            ..Default::default()
        });
        core(dir.path(), &fake)
            .shell_menu_sync("열기", true)
            .unwrap();
        assert!(fake.calls().is_empty());
    }

    #[test]
    fn labels_are_checked() {
        let dir = tempfile::tempdir().unwrap();
        let fake = Fake::new();
        let c = core(dir.path(), &fake);
        for bad in ["", "  ", "줄\n바꿈", "a/b", &"가".repeat(22)] {
            let err = c.shell_menu_sync(bad, true).unwrap_err();
            assert_eq!(
                ErrorView::from(&err).code,
                ErrorCode::InvalidInput,
                "{bad:?}"
            );
            assert!(c.shell_menu_set(true, bad).is_err());
        }
        assert!(fake.calls().is_empty());
        // 64바이트까지는 된다(한글 21자 = 63바이트).
        c.shell_menu_set(true, &"가".repeat(21)).unwrap();
    }

    #[test]
    fn old_settings_file_reads_as_undecided() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"v":1,"allow_published_rewrite":false}"#,
        )
        .unwrap();
        let fake = Fake::new();
        assert!(
            core(dir.path(), &fake)
                .shell_menu_sync("열기", true)
                .unwrap()
                .installed
        );
        let text = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
        assert!(text.contains(r#""shell_menu":true"#), "{text}");
    }
}
