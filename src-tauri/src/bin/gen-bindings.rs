//! 화면용 TS 바인딩과 액션 메타를 만든다: `cargo run -p app --bin gen-bindings`

#![forbid(unsafe_code)]

use std::path::Path;

fn main() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib");
    std::fs::create_dir_all(&lib).expect("src/lib 폴더를 만들지 못했습니다");
    app_lib::bindings::export_all(&lib).expect("바인딩을 만들지 못했습니다");
    println!(
        "{} 에 bindings.ts, action-meta.json을 썼습니다",
        lib.display()
    );
}
