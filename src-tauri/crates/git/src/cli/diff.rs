//! unified diff 파서. git이 오래 지켜 온 기계용으로 안정된 형식만 읽는다.
//! 파일 헤더의 경로는 읽지 않는다(요청한 파일 하나만 비교하므로).

use crate::{DiffLine, FileDiff, GitError, Hunk, LineKind, Result};

/// 이보다 크면 앞부분만 담고 `truncated`로 표시한다.
pub(crate) const MAX_LINES: usize = 5_000;
pub(crate) const MAX_BYTES: usize = 1024 * 1024;

/// git 출력은 이만큼까지만 읽는다(머리줄과 `+`·`-` 기호 몫으로 줄 상한보다 넉넉히).
pub(crate) const MAX_OUTPUT: usize = MAX_BYTES * 2;
/// 이보다 큰 새 파일은 git을 부르지 않고 앞부분만 직접 읽는다.
pub(crate) const DIRECT_READ_OVER: u64 = 2 * 1024 * 1024;

/// 큰 새 파일: 앞부분 `MAX_BYTES`만 읽어 추가 줄로 만든다. 앞부분에 NUL이 있으면 이진 파일.
pub(crate) fn untracked_prefix(path: &std::path::Path) -> Result<FileDiff> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(GitError::Spawn)?;
    let mut buf = Vec::with_capacity(MAX_BYTES);
    file.by_ref()
        .take(MAX_BYTES as u64)
        .read_to_end(&mut buf)
        .map_err(GitError::Spawn)?;
    if buf.iter().take(8000).any(|b| *b == 0) {
        return Ok(FileDiff {
            binary: true,
            truncated: false,
            hunks: Vec::new(),
        });
    }
    let text = String::from_utf8_lossy(&buf);
    let lines: Vec<DiffLine> = text
        .split('\n')
        .take(MAX_LINES)
        .map(|l| DiffLine {
            kind: LineKind::Added,
            text: l.strip_suffix('\r').unwrap_or(l).to_string(),
            no_newline_at_end: false,
        })
        .collect();
    let new_len = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    Ok(FileDiff {
        binary: false,
        truncated: true,
        hunks: vec![Hunk {
            old_start: 0,
            old_len: 0,
            new_start: 1,
            new_len,
            section: String::new(),
            lines,
        }],
    })
}

/// 저장이 하나도 없는 저장소에서 비교 기준으로 쓰는 빈 트리를 만드는 입력(빈 stdin).
pub(crate) const EMPTY_TREE_ARGS: &[&str] = &["hash-object", "-t", "tree", "--stdin"];

/// 사용자 설정(diff.external, textconv, 색, 줄 수)에 흔들리지 않게 고정하는 옵션. 문맥 줄 수는 [`context_arg`].
pub(crate) const COMMON_ARGS: &[&str] = &["--no-color", "--no-ext-diff", "--no-textconv", "-M"];

/// 문맥 줄 수 인자. 보통 3줄, 접힌 줄을 펼칠 때는 파일 전체(D-156). 상수 둘에서만 고른다.
pub(crate) fn context_arg(full: bool) -> &'static str {
    if full { "-U1000000" } else { "-U3" }
}

pub(crate) fn parse(text: &str) -> Result<FileDiff> {
    let mut diff = FileDiff::default();
    let mut current: Option<Hunk> = None;
    let mut lines = 0usize;
    let mut bytes = 0usize;

    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = line.strip_prefix("@@ ") {
            if let Some(h) = current.take() {
                diff.hunks.push(h);
            }
            current = Some(parse_hunk_header(rest)?);
            continue;
        }
        let Some(hunk) = current.as_mut() else {
            // 파일 헤더 영역
            if line.starts_with("Binary files ") && line.ends_with(" differ") {
                diff.binary = true;
            }
            continue;
        };
        let (kind, body) = match line.split_at_checked(1) {
            Some((" ", body)) => (LineKind::Context, body),
            Some(("+", body)) => (LineKind::Added, body),
            Some(("-", body)) => (LineKind::Removed, body),
            Some(("\\", _)) => {
                // "\ No newline at end of file"는 바로 앞 줄에 붙는 표시다.
                if let Some(last) = hunk.lines.last_mut() {
                    last.no_newline_at_end = true;
                }
                continue;
            }
            _ => {
                // 다음 파일 헤더(diff --git …) 또는 끝의 빈 줄: hunk가 끝났다.
                if let Some(h) = current.take() {
                    diff.hunks.push(h);
                }
                continue;
            }
        };
        lines += 1;
        bytes += body.len();
        if lines > MAX_LINES || bytes > MAX_BYTES {
            diff.truncated = true;
            break;
        }
        hunk.lines.push(DiffLine {
            kind,
            text: body.to_string(),
            no_newline_at_end: false,
        });
    }
    if let Some(h) = current.take() {
        diff.hunks.push(h);
    }
    Ok(diff)
}

