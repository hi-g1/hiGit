//! "폴더에서 보기"와 "기본 앱으로 열기"에 넘길 파일 경로를 검사한다(D-216).
//!
//! 화면은 열린 저장소 기준 상대 경로만 보낸다. 여기서 저장소 밖·`.git` 안·링크·없는 파일을 거절하고, 열기는 실행될 수
//! 있는 파일(차단표 확장자, 실행 파일 앞 바이트, 실행 권한 비트)을 거절한다. OS에 넘기는 일은 app이 한다.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::{CoreError, Result};

/// 무엇을 하려는지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// 파일 관리자에서 그 파일을 고른 채 연다. 있는 파일·폴더면 된다.
    Reveal,
    /// OS 기본 앱으로 연다. 실행될 수 없는 일반 파일만.
    Open,
}

/// 기본 앱으로 열지 않는 확장자(소문자, 마지막 확장자). Windows "위험한 첨부 파일 형식"과 실행 스크립트·설치
/// 패키지. 화면은 바인딩 상수로 받아 메뉴 항목을 미리 끈다.
pub const OPEN_BLOCKED_EXTENSIONS: &[&str] = &[
    // Windows 실행·설치
    "exe",
    "com",
    "bat",
    "cmd",
    "msi",
    "msp",
    "mst",
    "msix",
    "msixbundle",
    "appx",
    "appxbundle",
    "appinstaller",
    "scr",
    "pif",
    "cpl",
    "dll",
    "sys",
    "drv",
    "ocx",
    "inf",
    "ins",
    "isp",
    "msc",
    "reg",
    "lnk",
    "url",
    "scf",
    "hta",
    "chm",
    "hlp",
    "gadget",
    "application",
    "appref-ms",
    "settingcontent-ms",
    "library-ms",
    "search-ms",
    "searchconnector-ms",
    "diagcab",
    "diagpkg",
    "xbap",
    "website",
    "shb",
    "shs",
    "sct",
    "jnlp",
    "iso",
    "img",
    "vhd",
    "vhdx",
    // Windows 스크립트 호스트·PowerShell
    "js",
    "jse",
    "vb",
    "vbs",
    "vbe",
    "wsf",
    "wsh",
    "wsc",
    "ws",
    "ps1",
    "ps1xml",
    "ps2",
    "ps2xml",
    "psc1",
    "psc2",
    "psm1",
    "psd1",
    // Unix 셸·실행·설치
    "sh",
    "bash",
    "zsh",
    "csh",
    "ksh",
    "fish",
    "command",
    "run",
    "bin",
    "out",
    "desktop",
    "appimage",
    "deb",
    "rpm",
    "snap",
    "flatpakref",
    // 인터프리터가 실행하는 것
    "py",
    "pyw",
    "pl",
    "rb",
    "php",
    "jar",
    // Windows "위험한 첨부 파일 형식"의 나머지와 매크로·연결·원격 접속 파일(반복 5 끝 점검)
    "ade",
    "adp",
    "app",
    "asp",
    "aspx",
    "asx",
    "bas",
    "bgi",
    "cab",
    "cer",
    "cnt",
    "crt",
    "der",
    "fxp",
    "grp",
    "hpj",
    "htc",
    "its",
    "job",
    "mad",
    "maf",
    "mag",
    "mam",
    "maq",
    "mar",
    "mas",
    "mat",
    "mau",
    "mav",
    "maw",
    "mcf",
    "mda",
    "mdb",
    "mde",
    "mdt",
    "mdw",
    "mdz",
    "msh",
    "msh1",
    "msh2",
    "msh1xml",
    "msh2xml",
    "mshxml",
    "msu",
    "ops",
    "osd",
    "pcd",
    "plg",
    "prf",
    "prg",
    "printerexport",
    "psdm1",
    "pst",
    "pyc",
    "pyo",
    "pyz",
    "pyzw",
    "rdp",
    "theme",
    "themepack",
    "deskthemepack",
    "tmp",
    "vbp",
    "vsmacros",
    "vsto",
    "vsw",
    "webpnp",
    "wsb",
    "xll",
    "wll",
    "xnk",
    "slk",
    "iqy",
    "mht",
    "mhtml",
    "docm",
    "dotm",
    "xlsm",
    "xltm",
    "xlam",
    "pptm",
    "potm",
    "ppam",
    "ppsm",
    "sldm",
];

