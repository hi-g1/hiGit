//! Tauri 명령. 이름은 `<영역>_<동사>`(docs/architecture.md). git 작업은 메인 스레드를
//! 막지 않게 blocking 스레드에서 실행한다.

use std::path::PathBuf;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionMetaView, ActionResult, AppStatus, BranchListView, BranchNameCheck,
    ConflictFileView, ConflictStateView, DiffRequest, ErrorCode, ErrorView, FileDiffView,
    HomeMapView, LogPage, LogQuery, MergeHintView, OpenHow, PendingWorkView, PreviewView,
    ProjectOpenResult, ProjectSummaryView, RecentProject, RecoveryView, SecretCheckRequest,
    SecretCheckView, StashListView, StatusView, TrustChoice, UndoResult,
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

/// 로그 폴더를 파일 관리자로 연다(D-078). 위치는 core가 정하고 화면은 넘기지 않는다.
#[tauri::command]
#[specta::specta]
pub fn app_open_log_folder(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let core = core(&state)?;
    let dir = core.log_dir().ok_or(ErrorView {
        code: ErrorCode::Internal,
        detail: None,
    })?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
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

/// 신뢰 확인에서 고른 결과로 연다(D-039).
#[tauri::command]
#[specta::specta]
pub async fn project_trust(
    state: State<'_, AppState>,
    path: String,
    choice: TrustChoice,
) -> CmdResult<ProjectOpenResult> {
    blocking(state, move |core| {
        core.trust_project(&PathBuf::from(path), choice.into())
    })
    .await
}

/// 주인이 다른 폴더를 믿기로 하고 연다(D-039).
#[tauri::command]
#[specta::specta]
pub async fn project_allow_owner(
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<ProjectOpenResult> {
    blocking(state, move |core| {
        core.allow_folder_owner(&PathBuf::from(path))
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

/// 도는 원격 작업을 취소하거나 멈춘 로컬 쓰기를 강제로 멈춘다(D-080).
#[tauri::command]
#[specta::specta]
pub async fn action_cancel(state: State<'_, AppState>, folder: String) -> CmdResult<()> {
    blocking(state, move |core| core.cancel(&PathBuf::from(folder))).await
}

/// 마지막 작업을 되돌린다(Ctrl+Z, D-042).
#[tauri::command]
#[specta::specta]
pub async fn undo_last(state: State<'_, AppState>, folder: String) -> CmdResult<UndoResult> {
    blocking(state, move |core| core.undo(&PathBuf::from(folder))).await
}

/// 되돌린 작업을 다시 한다(Ctrl+Shift+Z, Ctrl+Y).
#[tauri::command]
#[specta::specta]
pub async fn redo_last(state: State<'_, AppState>, folder: String) -> CmdResult<UndoResult> {
    blocking(state, move |core| core.redo(&PathBuf::from(folder))).await
}

#[tauri::command]
#[specta::specta]
pub async fn query_home_map(state: State<'_, AppState>, repo: String) -> CmdResult<HomeMapView> {
    blocking(state, move |core| {
        core.queries().home_map(&PathBuf::from(repo))
    })
    .await
}

/// 충돌 해결 화면 상태(D-069).
#[tauri::command]
#[specta::specta]
pub async fn query_conflict_state(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<ConflictStateView> {
    blocking(state, move |core| {
        core.queries().conflict_state(&PathBuf::from(repo))
    })
    .await
}

/// 충돌 파일 하나의 고를 곳(D-086).
#[tauri::command]
#[specta::specta]
pub async fn query_conflict_file(
    state: State<'_, AppState>,
    repo: String,
    path: String,
) -> CmdResult<ConflictFileView> {
    blocking(state, move |core| {
        core.queries().conflict_file(&PathBuf::from(repo), &path)
    })
    .await
}

/// 작업 내역(D-085).
#[tauri::command]
#[specta::specta]
pub async fn query_recovery(state: State<'_, AppState>, repo: String) -> CmdResult<RecoveryView> {
    blocking(state, move |core| core.recovery(&PathBuf::from(repo))).await
}

/// 끝나지 않은 작업(D-082). 프로젝트를 열 때 알림 줄에 쓴다.
#[tauri::command]
#[specta::specta]
pub async fn query_pending_work(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<PendingWorkView> {
    blocking(state, move |core| core.pending_work(&PathBuf::from(repo))).await
}

/// 임시 보관 목록(D-076).
#[tauri::command]
#[specta::specta]
pub async fn query_stashes(state: State<'_, AppState>, repo: String) -> CmdResult<StashListView> {
    blocking(state, move |core| {
        core.queries().stashes(&PathBuf::from(repo))
    })
    .await
}

/// 합치기 제안(D-048, D-071).
#[tauri::command]
#[specta::specta]
pub async fn query_merge_hint(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<MergeHintView> {
    blocking(state, move |core| {
        core.queries().merge_hint(&PathBuf::from(repo))
    })
    .await
}

/// 버전 저장 전 비밀 파일 검사(D-077).
#[tauri::command]
#[specta::specta]
pub async fn query_secret_check(
    state: State<'_, AppState>,
    repo: String,
    request: SecretCheckRequest,
) -> CmdResult<SecretCheckView> {
    blocking(state, move |core| {
        core.queries().secret_check(&PathBuf::from(repo), &request)
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
