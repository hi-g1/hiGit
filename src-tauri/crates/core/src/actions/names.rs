//! 저장소에 이미 있는 이름(브랜치, 원격, upstream) 검사. git에 넘기기 전에 막는다(D-035).
//!
//! git 버전과 명령에 따라 `--end-of-options`를 쓸 수 없는 곳이 있어(checkout, reset),
//! `-`로 시작하는 이름이 옵션으로 읽히는 길은 여기서 닫는다. 새 브랜치 이름은
//! `GitBackend::check_branch_name`이 따로 검사한다.

use crate::{CoreError, Result};

/// 받은 저장소에 `refs/heads/-x` 같은 ref가 있으면 목록에는 보이되 다룰 때 이 오류가 난다.
pub(crate) const UNSUPPORTED_NAME: &str = "이 이름의 브랜치는 hiGit에서 다룰 수 없어요";

pub(crate) fn validate_ref_like(name: &str) -> Result<()> {
    let bad = name.is_empty()
        || name.len() > 255
        || name.starts_with('-')
        || name.chars().any(char::is_control);
    if bad {
        return Err(CoreError::InvalidInput(UNSUPPORTED_NAME));
    }
    Ok(())
}

/// 복제 주소 최대 바이트.
pub(crate) const CLONE_URL_MAX: usize = 2048;
/// 복제할 폴더 이름 최대 바이트.
pub(crate) const CLONE_NAME_MAX: usize = 255;

/// 복제 주소(D-035, D-115): `https://…`, `ssh://…`, `git@host:path`만. 앞뒤 공백은 떼고
/// 검사한 주소를 돌려준다. 사용자 정보가 든 https, 비밀번호가 든 ssh, 로컬 경로, 다른 전송
/// 방식(`ext::`·`file:`·`git:`·`http:`)은 거절한다.
pub(crate) fn validate_clone_url(url: &str) -> Result<String> {
    let bad = || CoreError::CloneUrlInvalid;
    let url = url.trim();
    if url.is_empty()
        || url.len() > CLONE_URL_MAX
        || url.starts_with('-')
        || url
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return Err(bad());
    }
    let host_ok = |host: &str| {
        !host.is_empty()
            && !host.starts_with('-')
            && host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
    };
    let path_ok = |path: &str| !path.is_empty() && path != "/";
    if let Some(rest) = url.strip_prefix("https://") {
        let (authority, path) = rest.split_once('/').ok_or_else(bad)?;
        if authority.contains('@') {
            return Err(bad());
        }
        let host = match authority.rsplit_once(':') {
            Some((host, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => {
                host
            }
            Some(_) => return Err(bad()),
            None => authority,
        };
        if !host_ok(host) || !path_ok(path) {
            return Err(bad());
        }
        return Ok(url.to_owned());
    }
    if let Some(rest) = url.strip_prefix("ssh://") {
        let (authority, path) = rest.split_once('/').ok_or_else(bad)?;
        let host_port = match authority.rsplit_once('@') {
            Some((user, host)) if valid_ssh_user(user) => host,
            Some(_) => return Err(bad()),
            None => authority,
        };
        let host = match host_port.rsplit_once(':') {
            Some((host, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => {
                host
            }
            Some(_) => return Err(bad()),
            None => host_port,
        };
        if !host_ok(host) || !path_ok(path) {
            return Err(bad());
        }
        return Ok(url.to_owned());
    }
    // scp 꼴은 `git@host:path`만(D-035).
    let rest = url.strip_prefix("git@").ok_or_else(bad)?;
    let (host, path) = rest.split_once(':').ok_or_else(bad)?;
    if !host_ok(host) || !path_ok(path) || path.starts_with('/') || path.starts_with('-') {
        return Err(bad());
    }
    Ok(url.to_owned())
}

fn valid_ssh_user(user: &str) -> bool {
    !user.is_empty()
        && user
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Windows가 파일 이름으로 쓰지 못하는 장치 이름(확장자가 붙어도).
const WINDOWS_RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// 복제할 폴더 이름 하나. 두 OS 모두에서 쓸 수 있는 이름만 받는다.
pub(crate) fn validate_clone_name(name: &str) -> Result<()> {
    let bad = Err(CoreError::CloneNameInvalid);
    if name.is_empty()
        || name.len() > CLONE_NAME_MAX
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        || name.ends_with('.')
        || name.ends_with(' ')
        || name.starts_with(' ')
        || name.starts_with('-')
    {
        return bad;
    }
    let lower = name.to_lowercase();
    if matches!(lower.as_str(), "." | ".." | ".git") {
        return bad;
    }
    let stem = lower.split('.').next().unwrap_or("");
    if WINDOWS_RESERVED.contains(&stem) {
        return bad;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_urls() {
        for url in [
            "https://github.com/octo/hello.git",
            "  https://github.com/octo/hello  ",
            "https://git.example.com:8443/team/repo",
            "ssh://git@github.com/octo/hello.git",
            "ssh://git@github.com:22/octo/hello.git",
            "ssh://github.com/octo/hello.git",
            "git@github.com:octo/hello.git",
            "git@gitlab.example.com:group/sub/repo.git",
        ] {
            assert!(validate_clone_url(url).is_ok(), "{url}");
        }
        assert_eq!(
            validate_clone_url("  https://github.com/a/b ").unwrap(),
            "https://github.com/a/b"
        );
        let long = format!("https://github.com/{}", "a".repeat(2100));
        for url in [
            "",
            "-u",
            "--upload-pack=touch x",
            "http://github.com/octo/hello",
            "https://user:pw@github.com/octo/hello",
            "https://token@github.com/octo/hello",
            "https://github.com",
            "https://github.com/",
            "https://-evil.com/x",
            "https://gi thub.com/x",
            "https://github.com/a b",
            "https://github.com/a\nb",
            "https://github.com:x/a",
            "ssh://user:pw@github.com/x",
            "ssh://bad user@github.com/x",
            "ext::sh -c touch% x",
            "file:///etc",
            "git://github.com/octo/hello",
            "/home/me/repo",
            "C:\\repos\\x",
            "C:/repos/x",
            "user@github.com:octo/hello",
            "git@github.com:/abs",
            "git@github.com:-x",
            "git@:x",
            "github.com:octo/hello",
            "./repo",
            &long,
        ] {
            assert!(validate_clone_url(url).is_err(), "{url:?}");
        }
    }

    #[test]
    fn clone_names() {
        for name in ["hello", "한글 저장소", "my.repo", "a-b_c", ".github"] {
            assert!(validate_clone_name(name).is_ok(), "{name}");
        }
        let long = "a".repeat(256);
        for name in [
            "", ".", "..", ".git", ".GIT", "a/b", "a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b",
            "a>b", "a|b", "a\nb", "trail.", "trail ", " lead", "-opt", "CON", "con.txt", "Nul",
            "lpt9.md", "COM1", &long,
        ] {
            assert!(validate_clone_name(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn rejects_option_like_and_control() {
        for name in [
            "",
            "-x",
            "--upload-pack=x",
            "-c",
            "a\nb",
            "a\0b",
            &"a".repeat(256),
        ] {
            assert!(validate_ref_like(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn accepts_normal_names() {
        for name in ["main", "기능/한글", "a-b", "origin", "release/1.0"] {
            assert!(validate_ref_like(name).is_ok(), "{name:?}");
        }
    }
}
