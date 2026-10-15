//! Tauri 명령. 이름은 `<영역>_<동사>`(docs/architecture.md). git 작업은 메인 스레드를
//! 막지 않게 blocking 스레드에서 실행한다.

use std::path::PathBuf;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionMetaView, ActionResult, AppStatus, BranchListView, BranchNameCheck,
    DiffRequest, ErrorCode, ErrorView, FileDiffView, LogPage, LogQuery, OpenHow, PreviewView,
    ProjectOpenResult, ProjectSummaryView, RecentProject, StatusView,
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
    Ok(core(&state)?.app_status())
}

/// Git 내려받기 페이지를 기본 브라우저로 연다(D-053). 주소는 core가 정하고 화면은 넘기지 않는다.
#[tauri::command]
#[specta::specta]
pub fn app_open_git_download(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let url = core(&state)?.git_download_url();
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| ErrorView {
            code: ErrorCode::Internal,
            detail: Some(e.to_string()),
        })
}

#[tauri::command]
#[specta::specta]
pub async fn project_open(
    state: State<'_, AppState>,
    path: String,
    how: OpenHow,
) -> CmdResult<ProjectOpenResult> {
    // 프로젝트 열기의 유일한 입구(D-032).
    blocking(state, move |core| {
        core.open_project(&PathBuf::from(path), how)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub fn recent_list(state: State<'_, AppState>) -> CmdResult<Vec<RecentProject>> {
    Ok(core(&state)?.recent_projects())
}

#[tauri::command]
#[specta::specta]
pub fn recent_remove(state: State<'_, AppState>, path: String) -> CmdResult<()> {
    core(&state)?.remove_recent(&PathBuf::from(path));
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn query_project_summary(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<ProjectSummaryView> {
    blocking(state, move |core| {
        core.queries().project_summary(&PathBuf::from(repo))
    })
    .await
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

#[tauri::command]
#[specta::specta]
pub async fn query_branches(state: State<'_, AppState>, repo: String) -> CmdResult<BranchListView> {
    blocking(state, move |core| {
        core.queries().branches(&PathBuf::from(repo))
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn query_branch_name(
    state: State<'_, AppState>,
    repo: String,
    name: String,
) -> CmdResult<BranchNameCheck> {
    blocking(state, move |core| {
        core.queries()
            .check_branch_name(&PathBuf::from(repo), &name)
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
