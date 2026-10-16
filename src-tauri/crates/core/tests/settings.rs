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
