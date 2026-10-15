pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("hiGit을 실행하지 못했습니다");
}
