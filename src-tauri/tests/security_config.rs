//! 웹뷰 보안 설정을 고정한다(D-037, D-052). 설정 파일을 읽어 비교하므로 앱을 띄우지 않는다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

fn read(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn json(rel: &str) -> serde_json::Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src ipc: http://ipc.localhost; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'";

#[test]
fn csp_is_fixed_and_has_no_remote_sources() {
    let conf = json("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"].as_str().unwrap();
    assert_eq!(csp, CSP);
    for bad in ["unsafe-eval", "https:", "http:*", "*"] {
        let found = csp
            .split([' ', ';'])
            .any(|t| t == bad || (bad == "https:" && t.starts_with("https:")));
        assert!(!found, "{bad}");
    }
    let global = &conf["app"]["withGlobalTauri"];
    assert!(global.is_null() || global == &serde_json::Value::Bool(false));
}

/// 글자 크기는 앱이 Ctrl+=·−·0으로 바꾸므로 웹뷰 자체 확대는 끈다(D-152). Linux의 확대 폴리필은
/// 확대 권한이 있어야 돌므로 그 권한도 열지 않는다.
#[test]
fn webview_zoom_stays_off() {
    let conf = json("tauri.conf.json");
    let windows = conf["app"]["windows"].as_array().unwrap();
    assert!(!windows.is_empty());
    for w in windows {
        assert_eq!(w["zoomHotkeysEnabled"], serde_json::Value::Bool(false));
        // 첫 그림 전 흰 화면을 보이지 않게 숨긴 채 만든다(D-190). 화면이 `app_ready`로 보인다.
        assert_eq!(w["visible"], serde_json::Value::Bool(false));
    }
    let cap = json("capabilities/default.json");
    for p in cap["permissions"].as_array().unwrap() {
        let p = p.as_str().unwrap();
        assert!(
            !(p.contains("webview-zoom") || p == "core:webview:default"),
            "{p}"
        );
    }
}

#[test]
fn capability_has_no_broad_permissions() {
    let cap = json("capabilities/default.json");
    let perms: Vec<&str> = cap["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    for p in &perms {
        assert!(
            !(p.starts_with("fs:")
                || p.starts_with("shell:")
                || p.starts_with("opener:")
                || p.starts_with("window-state:")
                || p.starts_with("single-instance:")),
            "{p}"
        );
        assert_ne!(*p, "core:default");
    }
}

/// `src/bindings.rs`의 `commands::이름` 목록.
fn registered_commands() -> BTreeSet<String> {
    read("src/bindings.rs")
        .split("commands::")
        .skip(1)
        .filter_map(|rest| {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

/// `src/commands.rs`에서 `#[tauri::command]` 바로 뒤의 함수 이름.
fn defined_commands() -> BTreeSet<String> {
    let src = read("src/commands.rs");
    let lines: Vec<&str> = src.lines().collect();
    let mut found = BTreeSet::new();
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().starts_with("#[tauri::command") {
            continue;
        }
        let next = lines[i + 1..]
            .iter()
            .find(|l| l.contains("fn "))
            .expect("명령 뒤에 함수");
        let after = next.split("fn ").nth(1).unwrap();
        found.insert(after.split('(').next().unwrap().trim().to_string());
    }
    found
}

/// `build.rs`의 `COMMANDS` 목록.
fn manifest_commands() -> BTreeSet<String> {
    let src = read("build.rs");
    let start = src.find("const COMMANDS").unwrap();
    let end = src[start..].find("];").unwrap() + start;
    src[start..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

fn capability_commands() -> BTreeSet<String> {
    json("capabilities/default.json")["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p.as_str()?.strip_prefix("allow-"))
        .map(|p| p.replace('-', "_"))
        .collect()
}

#[test]
fn commands_match_in_all_four_places() {
    let defined = defined_commands();
    assert!(defined.len() >= 10, "{defined:?}");
    assert_eq!(registered_commands(), defined, "src/bindings.rs");
    assert_eq!(manifest_commands(), defined, "build.rs");
    assert_eq!(capability_commands(), defined, "capabilities/default.json");
}

#[test]
fn credential_helper_and_single_instance_come_first() {
    // git이 helper로 부른 실행은 창·플러그인보다 먼저 갈라진다(D-111). 단일 인스턴스가 그 실행을 기존 창으로
    // 넘기지 않게 한다(D-213).
    let main = read("src/main.rs");
    let helper = main
        .find("--git-credential")
        .expect("main.rs의 helper 분기");
    let run = main.find("app_lib::run()").expect("main.rs의 run()");
    assert!(helper < run, "helper 분기가 run()보다 앞");
    // 첫 플러그인은 단일 인스턴스(두 번째 실행이 다른 플러그인을 띄우기 전에 끝난다).
    let lib = read("src/lib.rs");
    let first = lib.find(".plugin(").expect("lib.rs의 플러그인");
    assert!(
        lib[first..].starts_with(".plugin(tauri_plugin_single_instance::init("),
        "첫 플러그인: {}",
        &lib[first..first + 60]
    );
}

#[test]
fn window_minimum_size_matches_the_fit_rule() {
    // 창 기억(D-217)이 되살린 크기를 고칠 때 쓰는 최소 크기와 같다.
    let conf = json("tauri.conf.json");
    let window = &conf["app"]["windows"][0];
    assert_eq!(window["minWidth"], 800);
    assert_eq!(window["minHeight"], 560);
    assert!(read("src/window_fit.rs").contains("MIN_LOGICAL: (u32, u32) = (800, 560)"));
}

#[test]
fn installers_are_user_nsis_deb_and_appimage() {
    // D-220: Windows는 관리자 확인 없는 사용자 단위 NSIS(한국어·English), Linux는 deb·AppImage.
    let conf = json("tauri.conf.json");
    let bundle = &conf["bundle"];
    assert_eq!(
        bundle["targets"],
        serde_json::json!(["nsis", "deb", "appimage"])
    );
    let nsis = &bundle["windows"]["nsis"];
    assert_eq!(nsis["installMode"], "currentUser");
    assert_eq!(nsis["languages"], serde_json::json!(["Korean", "English"]));
    assert_eq!(
        bundle["linux"]["deb"]["depends"],
        serde_json::json!(["git"])
    );
    // 서명 설정은 아직 없다(D-221, 자리만).
    assert!(bundle["windows"].get("certificateThumbprint").is_none());
    assert!(bundle["windows"].get("signCommand").is_none());
    // 라이선스 원문을 함께 넣는다.
    let resources = bundle["resources"].as_object().unwrap();
    for name in [
        "LICENSE",
        "THIRD_PARTY_NOTICES.md",
        "licenses/Pretendard-OFL.txt",
        "licenses/JetBrainsMono-OFL.txt",
        "licenses/D2Coding-OFL.txt",
    ] {
        assert!(resources.values().any(|v| v == name), "{name}");
    }
}

#[test]
fn desktop_entry_does_not_claim_folders() {
    // 폴더 열기는 오른쪽 클릭 메뉴로만 한다. `.desktop`이 폴더 형식을 맡으면 파일 앱의 기본 앱이 바뀐다.
    let desktop = read("../packaging/linux/higit.desktop");
    assert!(!desktop.contains("MimeType"), "{desktop}");
    assert!(desktop.contains("Categories=Development;RevisionControl;"));
}

#[test]
fn uninstaller_removes_only_the_shell_menu_keys() {
    let hooks = read("../packaging/windows/installer-hooks.nsh");
    let deletes: Vec<&str> = hooks
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("Delete") || l.starts_with("Write") || l.starts_with("Exec"))
        .collect();
    assert_eq!(
        deletes,
        [
            r#"DeleteRegKey HKCU "Software\Classes\Directory\shell\higit""#,
            r#"DeleteRegKey HKCU "Software\Classes\Directory\Background\shell\higit""#,
        ]
    );
}
