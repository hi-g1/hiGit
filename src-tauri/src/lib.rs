//! Tauri 진입점. 명령은 얇게: 입력 검사 → core 호출 → 결과 변환.
//! app은 `higit-core`만 부른다(git, platform을 직접 쓰지 않는다).

#![forbid(unsafe_code)]

pub mod bindings;
mod commands;

use std::sync::Arc;
use std::time::Duration;

use higit_core::view::ErrorView;

/// 화면이 `app_ready`를 부르지 못해도(JS 오류 등) 이만큼 뒤에는 창을 보인다(D-190).
pub const SHOW_WINDOW_FALLBACK: Duration = Duration::from_secs(3);

/// 앱이 시작할 때 만든 core. git이 없거나 오래되었으면 그 오류를 들고 있다가 화면에 보여준다.
pub(crate) struct AppState(pub Result<Arc<higit_core::Core>, ErrorView>);

// Tauri가 창을 띄우지 못하면 이어서 할 일이 없다. 앱의 맨 바깥이라 여기서만 끝낸다.
#[allow(clippy::expect_used)]
pub fn run() {
    let core = higit_core::init().map(Arc::new).map_err(ErrorView::from);
    let builder = bindings::builder();

    let progress_core = core.as_ref().ok().cloned();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 창은 숨긴 채 만들고 화면이 첫 렌더 뒤 `app_ready`로 보인다. 그래도 안 보이면 스스로 보인다.
            let fallback = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(SHOW_WINDOW_FALLBACK);
                use tauri::Manager;
                if let Some(window) = fallback.get_webview_window("main")
                    && !window.is_visible().unwrap_or(true)
                {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            });
            if let Some(core) = progress_core {
                let handle = app.handle().clone();
                core.set_progress_sink(Arc::new(move |payload| {
                    use tauri::Emitter;
                    let _ = handle.emit(bindings::OP_PROGRESS_EVENT, payload);
                }));
            }
            Ok(())
        })
        .manage(AppState(core))
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("hiGit을 실행하지 못했습니다");
}
