//! 실행 인자에서 열 폴더를 읽는다(D-213). `higit <폴더>`, 우클릭 메뉴, 이미 켜진 앱에 넘어온 두 번째 실행이 쓴다.
//!
//! 여기서는 글자만 다룬다. 폴더가 있는지, 저장소인지, 믿을지는 `Core::open_project`가 정한다.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// 경로 글자 상한(`check_path_len`과 같다).
const MAX_PATH_BYTES: usize = 4096;

/// 첫 인자는 실행 파일. `--` 뒤 첫 인자가 경로이고, 그 전에는 `-`로 시작하는 인자를 건너뛰고 첫 위치 인자를 경로로
/// 본다. git이 credential helper로 부른 실행(`--git-credential`)이면 `None`. 상대 경로는 `cwd` 기준. 끝에 붙은 `"`는
/// 뗀다(Windows에서 `"C:\폴더\"`가 `C:\폴더"`로 넘어온다). 4096바이트를 넘거나 NUL이 든 경로는 받지 않는다.
pub fn parse_args<S: AsRef<OsStr>>(args: &[S], cwd: &Path) -> Option<PathBuf> {
    let rest = args.get(1..)?;
    if rest.iter().any(|a| a.as_ref() == "--git-credential") {
        return None;
    }
    let mut after_dashes = false;
    let mut found = None;
    for arg in rest {
        let arg = arg.as_ref();
        if after_dashes {
            found = Some(arg);
            break;
        }
        if arg == "--" {
            after_dashes = true;
            continue;
        }
        if arg.to_string_lossy().starts_with('-') {
            continue;
        }
        found = Some(arg);
        break;
    }
    let text = found?.to_string_lossy().into_owned();
    let text = text.strip_suffix('"').unwrap_or(&text);
    if text.is_empty() || text.len() > MAX_PATH_BYTES || text.contains('\0') {
        return None;
    }
    let path = Path::new(text);
    Some(if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Option<PathBuf> {
        let cwd = if cfg!(windows) { r"C:\work" } else { "/work" };
        parse_args(args, Path::new(cwd))
    }

    fn abs(p: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!(
                r"C:\{}",
                p.trim_start_matches('/').replace('/', r"\")
            ))
        } else {
            PathBuf::from(p)
        }
    }

    #[test]
    fn table() {
        assert_eq!(parse(&[]), None);
        assert_eq!(parse(&["higit"]), None);
        assert_eq!(parse(&["higit", "--flag"]), None);
        assert_eq!(parse(&["higit", "proj"]), Some(abs("/work").join("proj")));
        assert_eq!(
            parse(&["higit", "-x", "proj"]),
            Some(abs("/work").join("proj"))
        );
        assert_eq!(
            parse(&["higit", "--", "-dash"]),
            Some(abs("/work").join("-dash"))
        );
        assert_eq!(parse(&["higit", "--git-credential", "get"]), None);
        assert_eq!(parse(&["higit", "proj", "--git-credential"]), None);
        assert_eq!(
            parse(&["higit", "한글 폴더/새 프로젝트"]),
            Some(abs("/work").join("한글 폴더/새 프로젝트"))
        );
        assert_eq!(parse(&["higit", ""]), None);
        let long = "a".repeat(MAX_PATH_BYTES + 1);
        assert_eq!(parse(&["higit", &long]), None);
        assert_eq!(parse(&["higit", "a\0b"]), None);
    }

    #[test]
    fn absolute_paths_and_trailing_quote() {
        if cfg!(windows) {
            assert_eq!(
                parse(&["higit", r#"C:\내 폴더""#]),
                Some(PathBuf::from(r"C:\내 폴더"))
            );
            assert_eq!(
                parse(&["higit", r#"C:""#]),
                Some(PathBuf::from(r"C:")).map(|p| Path::new(r"C:\work").join(p))
            );
        } else {
            assert_eq!(
                parse(&["higit", "/home/me/proj"]),
                Some(PathBuf::from("/home/me/proj"))
            );
            assert_eq!(
                parse(&["higit", "/home/me/proj\""]),
                Some(PathBuf::from("/home/me/proj"))
            );
        }
    }
}
