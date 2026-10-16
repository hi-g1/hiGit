//! `--progress` 줄에서 단계와 개수만 읽는다(D-172). 진행 줄은 기계용 형식이 아니라 D-003의 예외이고,
//! 실패 문구처럼 이 파일 한 곳에서만 읽는다. `LC_ALL=C`로 받은 영어 원문이다. 원문은 밖으로 내보내지 않는다.

/// 진행 단계.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressPhase {
    /// 보낼 버전을 세는 중(`Counting objects`, `Compressing objects`).
    Count,
    /// 받는 중(`Receiving objects`).
    Receive,
    /// 받은 것을 푸는 중(`Resolving deltas`).
    Resolve,
    /// 보내는 중(`Writing objects`).
    Send,
}

/// 마지막으로 읽은 진행.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressCount {
    pub phase: ProgressPhase,
    pub done: u32,
    pub total: u32,
}

const PHASES: [(&str, ProgressPhase); 5] = [
    ("Receiving objects:", ProgressPhase::Receive),
    ("Resolving deltas:", ProgressPhase::Resolve),
    ("Writing objects:", ProgressPhase::Send),
    ("Counting objects:", ProgressPhase::Count),
    ("Compressing objects:", ProgressPhase::Count),
];

/// 한 줄(`Receiving objects:  45% (90/200), 1.2 MiB | 3 MiB/s`)을 읽는다. 서버 쪽 줄(`remote:`)과 모르는 줄은 `None`.
pub(crate) fn parse_line(line: &str) -> Option<ProgressCount> {
    let line = line.trim();
    let (phase, rest) = PHASES
        .iter()
        .find_map(|(prefix, phase)| line.strip_prefix(prefix).map(|rest| (*phase, rest)))?;
    let open = rest.find('(')?;
    let close = open + rest[open..].find(')')?;
    let (done, total) = rest[open + 1..close].split_once('/')?;
    let done: u32 = done.trim().parse().ok()?;
    let total: u32 = total.trim().parse().ok()?;
    (total > 0 && done <= total).then_some(ProgressCount { phase, done, total })
}

/// 지금까지 받은 stderr 끝부분에서 마지막 진행을 찾는다. 줄은 `\r`·`\n`으로 나뉘고, 끝나지 않은 마지막 조각은 보지
/// 않는다(덜 온 숫자를 읽지 않게).
pub fn last_progress(tail: &[u8]) -> Option<ProgressCount> {
    let text = String::from_utf8_lossy(tail);
    let end = text.rfind(['\r', '\n'])?;
    text[..end].split(['\r', '\n']).rev().find_map(parse_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(phase: ProgressPhase, done: u32, total: u32) -> Option<ProgressCount> {
        Some(ProgressCount { phase, done, total })
    }

    #[test]
    fn reads_counts_from_real_lines() {
        let fetch = b"remote: Enumerating objects: 200, done.\nremote: Counting objects: 100% (200/200), done.\rReceiving objects:  45% (90/200), 1.20 MiB | 2.00 MiB/s\r";
        assert_eq!(last_progress(fetch), count(ProgressPhase::Receive, 90, 200));
        let resolve = b"Receiving objects: 100% (200/200), done.\nResolving deltas:  10% (3/30)\r";
        assert_eq!(last_progress(resolve), count(ProgressPhase::Resolve, 3, 30));
        let push = b"Enumerating objects: 5, done.\nCounting objects: 100% (5/5), done.\nWriting objects:  60% (3/5)\r";
        assert_eq!(last_progress(push), count(ProgressPhase::Send, 3, 5));
    }

    #[test]
    fn unfinished_or_odd_lines_are_skipped() {
        // 끝나지 않은 마지막 조각은 보지 않는다.
        assert_eq!(last_progress(b"Receiving objects:  45% (90/2"), None);
        assert_eq!(
            last_progress(b"Receiving objects:  40% (80/200)\rReceiving objects:  45% (90/2"),
            count(ProgressPhase::Receive, 80, 200)
        );
        // 서버 줄, 백분율만, 모르는 줄.
        assert_eq!(
            last_progress(b"remote: Counting objects: 50% (1/2)\n"),
            None
        );
        assert_eq!(last_progress(b"Receiving objects:  45%\r"), None);
        assert_eq!(last_progress(b"fatal: something\n"), None);
        assert_eq!(last_progress(b"Writing objects: 100% (3/0)\n"), None);
        assert_eq!(last_progress(b"Writing objects: 100% (9/3)\n"), None);
    }
}
