//! 원격 주소에서 호스트·소유자·저장소 이름을 읽는다.
//!
//! 호스트는 주소의 호스트 자리만 본다. 경로에 `github.com`이 있어도 GitHub가 아니다.

/// GitHub의 웹 호스트. 호스트가 이것과 정확히 같을 때만 GitHub 저장소다(D-109).
pub const GITHUB_HOST: &str = "github.com";

const OWNER_MAX: usize = 39;
const NAME_MAX: usize = 100;
const HOST_MAX: usize = 253;
const URL_MAX: usize = 2048;

/// 원격 주소가 가리키는 저장소.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRepo {
    /// 소문자 호스트. 기본이 아닌 포트는 `host:port`로 붙는다(git credential의 `host`와 같은 모양).
    pub host: String,
    pub owner: String,
    /// `.git`을 뗀 저장소 이름.
    pub name: String,
}

impl RemoteRepo {
    /// github.com 저장소인지.
    pub fn is_github(&self) -> bool {
        self.host == GITHUB_HOST
    }
}

/// `https://host/owner/name(.git)(/)`, `ssh://[user@]host(:port)/owner/name(.git)`,
/// `[user@]host:owner/name(.git)`만 읽는다. 그 밖(http, 로컬 경로, 사용자 정보가 든 https,
/// 조각이 둘이 아닌 경로, 이상한 글자)은 `None`이다.
pub fn parse_remote(url: &str) -> Option<RemoteRepo> {
    if url.is_empty()
        || url.len() > URL_MAX
        || url.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return None;
    }
    if let Some(rest) = url.strip_prefix("https://") {
        let (authority, path) = rest.split_once('/')?;
        // 사용자 정보가 든 https 주소는 비밀이 주소에 있을 수 있어 다루지 않는다.
        if authority.contains('@') {
            return None;
        }
        let host = host_port(authority, 443)?;
        return repo_from_path(host, path);
    }
    if let Some(rest) = url.strip_prefix("ssh://") {
        let (authority, path) = rest.split_once('/')?;
        let host_part = match authority.rsplit_once('@') {
            Some((user, host)) if valid_user(user) => host,
            Some(_) => return None,
            None => authority,
        };
        let host = host_port(host_part, 22)?;
        return repo_from_path(host, path);
    }
    if url.contains("://") {
        return None;
    }
    // scp 형식. git 규칙대로 첫 `:` 앞에 `/`가 있으면 로컬 경로다.
    let (authority, path) = url.split_once(':')?;
    if authority.contains('/') || path.starts_with('/') {
        return None;
    }
    let host_part = match authority.split_once('@') {
        Some((user, host)) if valid_user(user) => host,
        Some(_) => return None,
        None => authority,
    };
    let host = valid_host(host_part)?;
    repo_from_path(host, path)
}

/// `host[:port]`를 소문자로. 기본 포트는 떼고, 다른 포트는 `host:port`로 남긴다.
fn host_port(authority: &str, default_port: u16) -> Option<String> {
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            if port.is_empty() || port.len() > 5 || !port.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let port: u16 = port.parse().ok()?;
            if port == 0 {
                return None;
            }
            (host, Some(port))
        }
        None => (authority, None),
    };
    let host = valid_host(host)?;
    Some(match port {
        Some(port) if port != default_port => format!("{host}:{port}"),
        _ => host,
    })
}

fn valid_host(host: &str) -> Option<String> {
    if host.is_empty() || host.len() > HOST_MAX {
        return None;
    }
    let host = host.to_ascii_lowercase();
    let labels_ok = host.split('.').all(|label| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    });
    labels_ok.then_some(host)
}

