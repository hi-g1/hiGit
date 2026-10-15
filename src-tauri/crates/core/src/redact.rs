//! "자세히 보기"에 보여줄 원문에서 자격 증명과 개인 경로를 가린다(D-040).
//!
//! 화면에 오류를 보내는 한 곳(`ErrorView` 만들기)에서만 부른다. 로그 파일은 반복 2.

/// 주소 속 아이디·토큰(`https://아이디:토큰@`, `https://ghp_…@`), GitHub 토큰, 홈 폴더 경로를 가린다.
pub(crate) fn redact(text: &str, home: Option<&str>) -> String {
    let mut out = mask_url_credentials(text);
    out = mask_tokens(&out);
    if let Some(home) = home.filter(|h| h.len() > 3) {
        out = out.replace(home, "~");
        // git은 Windows 경로를 `C:/Users/…`로도 쓴다.
        let slashed = home.replace('\\', "/");
        if slashed != home {
            out = out.replace(&slashed, "~");
        }
    }
    out
}

/// `://` 뒤에서 `/`·공백·따옴표보다 먼저 나오는 `@`까지를 `***`로.
fn mask_url_credentials(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("://") {
        let (head, tail) = rest.split_at(at + 3);
        out.push_str(head);
        let end = tail
            .find(|c: char| c == '/' || c.is_whitespace() || c == '\'' || c == '"')
            .unwrap_or(tail.len());
        match tail[..end].rfind('@') {
            Some(cred_end) if cred_end > 0 => {
                out.push_str("***");
                rest = &tail[cred_end..];
            }
            _ => rest = tail,
        }
    }
    out.push_str(rest);
    out
}

const TOKEN_PREFIXES: &[&str] = &["github_pat_", "ghp_", "gho_", "ghu_", "ghs_", "ghr_"];

fn mask_tokens(text: &str) -> String {
    let mut out = text.to_string();
    for prefix in TOKEN_PREFIXES {
        let mut from = 0;
        while let Some(pos) = out[from..].find(prefix).map(|p| p + from) {
            let body_start = pos + prefix.len();
            let body_len = out[body_start..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(out.len() - body_start);
            if body_len >= 10 {
                out.replace_range(body_start..body_start + body_len, "***");
                from = body_start + 3;
            } else {
                from = body_start;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_credentials_in_urls() {
        let cases = [
            (
                "fatal: unable to access 'https://kim:secret123@github.com/a/b.git/': 403",
                "fatal: unable to access 'https://***@github.com/a/b.git/': 403",
            ),
            (
                "remote: https://ghp_abcdefghijklmnopqrstuvwxyz0123@github.com/a/b",
                "remote: https://***@github.com/a/b",
            ),
            ("ssh://git@host.example/x", "ssh://***@host.example/x"),
            ("https://github.com/a/b.git", "https://github.com/a/b.git"),
            ("메일 a@b.com 그대로", "메일 a@b.com 그대로"),
        ];
        for (input, want) in cases {
            assert_eq!(redact(input, None), want, "{input}");
        }
    }

    #[test]
    fn masks_tokens_and_home() {
        let text = "token ghp_ABCDEFGHIJKLMNOPQRST and github_pat_11ABCDEFG0123456789_xyz in /home/minji/project";
        let got = redact(text, Some("/home/minji"));
        assert_eq!(got, "token ghp_*** and github_pat_*** in ~/project");
        let win = redact(
            r"C:\Users\민지\work and C:/Users/민지/work",
            Some(r"C:\Users\민지"),
        );
        assert_eq!(win, r"~\work and ~/work");
        // 짧은 것은 토큰이 아니다.
        assert_eq!(redact("ghp_short", None), "ghp_short");
    }
}
