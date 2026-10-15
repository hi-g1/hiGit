//! `git log --no-walk --numstat -z` 파서(노선도 선 굵기, D-089).
//!
//! 버전마다 `RS 해시 NUL`이 오고, 바뀐 것이 있으면 줄바꿈 하나 뒤에 `추가\t삭제\t경로 NUL`이
//! 이어진다. 이진 파일은 `-\t-`. 빈 버전과 합치기 버전은 항목이 없다. 경로는 세지만 담지 않는다.

use crate::{CommitStat, Oid};

const RECORD: char = '\x1e';

/// 잘린 출력이면 마지막(덜 받은) 버전은 뺀다.
pub(crate) fn parse(text: &str, truncated: bool) -> Vec<CommitStat> {
    let records: Vec<&str> = text.split(RECORD).skip(1).collect();
    let keep = if truncated {
        records.len().saturating_sub(1)
    } else {
        records.len()
    };
    records[..keep]
        .iter()
        .filter_map(|record| {
            let (oid, rest) = record.split_once('\0')?;
            let oid = Oid::parse(oid)?;
            let mut stat = CommitStat {
                oid,
                files: 0,
                added: 0,
                removed: 0,
            };
            for entry in rest.strip_prefix('\n').unwrap_or(rest).split('\0') {
                let mut f = entry.splitn(3, '\t');
                let (Some(added), Some(removed), Some(_path)) = (f.next(), f.next(), f.next())
                else {
                    continue;
                };
                stat.files = stat.files.saturating_add(1);
                stat.added = stat.added.saturating_add(added.parse().unwrap_or(0));
                stat.removed = stat.removed.saturating_add(removed.parse().unwrap_or(0));
            }
            Some(stat)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "6dc4f927e8cb4978721cf6225e1ec5da10e72abd";
    const B: &str = "6ed7f913a5e957d4c4b696ecb25cd783b42c4d44";
    const C: &str = "ceada4ff15c4fa4be5e2d3a640e5fe8fd582a31e";

    #[test]
    fn parses_fixed_output() {
        // git 2.34.1에서 받은 출력: 빈 버전, 텍스트·이진·탭이 든 이름, 첫 버전.
        let text = format!(
            "\x1e{A}\0\x1e{B}\0\n2\t1\ta.txt\0-\t-\tb.bin\01\t0\t탭\t이름.txt\0\x1e{C}\0\n2\t0\ta.txt\0"
        );
        let stats = parse(&text, false);
        assert_eq!(stats.len(), 3);
        assert_eq!(
            (stats[0].files, stats[0].added, stats[0].removed),
            (0, 0, 0)
        );
        assert_eq!(
            (stats[1].files, stats[1].added, stats[1].removed),
            (3, 3, 1)
        );
        assert_eq!(stats[2].oid.as_str(), C);
        assert_eq!((stats[2].files, stats[2].added), (1, 2));
    }

    #[test]
    fn a_cut_output_drops_the_last_version() {
        let text = format!("\x1e{A}\0\n1\t1\ta\0\x1e{B}\0\n5\t0\tb");
        let stats = parse(&text, true);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].oid.as_str(), A);
        assert!(parse("", true).is_empty());
    }

    #[test]
    fn odd_records_are_skipped() {
        assert!(parse("\x1e없는해시\0\n1\t1\ta\0", false).is_empty());
    }
}
