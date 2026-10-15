//! 명령 등록과 TS 바인딩 생성을 이 파일 하나에 모은다(D-019).
//! 바인딩 라이브러리를 바꿀 때 고칠 곳은 이 파일과 `higit_core::view`뿐이어야 한다.

use std::path::Path;

use higit_core::view::{ActionMetaView, OpProgress, RepoChanged, WatchStatus};
use specta_typescript::Typescript;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands};

use crate::commands;

/// 화면이 부를 수 있는 명령 전체. 앱 실행과 바인딩 생성이 같이 쓴다.
pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        // 실패하면 Promise가 ErrorView로 reject된다. src/lib/api.ts가 ApiError로 바꾼다.
        .error_handling(ErrorHandlingMode::Throw)
        .commands(collect_commands![
            commands::app_status,
            commands::app_open_git_download,
            commands::app_open_log_folder,
            commands::project_open,
            commands::project_trust,
            commands::project_allow_owner,
            commands::recent_list,
            commands::recent_remove,
            commands::action_list,
            commands::action_preview,
            commands::action_execute,
            commands::action_cancel,
            commands::undo_last,
            commands::redo_last,
            commands::query_status,
            commands::query_log,
            commands::query_diff,
            commands::query_branches,
            commands::query_branch_name,
            commands::query_home_map,
            commands::query_secret_check,
            commands::query_merge_hint,
            commands::query_stashes,
            commands::query_recovery,
            commands::query_conflict_state,
            commands::query_conflict_file,
            commands::query_project_summary,
            commands::repo_watch,
            commands::repo_unwatch,
        ])
        // 이벤트는 core가 tauri를 모르도록 typed event 대신 이름 상수 + 타입으로 내보낸다(D-019).
        .typ::<RepoChanged>()
        .typ::<WatchStatus>()
        .typ::<OpProgress>()
        .constant("REPO_CHANGED_EVENT", REPO_CHANGED_EVENT)
        .constant("WATCH_STATUS_EVENT", WATCH_STATUS_EVENT)
        .constant("OP_PROGRESS_EVENT", OP_PROGRESS_EVENT)
}

/// 저장소 파일이 바뀌었을 때(페이로드: `RepoChanged`).
pub const REPO_CHANGED_EVENT: &str = "repo-changed";
/// 감시가 켜지거나 꺼졌을 때(페이로드: `WatchStatus`).
pub const WATCH_STATUS_EVENT: &str = "watch-status";
/// 오래 걸리는 작업이 도는 동안 1초마다(페이로드: `OpProgress`, D-080).
pub const OP_PROGRESS_EVENT: &str = "op-progress";

const HEADER: &str =
    "// 이 파일은 생성물입니다. 고치지 말고 `cargo run -p app --bin gen-bindings`로 다시 만드세요.";

/// `src/lib/bindings.ts`와 `src/lib/action-meta.json`을 쓴다. `frontend_lib`은 `src/lib` 폴더.
pub fn export_all(frontend_lib: &Path) -> Result<(), Box<dyn std::error::Error>> {
    builder().export(
        Typescript::default().header(HEADER),
        frontend_lib.join("bindings.ts"),
    )?;

    let metas: Vec<ActionMetaView> = higit_core::actions::all_metas()
        .iter()
        .map(ActionMetaView::from)
        .collect();
    let json = serde_json::to_string_pretty(&metas)? + "\n";
    std::fs::write(frontend_lib.join("action-meta.json"), json)?;
    Ok(())
}
