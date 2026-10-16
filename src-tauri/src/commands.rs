//! Tauri 명령. 이름은 `<영역>_<동사>`(docs/architecture.md). git 작업은 메인 스레드를
//! 막지 않게 blocking 스레드에서 실행한다.

use std::path::PathBuf;
use std::sync::Arc;

use higit_core::view::{
    ActionInput, ActionMetaView, ActionResult, AppSettingsPatch, AppSettingsView, AppStatus,
    BranchListView, BranchNameCheck, CloneDefaultsView, CommitDetailView, CommitStatView,
    ConflictDraftView, ConflictFileView, ConflictStateView, DiffRequest, DisplayHintsView,
    EmailCheckView, ErrorCode, ErrorView, FileDiffView, ForcePushPlanView, GitHubRepoListView,
    GitHubStatusView, HomeMapView, HostingContextView, HostingPage, IdentityView, ImagePairView,
    ImageRequest, LogPage, LogQuery, LoginChallengeView, LoginPollView, MergeHintView, OpenHow,
    PendingWorkView, PrCommentsView, PrDetailView, PrDraftView, PrListRequest, PrListView,
    PrRepoView, PreviewView, ProjectOpenResult, ProjectSummaryView, PublishDraftView,
    RecentProject, RecoveryView, RemoteBranchPlanView, RewritePlanView, ScopeAdd,
    SecretCheckRequest, SecretCheckView, StashListView, StatusView, SwitchPlanView, TrustChoice,
    UndoResult,
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

/// GitHub 페이지를 기본 브라우저로 연다(D-109). 화면은 페이지 종류만 고르고 주소는 core가
/// 상수와 검사한 값으로 만든다. 검토 요청은 열린 저장소(`folder`)만.
#[tauri::command]
#[specta::specta]
pub async fn app_open_hosting_page(
    app: AppHandle,
    state: State<'_, AppState>,
    folder: Option<String>,
    page: HostingPage,
) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let url = blocking(state, move |core| {
        core.hosting_page_url(folder.as_deref().map(std::path::Path::new), page)
    })
    .await?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| ErrorView {
            code: ErrorCode::Internal,
            detail: Some(e.to_string()),
        })
}