fn valid_user(user: &str) -> bool {
    !user.is_empty()
        && user.len() <= 64
        && user
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn repo_from_path(host: String, path: &str) -> Option<RemoteRepo> {
    let path = path.strip_suffix('/').unwrap_or(path);
    let (owner, name) = path.split_once('/')?;
    let name = name.strip_suffix(".git").unwrap_or(name);
    if !valid_owner(owner) || !valid_name(name) {
        return None;
    }
    Some(RemoteRepo {
        host,
        owner: owner.to_owned(),
        name: name.to_owned(),
    })
}

/// GitHub 사용자·조직 이름 규칙: 영숫자와 `-`, 1~39자, `-`로 시작하거나 끝나지 않음.
pub fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= OWNER_MAX
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && owner
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// GitHub 저장소 이름 규칙: 영숫자와 `.`·`_`·`-`, 1~100자, `.`·`..`은 안 됨.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= NAME_MAX
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(host: &str, owner: &str, name: &str) -> RemoteRepo {
        RemoteRepo {
            host: host.into(),
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// 받은 저장소의 원격 주소는 무엇이든 될 수 있다. 고정 씨앗으로 표본을 바꾼 글자 2만 개에 패닉하지 않고,
    /// 받아들인 것은 늘 검사를 통과한 소유자·이름이다(D-038, D-222).
    #[test]
    fn random_urls_never_panic() {
        let samples = [
            "https://github.com/octo/hello.git",
            "git@github.com:octo/hello.git",
            "ssh://git@github.com:22/octo/hello",
            "https://user:token@github.com/octo/hello/",
            "https://ghe.example.com/a/b",
        ];
        const SPECIAL: &[char] = &[
            '/', ':', '@', '.', '%', '?', '#', '\\', ' ', '\n', '\0', '한', 'é', '-', '_',
        ];
        let mut seed: u64 = 0xD1B5_4A32_D192_ED03;
        let mut next = |n: usize| {
            seed ^= seed >> 12;
            seed ^= seed << 25;
            seed ^= seed >> 27;
            usize::try_from(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) % (n.max(1) as u64))
                .unwrap_or(0)
        };
        for _ in 0..20_000 {
            let mut text: Vec<char> = samples[next(samples.len())].chars().collect();
            for _ in 0..next(6) {
                let i = next(text.len() + 1);
                match next(3) {
                    0 if i < text.len() => {
                        text.remove(i);
                    }
                    1 if i < text.len() => text[i] = SPECIAL[next(SPECIAL.len())],
                    _ => text.insert(i, SPECIAL[next(SPECIAL.len())]),
                }
            }
            let url: String = text.into_iter().collect();
            if let Some(r) = parse_remote(&url) {
                assert!(
                    valid_owner(&r.owner) && valid_name(&r.name),
                    "{url:?} → {r:?}"
                );
            }
        }
    }

    #[test]
    fn accepts_common_remote_forms() {
        let cases = [
            (
                "https://github.com/octo/hello",
                repo("github.com", "octo", "hello"),
            ),
            (
                "https://github.com/octo/hello.git",
                repo("github.com", "octo", "hello"),
            ),
            (
                "https://github.com/octo/hello/",
                repo("github.com", "octo", "hello"),
            ),
            (
                "https://github.com/octo/hello.git/",
                repo("github.com", "octo", "hello"),
            ),
            (
                "https://GitHub.COM/Octo/Hello",
                repo("github.com", "Octo", "Hello"),
            ),
            (
                "https://github.com:443/octo/hello",
                repo("github.com", "octo", "hello"),
            ),
            (
                "https://git.example.com:8443/a/b",
                repo("git.example.com:8443", "a", "b"),
            ),
            (
                "ssh://git@github.com/octo/hello.git",
                repo("github.com", "octo", "hello"),
            ),
            (
                "ssh://git@github.com:22/octo/hello.git",
                repo("github.com", "octo", "hello"),
            ),
            (
                "ssh://git@ssh.github.com:443/octo/hello.git",
                repo("ssh.github.com:443", "octo", "hello"),
            ),
            (
                "git@github.com:octo/hello.git",
                repo("github.com", "octo", "hello"),
            ),
            (
                "git@github.com:octo/my.repo_name-2",
                repo("github.com", "octo", "my.repo_name-2"),
            ),
            ("github.com:octo/hello", repo("github.com", "octo", "hello")),
            (
                "https://github.com/a-b-c/x.git.git",
                repo("github.com", "a-b-c", "x.git"),
            ),
        ];
        for (url, want) in cases {
            assert_eq!(parse_remote(url), Some(want), "{url}");
        }
    }

    #[test]
    fn rejects_odd_or_unsafe_remotes() {
        let forty = format!("https://github.com/{}/r", "a".repeat(40));
        let long_name = format!("https://github.com/o/{}", "a".repeat(101));
        let cases = [
            "",
            "http://github.com/octo/hello",
            "https://user:pw@github.com/octo/hello",
            "https://token@github.com/octo/hello",
            "git@github.com:a/b/c",
            "https://github.com/a/b/c",
            "https://github.com/octo",
            "https://github.com/",
            "https://github.com",
            "https://github.com/octo/hello world",
            "https://github.com/octo/hello\n",
            " https://github.com/octo/hello",
            "https://github.com/../hello",
            "https://github.com/octo/..",
            "https://github.com/octo/.",
            "https://github.com/octo/.git",
            "https://github.com//hello",
            "https://github.com/-a/hello",
            "https://github.com/a-/hello",
            "https://github.com/o.wner/hello",
            "https://github.com/octo/hel%6Co",
            "https://github.com/octo/hello?x=1",
            "https://github.com/octo/hello#x",
            "https://github.com:0/octo/hello",
            "https://github.com:99999/octo/hello",
            "https://github..com/octo/hello",
            "https://[::1]/octo/hello",
            "ssh://bad user@github.com/octo/hello",
            "/home/me/octo/hello",
            "C:/repos/octo/hello",
            "./octo/hello",
            "file:///srv/octo/hello.git",
            "ext::sh -c touch% /tmp/x",
            &forty,
            &long_name,
        ];
        for url in cases {
            assert_eq!(parse_remote(url), None, "{url:?}");
        }
    }

    #[test]
    fn github_is_decided_by_host_only() {
        let github = [
            "https://github.com/octo/hello",
            "https://GITHUB.com/octo/hello.git",
            "git@github.com:octo/hello.git",
            "ssh://git@github.com/octo/hello",
        ];
        for url in github {
            assert!(parse_remote(url).is_some_and(|r| r.is_github()), "{url}");
        }
        let not_github = [
            "https://evil.example/github.com/hello",
            "https://github.com.evil.example/octo/hello",
            "https://evilgithub.com/octo/hello",
            "https://www.github.com/octo/hello",
            "https://github.com:8443/octo/hello",
            "git@gitlab.com:octo/hello.git",
            "ssh://git@ssh.github.com:443/octo/hello.git",
        ];
        for url in not_github {
            assert!(!parse_remote(url).is_some_and(|r| r.is_github()), "{url}");
        }
    }

    #[test]
    fn owner_and_name_limits() {
        assert!(valid_owner(&"a".repeat(39)));
        assert!(!valid_owner(&"a".repeat(40)));
        assert!(valid_name(&"a".repeat(100)));
        assert!(!valid_name(&"a".repeat(101)));
        assert!(valid_name(".github"));
        assert!(!valid_name("한글"));
    }
}
