// 앱 명령을 선언해 두면 capability에 적은 명령만 웹뷰에서 부를 수 있다(D-037).
// 새 명령은 src/commands.rs, src/bindings.rs, 이 목록, capabilities/default.json 네 곳에 함께 넣는다.
// tests/security_config.rs가 네 곳이 같은지 확인한다.
const COMMANDS: &[&str] = &[
    "app_status",
    "app_open_git_download",
    "app_open_log_folder",
    "app_open_hosting_page",
    "app_display_hints",
    "settings_get",
    "settings_set",
    "query_identity",
    "github_status",
    "github_connect_start",
    "github_connect_poll",
    "github_connect_cancel",
    "github_disconnect",
    "project_open",
    "project_trust",
    "project_allow_owner",
    "recent_list",
    "recent_remove",
    "action_list",
    "action_preview",
    "action_execute",
    "action_cancel",
    "undo_last",
    "redo_last",
    "query_status",
    "query_log",
    "query_log_stats",
    "query_route_preview",
    "query_rewrite_plan",
    "query_force_push_plan",
    "query_remote_branch_plan",
    "query_publish_draft",
    "query_github_repos",
    "query_github_email_check",
    "query_pr_repo",
    "query_pr_comments",
    "query_commit_detail",
    "query_diff",
    "query_branches",
    "query_branch_name",
    "query_home_map",
    "query_secret_check",
    "query_merge_hint",
    "query_stashes",
    "query_recovery",
    "query_pending_work",
    "query_conflict_state",
    "query_conflict_file",
    "query_project_summary",
    "query_clone_defaults",
    "query_hosting_context",
    "query_pr_list",
    "query_pr_detail",
    "query_pr_draft",
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
