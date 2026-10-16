//! OS 글자 설정 읽기(D-147, D-162). 앱이 루트 글자 크기에 곱할 OS 글자 배율과 다크 여부를 읽는다.
//! Ubuntu의 WebKitGTK는 GNOME 글자 배율(Xft.dpi)로 이미 화면 전체를 키우므로 배율은 100으로 두고
//! 다크만 읽는다(두 번 커지지 않게).
//!
//! 실행 파일은 절대 경로만 쓰고(PATH 바꿔치기 방지) 인자는 상수다. 출력은 숫자 파서로만 쓰고 로그나
//! 화면에 그대로 내보내지 않는다. 읽지 못하면 기본값(배율 100, 다크 모름)으로 내려간다.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// 자식 하나를 기다리는 시간.
const LIMIT: Duration = Duration::from_millis(1500);
/// 이보다 긴 출력은 읽지 않는다.
const MAX_OUTPUT: usize = 256;

/// OS 기본 본문 글자 크기(px, D-138). Windows 14, Ubuntu 15.
pub const BODY_PX: u8 = if cfg!(target_os = "linux") { 15 } else { 14 };

/// 화면 배율을 정하는 OS 값.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayHints {
    /// OS 기본 본문 글자 크기(px).
    pub body_px: u8,
    /// 앱이 곱할 OS 글자 배율(%). Windows 접근성 "텍스트 크기". Ubuntu는 웹뷰가 이미 반영해 100. 모르면 100.
    pub text_scale_percent: u16,
    /// OS가 다크 모드인지. Windows는 웹뷰가 스스로 따르므로 읽지 않는다(`None`).
    pub dark: Option<bool>,
}

impl Default for DisplayHints {
    fn default() -> Self {
        Self {
            body_px: BODY_PX,
            text_scale_percent: 100,
            dark: None,
        }
    }
}

/// OS 글자 설정을 읽는다. 실패하지 않고, 읽지 못한 칸은 기본값이다.
pub fn display_hints() -> DisplayHints {
    #[cfg(target_os = "linux")]
    {
        linux::read()
    }
    #[cfg(windows)]
    {
        windows::read()
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        DisplayHints::default()
    }
}

/// 자식을 띄워 `LIMIT` 안에 끝나면 표준 출력(최대 `MAX_OUTPUT`바이트)을 돌려준다. 넘기면 끝내고 `None`.
#[cfg_attr(not(any(target_os = "linux", windows)), allow(dead_code))]
fn run_limited(mut cmd: Command) -> Option<String> {
    crate::configure_child(&mut cmd);
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + LIMIT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(15)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    if !status.success() {
        return None;
    }
    let mut out = Vec::new();
    child
        .stdout
        .take()?
        .take(MAX_OUTPUT as u64 + 1)
        .read_to_end(&mut out)
        .ok()?;
    if out.len() > MAX_OUTPUT {
        return None;
    }
    String::from_utf8(out).ok()
}

/// gsettings `color-scheme` 값. `'prefer-dark'`면 참, `'default'`·`'prefer-light'`면 거짓.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn parse_color_scheme(text: &str) -> Option<bool> {
    match text.trim() {
        "'prefer-dark'" => Some(true),
        "'default'" | "'prefer-light'" => Some(false),
        _ => None,
    }
}

/// `reg query … /v <name>` 출력에서 `<name>    REG_DWORD    0x96` 줄의 값.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn parse_reg_dword(text: &str, name: &str) -> Option<u32> {
    text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        if parts.next()? != name || parts.next()? != "REG_DWORD" {
            return None;
        }
        let hex = parts.next()?.strip_prefix("0x")?;
        if parts.next().is_some() {
            return None;
        }
        u32::from_str_radix(hex, 16).ok()
    })
}