/// 경로 글자 상한(`check_path_len`과 같다).
const MAX_PATH_BYTES: usize = 4096;

fn refused(why: &'static str) -> CoreError {
    CoreError::OpenRefused(why)
}

/// `root`(열린 저장소의 정규화된 최상위) 아래 `rel`을 검사하고 OS에 넘길 절대 경로를 돌려준다.
pub(crate) fn resolve(root: &Path, rel: &str, purpose: Purpose) -> Result<PathBuf> {
    if rel.is_empty() || rel.len() > MAX_PATH_BYTES || rel.contains('\0') {
        return Err(refused("path"));
    }
    // git 경로는 늘 `/`. Windows의 `\`도 구분자로 보고, 드라이브·절대 경로는 거절한다.
    // 드라이브 문자(`C:`)는 어느 OS에서 왔든 절대 경로로 본다.
    let drive = rel.as_bytes().get(1) == Some(&b':') && rel.as_bytes()[0].is_ascii_alphabetic();
    if drive
        || rel.starts_with(['/', '\\'])
        || Path::new(rel).is_absolute()
        || Path::new(rel).has_root()
    {
        return Err(refused("absolute"));
    }
    // 구성요소를 모두 먼저 본다(없는 앞 폴더에서 "없음"으로 끝나 뒤의 장치 이름을 놓치지 않게).
    let parts: Vec<&str> = rel.split(['/', '\\']).collect();
    for part in &parts {
        let bad = part.is_empty()
            || *part == "."
            || *part == ".."
            || part.eq_ignore_ascii_case(".git")
            || !platform::is_plain_component(part);
        if bad {
            return Err(refused("component"));
        }
    }
    let mut path = root.to_path_buf();
    for (i, part) in parts.iter().enumerate() {
        path.push(part);
        // 구성요소마다 링크를 본다(저장소 밖을 가리키는 링크, 링크인 중간 폴더).
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(_) => return Err(CoreError::OpenMissing),
        };
        if meta.file_type().is_symlink() {
            return Err(refused("link"));
        }
        let last = i + 1 == parts.len();
        if !last && !meta.is_dir() {
            return Err(CoreError::OpenMissing);
        }
        if last && purpose == Purpose::Open {
            if !meta.is_file() {
                return Err(refused("not a file"));
            }
            if platform::is_executable(&meta) {
                return Err(refused("executable bit"));
            }
        }
    }
    if purpose == Purpose::Open {
        if blocked_extension(rel) {
            return Err(refused("extension"));
        }
        if executable_header(&path) {
            return Err(refused("header"));
        }
    }
    Ok(path)
}

/// 마지막 확장자가 차단표에 있는지(대소문자 무시).
pub(crate) fn blocked_extension(rel: &str) -> bool {
    let name = rel.rsplit(['/', '\\']).next().unwrap_or(rel);
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        let ext = ext.to_ascii_lowercase();
        OPEN_BLOCKED_EXTENSIONS.contains(&ext.as_str())
    })
}

/// ELF·PE(`MZ`)·스크립트(`#!`) 앞 바이트인지. 읽지 못하면 막는다.
fn executable_header(path: &Path) -> bool {
    let mut head = [0u8; 4];
    let Ok(file) = platform::open_regular_no_follow(path) else {
        return true;
    };
    let mut read = 0;
    let mut reader = file.take(4);
    while read < head.len() {
        match reader.read(&mut head[read..]) {
            Ok(0) => break,
            Ok(n) => read += n,
            Err(_) => return true,
        }
    }
    let head = &head[..read];
    head.starts_with(b"\x7fELF") || head.starts_with(b"MZ") || head.starts_with(b"#!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_extensions_are_lowercase_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for ext in OPEN_BLOCKED_EXTENSIONS {
            assert_eq!(*ext, ext.to_ascii_lowercase());
            assert!(seen.insert(ext), "{ext}");
        }
        assert!(blocked_extension("a/B.EXE"));
        assert!(blocked_extension("a.pdf.exe"));
        assert!(!blocked_extension("a.exe.pdf"));
        assert!(!blocked_extension("README"));
        assert!(!blocked_extension("dir.exe/readme.md"));
    }
}
