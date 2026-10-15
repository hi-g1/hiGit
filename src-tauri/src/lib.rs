//! Tauri 진입점. 명령은 얇게: 입력 검사 → core 호출 → 결과 변환.
//! app은 `higit-core`만 부른다(git, platform을 직접 쓰지 않는다).

#![forbid(unsafe_code)]

pub mod bindings;
mod commands;

use std::sync::Arc;

use higit_core::view::ErrorView;

/// 앱이 시작할 때 만든 core. git이 없거나 오래되었으면 그 오류를 들고 있다가 화면에 보여준다.
pub(crate) struct AppState(pub Result<Arc<higit_core::Core>, ErrorView>);

// Tauri가 창을 띄우지 못하면 이어서 할 일이 없다. 앱의 맨 바깥이라 여기서만 끝낸다.
#[allow(clippy::expect_used)]
pub fn run() {
    let core = higit_core::init().map(Arc::new).map_err(ErrorView::from);
    let builder = bindings::builder();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(core))
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("hiGit을 실행하지 못했습니다");
}