/// Windows 텍스트 크기(100~225%). 범위 밖이면 `None`.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn text_scale_from_reg(text: &str) -> Option<u16> {
    let value = parse_reg_dword(text, "TextScaleFactor")?;
    (100..=225)
        .contains(&value)
        .then(|| u16::try_from(value).ok())
        .flatten()
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{DisplayHints, parse_color_scheme, run_limited};
    use std::path::Path;
    use std::process::Command;

    const GSETTINGS: &str = "/usr/bin/gsettings";

    fn gsettings(key: &str) -> Option<String> {
        if !Path::new(GSETTINGS).is_file() {
            return None;
        }
        let mut cmd = Command::new(GSETTINGS);
        cmd.args(["get", "org.gnome.desktop.interface", key])
            .env("LC_ALL", "C");
        run_limited(cmd)
    }

    pub(super) fn read() -> DisplayHints {
        // 글자 배율은 WebKitGTK가 Xft.dpi로 이미 반영한다(D-162). 다크만 읽는다.
        DisplayHints {
            dark: gsettings("color-scheme")
                .as_deref()
                .and_then(parse_color_scheme),
            ..DisplayHints::default()
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::{DisplayHints, run_limited, text_scale_from_reg};
    use std::path::PathBuf;
    use std::process::Command;

    fn reg_exe() -> Option<PathBuf> {
        let root = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        let exe = root.join("System32").join("reg.exe");
        exe.is_file().then_some(exe)
    }

    pub(super) fn read() -> DisplayHints {
        let scale = reg_exe().and_then(|exe| {
            let mut cmd = Command::new(exe);
            cmd.args([
                "query",
                r"HKCU\Software\Microsoft\Accessibility",
                "/v",
                "TextScaleFactor",
            ]);
            run_limited(cmd)
        });
        DisplayHints {
            text_scale_percent: scale
                .as_deref()
                .and_then(text_scale_from_reg)
                .unwrap_or(100),
            ..DisplayHints::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_scheme_table() {
        assert_eq!(parse_color_scheme("'prefer-dark'\n"), Some(true));
        assert_eq!(parse_color_scheme("'default'\n"), Some(false));
        assert_eq!(parse_color_scheme("'prefer-light'"), Some(false));
        assert_eq!(parse_color_scheme("prefer-dark"), None);
        assert_eq!(parse_color_scheme(""), None);
    }

    #[test]
    fn reg_dword_table() {
        let out = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Accessibility\r\n    TextScaleFactor    REG_DWORD    0x96\r\n\r\n";
        assert_eq!(parse_reg_dword(out, "TextScaleFactor"), Some(150));
        assert_eq!(text_scale_from_reg(out), Some(150));
        assert_eq!(
            text_scale_from_reg("    TextScaleFactor    REG_DWORD    0x64\r\n"),
            Some(100)
        );
        // 범위 밖, 다른 이름, 다른 형식, 덧붙은 글자
        assert_eq!(
            text_scale_from_reg("    TextScaleFactor    REG_DWORD    0x32\r\n"),
            None
        );
        assert_eq!(
            text_scale_from_reg("    TextScaleFactor    REG_DWORD    0xff\r\n"),
            None
        );
        assert_eq!(
            text_scale_from_reg("    Other    REG_DWORD    0x96\r\n"),
            None
        );
        assert_eq!(
            text_scale_from_reg("    TextScaleFactor    REG_SZ    150\r\n"),
            None
        );
        assert_eq!(
            text_scale_from_reg("    TextScaleFactor    REG_DWORD    0x96 x\r\n"),
            None
        );
        assert_eq!(text_scale_from_reg(""), None);
    }

    #[test]
    fn body_px_follows_os() {
        let want = if cfg!(target_os = "linux") { 15 } else { 14 };
        assert_eq!(BODY_PX, want);
        assert_eq!(DisplayHints::default().body_px, want);
    }

    #[test]
    fn display_hints_never_fails_and_stays_in_range() {
        let hints = display_hints();
        assert_eq!(hints.body_px, BODY_PX);
        assert!((50..=300).contains(&hints.text_scale_percent));
        if cfg!(windows) {
            assert_eq!(hints.dark, None);
        }
        if cfg!(target_os = "linux") {
            assert_eq!(hints.text_scale_percent, 100);
        }
    }

    #[cfg(unix)]
    #[test]
    fn long_output_and_failures_are_none() {
        let mut long = Command::new("/bin/sh");
        long.args(["-c", "printf '%0300d' 0"]);
        assert_eq!(run_limited(long), None);
        let mut short = Command::new("/bin/sh");
        short.args(["-c", "printf '1.25'"]);
        assert_eq!(run_limited(short).as_deref(), Some("1.25"));
        let mut failed = Command::new("/bin/sh");
        failed.args(["-c", "printf '1.25'; exit 1"]);
        assert_eq!(run_limited(failed), None);
    }

    #[cfg(unix)]
    #[test]
    fn slow_child_is_killed() {
        let mut slow = Command::new("/bin/sh");
        slow.args(["-c", "sleep 5"]);
        let started = Instant::now();
        assert_eq!(run_limited(slow), None);
        assert!(started.elapsed() < Duration::from_secs(4));
    }

    /// 이 컴퓨터의 값을 보인다: `cargo test -p platform -- --ignored os_display_hints_print`
    #[test]
    #[ignore = "실제 OS 값을 출력만 한다"]
    fn os_display_hints_print() {
        println!("{:?}", display_hints());
    }
}