/// `-12,3 +12,4 @@ fn main()` → 시작·길이와 설명. 길이가 없으면 1.
fn parse_hunk_header(rest: &str) -> Result<Hunk> {
    let bad = || GitError::Parse(format!("hunk 머리 형식이 다릅니다: @@ {rest}"));
    let (ranges, section) = rest.split_once(" @@").ok_or_else(bad)?;
    let mut parts = ranges.split(' ');
    let old = parts
        .next()
        .and_then(|p| p.strip_prefix('-'))
        .ok_or_else(bad)?;
    let new = parts
        .next()
        .and_then(|p| p.strip_prefix('+'))
        .ok_or_else(bad)?;
    let range = |r: &str| -> Option<(u32, u32)> {
        match r.split_once(',') {
            Some((start, len)) => Some((start.parse().ok()?, len.parse().ok()?)),
            None => Some((r.parse().ok()?, 1)),
        }
    };
    let (old_start, old_len) = range(old).ok_or_else(bad)?;
    let (new_start, new_len) = range(new).ok_or_else(bad)?;
    Ok(Hunk {
        old_start,
        old_len,
        new_start,
        new_len,
        section: section.trim().to_string(),
        lines: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{self, git};
    use crate::{DiffKind, DiffTarget, GitBackend, RepoPath};
    use std::fs;
    use std::path::Path;

    #[test]
    fn parses_fixed_output() {
        let text = "diff --git a/a.txt b/a.txt\nindex 1..2 100644\n--- a/a.txt\n+++ b/a.txt\n\
                    @@ -1,3 +1,3 @@ 함수\n 하나\n-둘\n+두울\n 셋\n@@ -10 +10,2 @@\n-끝\n+끝\n+더함\n\\ No newline at end of file\n";
        let diff = parse(text).unwrap();
        assert!(!diff.binary && !diff.truncated);
        assert_eq!(diff.hunks.len(), 2);
        let h = &diff.hunks[0];
        assert_eq!(
            (h.old_start, h.old_len, h.new_start, h.new_len),
            (1, 3, 1, 3)
        );
        assert_eq!(h.section, "함수");
        let kinds: Vec<_> = h.lines.iter().map(|l| (l.kind, l.text.as_str())).collect();
        assert_eq!(
            kinds,
            [
                (LineKind::Context, "하나"),
                (LineKind::Removed, "둘"),
                (LineKind::Added, "두울"),
                (LineKind::Context, "셋"),
            ]
        );
        let h = &diff.hunks[1];
        assert_eq!(
            (h.old_start, h.old_len, h.new_start, h.new_len),
            (10, 1, 10, 2)
        );
        assert!(h.lines.last().unwrap().no_newline_at_end);
        assert!(!h.lines[0].no_newline_at_end);
    }

    #[test]
    fn binary_and_bad_headers() {
        let diff = parse("diff --git a/x b/x\nBinary files a/x and b/x differ\n").unwrap();
        assert!(diff.binary);
        assert!(diff.hunks.is_empty());
        assert!(parse("@@ 이상한 머리\n").is_err());
    }

    fn commit_all(p: &Path, message: &str) {
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "-m", message]);
    }

    fn diff(p: &Path, path: &str, kind: DiffKind) -> FileDiff {
        testutil::backend()
            .diff(
                &RepoPath::new(p),
                &DiffTarget {
                    path: path.into(),
                    orig_path: None,
                    kind,
                    full_context: false,
                },
            )
            .unwrap()
    }

    fn texts(d: &FileDiff, kind: LineKind) -> Vec<String> {
        d.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.kind == kind)
            .map(|l| l.text.clone())
            .collect()
    }

    #[test]
    fn modified_staged_and_unstaged_are_combined() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("한글 파일.txt"), "하나\n둘\n셋\n").unwrap();
        commit_all(p, "처음");
        fs::write(p.join("한글 파일.txt"), "하나\n두울\n셋\n").unwrap();
        git(p, &["add", "한글 파일.txt"]);
        fs::write(p.join("한글 파일.txt"), "하나\n두울\n셋\n넷\n").unwrap();

        let d = diff(p, "한글 파일.txt", DiffKind::Tracked);
        assert_eq!(texts(&d, LineKind::Removed), ["둘"]);
        assert_eq!(texts(&d, LineKind::Added), ["두울", "넷"]);
    }

    #[test]
    fn deleted_and_untracked_files() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("지울 파일.txt"), "a\nb\n").unwrap();
        commit_all(p, "처음");
        fs::remove_file(p.join("지울 파일.txt")).unwrap();
        fs::write(p.join("새 파일.txt"), "새 줄\n").unwrap();

        let d = diff(p, "지울 파일.txt", DiffKind::Tracked);
        assert_eq!(texts(&d, LineKind::Removed), ["a", "b"]);
        let d = diff(p, "새 파일.txt", DiffKind::Untracked);
        assert_eq!(texts(&d, LineKind::Added), ["새 줄"]);
        assert_eq!(d.hunks[0].old_start, 0);
    }

    #[test]
    fn renamed_file_shows_content_change() {
        let repo = testutil::init_repo();
        let p = repo.path();
        let body = "1\n2\n3\n4\n5\n6\n7\n8\n";
        fs::write(p.join("옛 이름.txt"), body).unwrap();
        commit_all(p, "처음");
        git(p, &["mv", "옛 이름.txt", "새 이름.txt"]);
        fs::write(p.join("새 이름.txt"), body.replace("8\n", "여덟\n")).unwrap();
        git(p, &["add", "새 이름.txt"]);

        let d = testutil::backend()
            .diff(
                &RepoPath::new(p),
                &DiffTarget {
                    path: "새 이름.txt".into(),
                    orig_path: Some("옛 이름.txt".into()),
                    kind: DiffKind::Tracked,
                    full_context: false,
                },
            )
            .unwrap();
        // 이름 바꿈으로 인식되면 바뀐 줄만 나온다(지운 파일 + 새 파일 전체가 아님).
        assert_eq!(texts(&d, LineKind::Removed), ["8"]);
        assert_eq!(texts(&d, LineKind::Added), ["여덟"]);
    }

    #[test]
    fn no_newline_crlf_and_binary() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("끝.txt"), "a\nb").unwrap();
        fs::write(p.join("crlf.txt"), "a\r\nb\r\n").unwrap();
        fs::write(p.join("그림.bin"), [0u8, 1, 2, 0, 255]).unwrap();
        commit_all(p, "처음");
        fs::write(p.join("끝.txt"), "a\nc").unwrap();
        fs::write(p.join("crlf.txt"), "a\r\nc\r\n").unwrap();
        fs::write(p.join("그림.bin"), [0u8, 9, 9, 0, 255]).unwrap();

        let d = diff(p, "끝.txt", DiffKind::Tracked);
        assert!(
            d.hunks[0]
                .lines
                .iter()
                .all(|l| l.kind == LineKind::Context || l.no_newline_at_end)
        );
        let d = diff(p, "crlf.txt", DiffKind::Tracked);
        assert_eq!(texts(&d, LineKind::Added), ["c"]);
        let d = diff(p, "그림.bin", DiffKind::Tracked);
        assert!(d.binary);
    }

    #[test]
    fn repo_without_commits() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("첫 파일.txt"), "처음 줄\n").unwrap();
        git(p, &["add", "첫 파일.txt"]);
        fs::write(p.join("첫 파일.txt"), "처음 줄\n둘째 줄\n").unwrap();

        let d = diff(p, "첫 파일.txt", DiffKind::Tracked);
        assert_eq!(texts(&d, LineKind::Added), ["처음 줄", "둘째 줄"]);
    }

    #[test]
    fn large_file_is_truncated() {
        let repo = testutil::init_repo();
        let p = repo.path();
        let big: String = (0..MAX_LINES + 100).map(|i| format!("줄 {i}\n")).collect();
        fs::write(p.join("큰 파일.txt"), big).unwrap();
        let d = diff(p, "큰 파일.txt", DiffKind::Untracked);
        assert!(d.truncated);
        assert_eq!(
            d.hunks.iter().map(|h| h.lines.len()).sum::<usize>(),
            MAX_LINES
        );
    }

    #[test]
    fn special_characters_are_literal() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("괄호[1].txt"), "a\n").unwrap();
        fs::write(p.join("괄호1.txt"), "x\n").unwrap();
        commit_all(p, "처음");
        fs::write(p.join("괄호[1].txt"), "b\n").unwrap();
        fs::write(p.join("괄호1.txt"), "y\n").unwrap();
        // `[1]`이 패턴으로 해석되면 괄호1.txt까지 섞인다. (`*`·`?`는 Windows 파일 이름에 쓸 수 없다)
        let d = diff(p, "괄호[1].txt", DiffKind::Tracked);
        assert_eq!(texts(&d, LineKind::Added), ["b"]);
    }
}
