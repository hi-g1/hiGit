// 릴리스 빌드에서 Windows 콘솔 창을 띄우지 않는다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

fn main() {
    // git이 credential helper로 다시 실행한 경우(D-111). 창·플러그인·로그보다 먼저 갈라지고
    // 표준 출력에는 helper 답 말고 아무것도 쓰지 않는다.
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--git-credential")
    {
        std::process::exit(higit_core::git_credential_main());
    }
    app_lib::run();
}
