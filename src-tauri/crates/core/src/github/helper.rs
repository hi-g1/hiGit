//! hiGit credential helper(D-111). git이 앱 실행 파일을 `--git-credential get`으로 다시 실행하면
//! 여기서 답한다.
//!
//! `protocol=https`와 `host=github.com`이 정확히 맞고 사용자 이름이 없을 때만 키체인을 읽어 토큰을
//! 준다. 그 밖에는 아무것도 쓰지 않고 0으로 끝난다(git은 다음 방법으로 넘어간다). 토큰은 표준 출력
//! 파이프로만 나가고 로그·오류·명령줄·환경 변수에 없다.

use std::io::{Read, Write};
use std::time::Duration;

use hosting::{ExposeSecret, SecretString};

use super::{KEYCHAIN_ACCOUNT, KEYCHAIN_SERVICE};

/// 입력 상한.
const INPUT_MAX_BYTES: usize = 8 * 1024;
const INPUT_MAX_LINES: usize = 64;
/// helper 전체 시간 제한(키체인 잠금 해제 창을 오래 기다리지 않는다).
const TIMEOUT: Duration = Duration::from_secs(10);

/// `--git-credential`로 실행된 앱의 진입점. 종료 코드는 늘 0이다.
pub fn git_credential_main() -> i32 {
    let args: Vec<String> = std::env::args_os()
        .skip(2)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run(&args, stdin.lock(), stdout.lock(), || {
        platform::Keychain::new(KEYCHAIN_SERVICE)
            .get(KEYCHAIN_ACCOUNT)
            .ok()
            .flatten()
    })
}

/// 시험할 수 있는 본체. `token`은 호스트가 맞을 때만 부른다.
pub(crate) fn run(
    args: &[String],
    input: impl Read,
    mut output: impl Write,
    token: impl FnOnce() -> Option<SecretString> + Send + 'static,
) -> i32 {
    if args.first().map(String::as_str) != Some("get") || args.len() != 1 {
        return 0;
    }
    let Some(text) = read_input(input) else {
        return 0;
    };
    if !wants_github(&text) {
        return 0;
    }
    // 키체인이 잠겨 창을 띄우고 오래 기다리면 답하지 않고 끝낸다.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(token());
    });
    let Ok(Some(token)) = rx.recv_timeout(TIMEOUT) else {
        return 0;
    };
    let answer = format!(
        "username=x-access-token\npassword={}\n\n",
        token.expose_secret()
    );
    // 쓰지 못해도(git이 먼저 끝남) 할 일은 없다.
    let _ = output.write_all(answer.as_bytes());
    let _ = output.flush();
    0
}

