// 앱 명령을 선언해 두면 capability에 적은 명령만 웹뷰에서 부를 수 있다(D-037).
// 새 명령은 src/commands.rs, src/bindings.rs, 이 목록, capabilities/default.json 네 곳에 함께 넣는다.
// tests/security_config.rs가 네 곳이 같은지 확인한다.
const COMMANDS: &[&str] = &[
    "app_status",
    "app_open_git_download",
    "project_open",
    "recent_list",
    "recent_remove",
    "action_list",
    "action_preview",
    "action_execute",
    "undo_last",
    "redo_last",
    "query_status",
    "query_log",
    "query_diff",
    "query_branches",
    "query_branch_name",
    "query_project_summary",
    "repo_watch",
    "repo_unwatch",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    if let Err(e) = tauri_build::try_build(attributes) {
        // 빌드 스크립트는 실패를 패닉으로 알린다(Cargo 관례).
        #[allow(clippy::panic)]
        {
            panic!("tauri build: {e:#}");
        }
    }
}
