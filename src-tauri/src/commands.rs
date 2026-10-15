//! Tauri 명령. 이름은 `<영역>_<동사>`(docs/architecture.md). git 작업은 메인 스레드를
//! 막지 않게 blocking 스레드에서 실행한다.

use std::path::PathBuf;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionMetaView, ActionResult, AppStatus, DiffRequest, ErrorCode, ErrorView,
    FileDiffView, FolderInfo, LogPage, LogQuery, PreviewView, StatusView,
};
use higit_core::{Core, WatchSignal};
use tauri::{AppHandle, Emitter, State};

use crate::AppState;

type CmdResult<T> = Result<T, ErrorView>;

fn core(state: &AppState) -> CmdResult<Arc<Core>> {
    state.0.clone()
}

/// core 작업을 blocking 스레드에서 실행한다.
async fn blocking<T, F>(state: State<'_, AppState>, f: F) -> CmdResult<T>
where
    T: Send + 'static,
    F: FnOnce(&Core) -> higit_core::Result<T> + Send + 'static,
{
    let core = core(&state)?;
    tauri::async_runtime::spawn_blocking(move || f(&core).map_err(ErrorView::from))
        .await
        .unwrap_or_else(|e| {
            Err(ErrorView {
                code: ErrorCode::Internal,
                detail: Some(e.to_string()),
            })
        })
}

/// git을 찾았는지와 버전. 실패하면 GitMissing / GitTooOld.
#[tauri::command]
#[specta::specta]
pub fn app_status(state: State<'_, AppState>) -> CmdResult<AppStatus> {
    let core = core(&state)?;
    Ok(AppStatus {
        git_version: core.git_version().to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn repo_open(state: State<'_, AppState>, path: String) -> CmdResult<FolderInfo> {
    blocking(state, move |core| core.open_folder(&PathBuf::from(path))).await
}

#[tauri::command]
#[specta::specta]
pub fn action_list(state: State<'_, AppState>) -> CmdResult<Vec<ActionMetaView>> {
    Ok(core(&state)?.action_metas())
}

#[tauri::command]
#[specta::specta]
pub async fn action_preview(
    state: State<'_, AppState>,
    folder: String,
    input: ActionInput,
) -> CmdResult<PreviewView> {
    blocking(state, move |core| {
        core.preview(&PathBuf::from(folder), &input)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn action_execute(
    state: State<'_, AppState>,
    folder: String,
    input: ActionInput,
    preview_token: Option<String>,
) -> CmdResult<ActionResult> {
    blocking(state, move |core| {
        core.execute(&PathBuf::from(folder), &input, preview_token)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn query_status(state: State<'_, AppState>, repo: String) -> CmdResult<StatusView> {
    blocking(state, move |core| {
        core.queries().status(&PathBuf::from(repo))
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn query_log(
    state: State<'_, AppState>,
    repo: String,
    query: LogQuery,
) -> CmdResult<LogPage> {
    blocking(state, move |core| {
        core.queries().log(&PathBuf::from(repo), query)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn query_diff(
    state: State<'_, AppState>,
    repo: String,
    request: DiffRequest,
) -> CmdResult<FileDiffView> {
    blocking(state, move |core| {
        core.queries().diff(&PathBuf::from(repo), &request)
    })
    .await
}

/// 저장소 감시를 시작한다. 변경은 `repo-changed`, 감시 상태는 `watch-status` 이벤트로 온다.
#[tauri::command]
#[specta::specta]
pub async fn repo_watch(app: AppHandle, state: State<'_, AppState>, repo: String) -> CmdResult<()> {
    blocking(state, move |core| {
        core.watch(
            &PathBuf::from(repo),
            Arc::new(move |signal| {
                // 화면이 닫혀 이벤트를 못 보내도 감시는 계속한다.
                let _ = match signal {
                    WatchSignal::Changed(payload) => {
                        app.emit(crate::bindings::REPO_CHANGED_EVENT, payload)
                    }
                    WatchSignal::Status(payload) => {
                        app.emit(crate::bindings::WATCH_STATUS_EVENT, payload)
                    }
                };
            }),
        )
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub fn repo_unwatch(state: State<'_, AppState>) -> CmdResult<()> {
    core(&state)?.unwatch();
    Ok(())
}