/// 빈 줄이나 끝까지, 상한 안에서 읽는다. 상한을 넘거나 UTF-8이 아니거나 NUL이 있으면 `None`.
fn read_input(input: impl Read) -> Option<String> {
    let mut bytes = Vec::new();
    input
        .take(INPUT_MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > INPUT_MAX_BYTES || bytes.contains(&0) {
        return None;
    }
    let text = String::from_utf8(bytes).ok()?;
    let body = match text.find("\n\n") {
        Some(end) => &text[..end],
        None => text.as_str(),
    };
    (body.lines().count() <= INPUT_MAX_LINES).then(|| body.to_owned())
}

/// `protocol=https`, `host=github.com`(대소문자 무시, 포트 없음), 사용자 이름 없음.
/// `protocol`·`host`가 두 번 오거나 `=` 없는 줄이 있으면 답하지 않는다. 모르는 키는 무시한다.
fn wants_github(text: &str) -> bool {
    let mut protocol = None;
    let mut host = None;
    for line in text.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            break;
        }
        let Some((key, value)) = line.split_once('=') else {
            return false;
        };
        match key {
            "protocol" => {
                if protocol.replace(value).is_some() {
                    return false;
                }
            }
            "host" => {
                if host.replace(value).is_some() {
                    return false;
                }
            }
            "username" => return false,
            // `wwwauth[]`·`capability[]` 같은 배열 키, `path` 등은 무시한다.
            _ => {}
        }
    }
    protocol.is_some_and(|p| p.eq_ignore_ascii_case("https"))
        && host.is_some_and(|h| h.eq_ignore_ascii_case(hosting::GITHUB_HOST))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    const TOKEN: &str = "gho_TESTSECRET0123456789";

    /// (종료 코드, 출력, 키체인을 읽었는지)
    fn answer(args: &[&str], input: &str) -> (i32, String, bool) {
        let asked = Arc::new(AtomicBool::new(false));
        let flag = asked.clone();
        let mut out = Vec::new();
        let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        let code = run(&args, input.as_bytes(), &mut out, move || {
            flag.store(true, Ordering::SeqCst);
            Some(SecretString::from(TOKEN.to_owned()))
        });
        (
            code,
            String::from_utf8(out).unwrap(),
            asked.load(Ordering::SeqCst),
        )
    }

    #[test]
    fn answers_github_https_only() {
        let (code, out, asked) = answer(&["get"], "protocol=https\nhost=github.com\n\n");
        assert_eq!(code, 0);
        assert!(asked);
        assert_eq!(
            out,
            format!("username=x-access-token\npassword={TOKEN}\n\n")
        );

        let (_, out, _) = answer(&["get"], "protocol=HTTPS\nhost=GitHub.com\n");
        assert!(out.contains(TOKEN));
        let (_, out, _) = answer(
            &["get"],
            "capability[]=authtype\nprotocol=https\nhost=github.com\nwwwauth[]=Basic realm=\"GitHub\"\npath=octo/hello.git\n\n",
        );
        assert!(out.contains(TOKEN));
    }

    #[test]
    fn stays_silent_for_anything_else() {
        let long = format!("protocol=https\nhost=github.com\nx={}\n", "a".repeat(9000));
        let many = format!("protocol=https\nhost=github.com\n{}", "k=v\n".repeat(70));
        let cases: &[(&[&str], &str)] = &[
            (&["get"], "protocol=https\nhost=github.com:443\n"),
            (&["get"], "protocol=https\nhost=evil.github.com\n"),
            (&["get"], "protocol=https\nhost=github.com.evil.example\n"),
            (&["get"], "protocol=http\nhost=github.com\n"),
            (&["get"], "protocol=ssh\nhost=github.com\n"),
            (&["get"], ""),
            (&["get"], "host=github.com\n"),
            (
                &["get"],
                "protocol=https\nhost=github.com\nhost=evil.example\n",
            ),
            (
                &["get"],
                "protocol=https\nprotocol=https\nhost=github.com\n",
            ),
            (&["get"], "protocol=https\nhost=github.com\nusername=mona\n"),
            (&["get"], "protocol=https\nhost=github.com\ngarbage\n"),
            (&["get"], "protocol=https\nhost=git\0hub.com\n"),
            (&["get"], &long),
            (&["get"], &many),
            (&["store"], "protocol=https\nhost=github.com\n"),
            (&["erase"], "protocol=https\nhost=github.com\n"),
            (&["unknown"], "protocol=https\nhost=github.com\n"),
            (&[], "protocol=https\nhost=github.com\n"),
            (&["get", "extra"], "protocol=https\nhost=github.com\n"),
        ];
        for (args, input) in cases {
            let (code, out, asked) = answer(args, input);
            assert_eq!(code, 0, "{args:?} {input:?}");
            assert_eq!(out, "", "{args:?} {input:?}");
            assert!(!asked, "키체인을 읽지 않아야 한다: {args:?} {input:?}");
        }
    }

    #[test]
    fn host_after_blank_line_is_ignored() {
        let (_, out, asked) = answer(&["get"], "protocol=https\n\nhost=github.com\n");
        assert_eq!(out, "");
        assert!(!asked);
    }

    #[test]
    fn no_token_means_no_answer() {
        let mut out = Vec::new();
        let code = run(
            &["get".to_owned()],
            "protocol=https\nhost=github.com\n".as_bytes(),
            &mut out,
            || None,
        );
        assert_eq!(code, 0);
        assert!(out.is_empty());
    }
}