/// 화면이 첫 렌더를 마쳤다(D-190). 숨겨 둔 창을 보이고 포커스를 준다. 이미 보이면 아무것도 하지 않고
/// 앱 상태를 바꾸지 않는다.
#[tauri::command]
#[specta::specta]
pub async fn app_ready(window: tauri::WebviewWindow) -> CmdResult<()> {
    if !window.is_visible().unwrap_or(true) {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

/// OS 글자 크기·배율·다크(D-147). 저장소를 받지 않는 읽기이고 Core 없이 돈다(Git이 없어도 쓴다).
#[tauri::command]
#[specta::specta]
pub async fn app_display_hints() -> CmdResult<DisplayHintsView> {
    tauri::async_runtime::spawn_blocking(higit_core::settings::display_hints)
        .await
        .map_err(|e| ErrorView {
            code: ErrorCode::Internal,
            detail: Some(e.to_string()),
        })
}

/// 전역 이름·이메일(D-125). 저장소를 받지 않는 읽기.
#[tauri::command]
#[specta::specta]
pub async fn query_identity(state: State<'_, AppState>) -> CmdResult<IdentityView> {
    blocking(state, |core| core.identity()).await
}

/// 앱 설정(D-127).
#[tauri::command]
#[specta::specta]
pub async fn settings_get(state: State<'_, AppState>) -> CmdResult<AppSettingsView> {
    blocking(state, |core| Ok(core.settings())).await
}

/// 앱 설정의 알려진 칸만 바꾼다.
#[tauri::command]
#[specta::specta]
pub async fn settings_set(
    state: State<'_, AppState>,
    patch: AppSettingsPatch,
) -> CmdResult<AppSettingsView> {
    blocking(state, move |core| Ok(core.set_settings(&patch))).await
}

/// 설정의 GitHub 구역 상태. 키체인을 건드리지 않는다.
#[tauri::command]
#[specta::specta]
pub async fn github_status(state: State<'_, AppState>) -> CmdResult<GitHubStatusView> {
    blocking(state, |core| Ok(core.github_status())).await
}

/// GitHub 로그인(device flow)을 시작한다.
#[tauri::command]
#[specta::specta]
pub async fn github_connect_start(
    state: State<'_, AppState>,
    add: Option<ScopeAdd>,
) -> CmdResult<LoginChallengeView> {
    blocking(state, move |core| core.github_connect_start(add)).await
}

/// 전역 이메일이 GitHub 계정의 것인지(D-186). 결과 종류만 받는다.
#[tauri::command]
#[specta::specta]
pub async fn query_github_email_check(state: State<'_, AppState>) -> CmdResult<EmailCheckView> {
    blocking(state, |core| core.github_email_check()).await
}

/// 내 GitHub 저장소 목록 한 쪽(D-133). 인자는 쪽 번호뿐이다.
#[tauri::command]
#[specta::specta]
pub async fn query_github_repos(
    state: State<'_, AppState>,
    page: u32,
) -> CmdResult<GitHubRepoListView> {
    blocking(state, move |core| core.github_repos(page)).await
}

/// 승인됐는지 묻는다. 간격 안이면 core가 네트워크 없이 답한다.
#[tauri::command]
#[specta::specta]
pub async fn github_connect_poll(state: State<'_, AppState>) -> CmdResult<LoginPollView> {
    blocking(state, |core| core.github_connect_poll()).await
}

#[tauri::command]
#[specta::specta]
pub async fn github_connect_cancel(state: State<'_, AppState>) -> CmdResult<()> {
    blocking(state, |core| {
        core.github_connect_cancel();
        Ok(())
    })
    .await
}

/// 이 컴퓨터에서 GitHub 연결을 끊는다.
#[tauri::command]
#[specta::specta]
pub async fn github_disconnect(state: State<'_, AppState>) -> CmdResult<()> {
    blocking(state, |core| core.github_disconnect()).await
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

/// 실행 인자로 받은 열 폴더를 한 번만 준다(D-213). 화면은 그 경로를 `project_open(path, "picked")`으로 연다.
#[tauri::command]
#[specta::specta]
pub fn app_take_launch_folder(launch: State<'_, crate::LaunchState>) -> CmdResult<Option<String>> {
    Ok(launch.take().map(|p| p.to_string_lossy().into_owned()))
}

/// 끄고 연 저장소의 신뢰 확인을 다시 띄운다("믿고 다시 열기", D-212). 열린 저장소만.
#[tauri::command]
#[specta::specta]
pub async fn project_reask_trust(
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<ProjectOpenResult> {
    blocking(state, move |core| core.reask_trust(&PathBuf::from(path))).await
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

/// 최근 목록의 프로젝트 고정(D-203).
#[tauri::command]
#[specta::specta]
pub fn recent_pin(state: State<'_, AppState>, path: String, pinned: bool) -> CmdResult<()> {
    Ok(core(&state)?.pin_project(&PathBuf::from(path), pinned)?)
}

/// 열린 저장소의 브랜치 고정(D-203). 고정은 지금 있는 로컬 브랜치만.
#[tauri::command]
#[specta::specta]
pub async fn recent_pin_branch(
    state: State<'_, AppState>,
    repo: String,
    branch: String,
    pinned: bool,
) -> CmdResult<()> {
    blocking(state, move |core| {
        core.pin_branch(&PathBuf::from(repo), &branch, pinned)
    })
    .await
}

/// 열린 저장소의 호스팅 정보(네트워크 없음).
#[tauri::command]
#[specta::specta]
pub async fn query_hosting_context(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<HostingContextView> {
    blocking(state, move |core| {
        core.queries().hosting_context(&PathBuf::from(repo))
    })
    .await
}

/// 검토 요청 목록 한 쪽.
#[tauri::command]
#[specta::specta]
pub async fn query_pr_list(
    state: State<'_, AppState>,
    repo: String,
    request: PrListRequest,
) -> CmdResult<PrListView> {
    blocking(state, move |core| {
        core.queries().pr_list(&PathBuf::from(repo), &request)
    })
    .await
}

/// 검토 요청 화면의 저장소 정보(올릴 권한·합치기 방법, D-177).
#[tauri::command]
#[specta::specta]
pub async fn query_pr_repo(state: State<'_, AppState>, repo: String) -> CmdResult<PrRepoView> {
    blocking(state, move |core| {
        core.queries().pr_repo(&PathBuf::from(repo))
    })
    .await
}

/// 검토 요청의 최근 댓글(D-178). `total`은 상세의 대화 댓글 수.
#[tauri::command]
#[specta::specta]
pub async fn query_pr_comments(
    state: State<'_, AppState>,
    repo: String,
    number: u32,
    total: u32,
) -> CmdResult<PrCommentsView> {
    blocking(state, move |core| {
        core.queries()
            .pr_comments(&PathBuf::from(repo), number, total)
    })
    .await
}

/// 검토 요청 하나(합칠 수 있는지, 시험 결과).
#[tauri::command]
#[specta::specta]
pub async fn query_pr_detail(
    state: State<'_, AppState>,
    repo: String,
    number: u32,
) -> CmdResult<PrDetailView> {
    blocking(state, move |core| {
        core.queries().pr_detail(&PathBuf::from(repo), number)
    })
    .await
}

/// 새 검토 요청 대화상자의 기본값(네트워크 없음).
#[tauri::command]
#[specta::specta]
pub async fn query_pr_draft(state: State<'_, AppState>, repo: String) -> CmdResult<PrDraftView> {
    blocking(state, move |core| {
        core.queries().pr_draft(&PathBuf::from(repo))
    })
    .await
}

/// 복제 대화상자의 기본값(부모 폴더 = 홈 폴더).
#[tauri::command]
#[specta::specta]
pub async fn query_clone_defaults(state: State<'_, AppState>) -> CmdResult<CloneDefaultsView> {
    blocking(state, |core| Ok(core.queries().clone_defaults())).await
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

/// 시작 화면 노선도(D-093). 홈과 같은 요약이고 지금 줄 버전 수만 `limit`.
#[tauri::command]
#[specta::specta]
pub async fn query_route_preview(
    state: State<'_, AppState>,
    repo: String,
    limit: u32,
) -> CmdResult<HomeMapView> {
    blocking(state, move |core| {
        core.queries().route_preview(&PathBuf::from(repo), limit)
    })
    .await
}

/// 버전 상세(D-105). 메시지·작성자·바뀐 파일 500개까지.
#[tauri::command]
#[specta::specta]
pub async fn query_commit_detail(
    state: State<'_, AppState>,
    repo: String,
    oid: String,
) -> CmdResult<CommitDetailView> {
    blocking(state, move |core| {
        core.queries().commit_detail(&PathBuf::from(repo), &oid)
    })
    .await
}

/// 덮어쓰며 올리기 확인 창의 값(D-128). 네트워크 없이 읽기만 한다.
#[tauri::command]
#[specta::specta]
pub async fn query_force_push_plan(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<ForcePushPlanView> {
    blocking(state, move |core| {
        core.force_push_plan(&PathBuf::from(repo))
    })
    .await
}

/// GitHub에 올리기 대화상자의 처음 값(D-132). 네트워크 없이 읽기만 한다.
#[tauri::command]
#[specta::specta]
pub async fn query_branch_protection(
    state: State<'_, AppState>,
    repo: String,
    remote: String,
    branch: String,
) -> CmdResult<Option<bool>> {
    blocking(state, move |core| {
        core.queries()
            .branch_protection(&PathBuf::from(repo), &remote, &branch)
    })
    .await
}

/// GitHub에 올리기 대화상자의 처음 값(D-132).
#[tauri::command]
#[specta::specta]
pub async fn query_publish_draft(
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<PublishDraftView> {
    blocking(state, move |core| {
        core.queries().publish_draft(&PathBuf::from(repo))
    })
    .await
}

/// 이미지 비교의 두 쪽(D-206). 읽기만 하고 한쪽 3MB까지.
#[tauri::command]
#[specta::specta]
pub async fn query_image_pair(
    state: State<'_, AppState>,
    repo: String,
    request: ImageRequest,
) -> CmdResult<ImagePairView> {
    blocking(state, move |core| {
        core.queries().image_pair(&PathBuf::from(repo), &request)
    })
    .await
}

/// 브랜치 바꾸기 대화상자의 값(D-195). 고친 파일 수와 새 브랜치로 옮길 수 있는지. 네트워크 없이 읽기만 한다.
#[tauri::command]
#[specta::specta]
pub async fn query_switch_plan(
    state: State<'_, AppState>,
    repo: String,
    branch: String,
    remote: Option<String>,
) -> CmdResult<SwitchPlanView> {
    blocking(state, move |core| {
        core.queries()
            .switch_plan(&PathBuf::from(repo), &branch, remote.as_deref())
    })
    .await
}

/// 원격 브랜치 지우기·이름 바꾸기 확인 창의 값(D-130). 네트워크 없이 읽기만 한다.
#[tauri::command]
#[specta::specta]
pub async fn query_remote_branch_plan(
    state: State<'_, AppState>,
    repo: String,
    remote: String,
    branch: String,
) -> CmdResult<RemoteBranchPlanView> {
    blocking(state, move |core| {
        core.queries()
            .remote_branch_plan(&PathBuf::from(repo), &remote, &branch)
    })
    .await
}

/// 기록 정리 카드 지도의 처음 모습(D-106). 고른 버전 50개까지.
#[tauri::command]
#[specta::specta]
pub async fn query_rewrite_plan(
    state: State<'_, AppState>,
    repo: String,
    oids: Vec<String>,
) -> CmdResult<RewritePlanView> {
    blocking(state, move |core| {
        core.rewrite_plan(&PathBuf::from(repo), oids)
    })
    .await
}

/// 버전별 바뀐 줄 수(노선도 선 굵기, D-089). 한 번에 100개까지.
#[tauri::command]
#[specta::specta]
pub async fn query_log_stats(
    state: State<'_, AppState>,
    repo: String,
    oids: Vec<String>,
) -> CmdResult<Vec<CommitStatView>> {
    blocking(state, move |core| {
        core.queries().log_stats(&PathBuf::from(repo), oids)
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
    blocking(state, move |core| core.conflict_state(&PathBuf::from(repo))).await
}

/// 충돌 해결 화면에서 고른 것을 저장소 안에 남긴다(D-211). 화면은 500ms 모아 보낸다.
#[tauri::command]
#[specta::specta]
pub async fn conflict_draft_save(
    state: State<'_, AppState>,
    repo: String,
    draft: ConflictDraftView,
) -> CmdResult<()> {
    blocking(state, move |core| {
        core.save_conflict_draft(&PathBuf::from(repo), &draft)
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
