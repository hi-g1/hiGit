//! Tauri 진입점. 명령은 얇게: 입력 검사 → core 호출 → 결과 변환.
//! app은 `higit-core`만 부른다(git, platform을 직접 쓰지 않는다).

#![forbid(unsafe_code)]

pub mod bindings;
mod commands;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use higit_core::view::ErrorView;

/// 화면이 `app_ready`를 부르지 못해도(JS 오류 등) 이만큼 뒤에는 창을 보인다(D-190).
pub const SHOW_WINDOW_FALLBACK: Duration = Duration::from_secs(3);

/// 앱이 시작할 때 만든 core. git이 없거나 오래되었으면 그 오류를 들고 있다가 화면에 보여준다.
pub(crate) struct AppState(pub Result<Arc<higit_core::Core>, ErrorView>);

/// 실행 인자로 받은 열 폴더(D-213). 첫 실행의 인자와 두 번째 실행이 넘긴 인자가 들어오고, 화면이
/// `app_take_launch_folder`로 한 번만 가져간다.
#[derive(Default)]
pub(crate) struct LaunchState(Mutex<Option<PathBuf>>);

impl LaunchState {
    fn put(&self, path: PathBuf) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(path);
    }

    pub(crate) fn take(&self) -> Option<PathBuf> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

/// 숨긴 창을 보이고 앞으로 가져온다(안전 타이머와 두 번째 실행이 같이 쓴다).
fn show_main(app: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// Tauri가 창을 띄우지 못하면 이어서 할 일이 없다. 앱의 맨 바깥이라 여기서만 끝낸다.
#[allow(clippy::expect_used)]
pub fn run() {
    let core = higit_core::init().map(Arc::new).map_err(ErrorView::from);
    let builder = bindings::builder();

    let progress_core = core.as_ref().ok().cloned();
    let launch = LaunchState::default();
    let cwd = std::env::current_dir().unwrap_or_default();
    if let Some(path) =
        higit_core::launch::parse_args(&std::env::args_os().collect::<Vec<_>>(), &cwd)
    {
        launch.put(path);
    }

    tauri::Builder::default()
        // 첫 플러그인이어야 두 번째 실행이 다른 플러그인을 띄우기 전에 끝난다. 콜백은 core를 부르지 않는다:
        // 열기는 화면이 `project_open`으로 해서 신뢰 확인을 거친다(D-213).
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            use tauri::{Emitter, Manager};
            show_main(app);
            if let Some(path) = higit_core::launch::parse_args(&argv, Path::new(&cwd)) {
                app.state::<LaunchState>().put(path);
                let _ = app.emit(bindings::OPEN_FOLDER_EVENT, ());
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 창은 숨긴 채 만들고 화면이 첫 렌더 뒤 `app_ready`로 보인다. 그래도 안 보이면 스스로 보인다.
            let fallback = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(SHOW_WINDOW_FALLBACK);
                use tauri::Manager;
                if fallback
                    .get_webview_window("main")
                    .is_some_and(|w| !w.is_visible().unwrap_or(true))
                {
                    show_main(&fallback);
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
        .manage(launch)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("hiGit을 실행하지 못했습니다");
}
