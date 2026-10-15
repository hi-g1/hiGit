//! `git log` 파서. 저장마다 앞에 RS(0x1e)를 붙이고, 필드는 NUL로 나눈다.
//! 끝을 잘라내는 방식은 마지막 필드(메시지)가 비어 있을 때 필드가 사라지므로,
//! 저장 단위로 먼저 나눈 뒤 각 저장의 줄바꿈 하나만 뗀다.

use crate::{Commit, GitError, Result};

/// 해시, 짧은 해시, 부모들, 작성자 이름, 이메일, 작성 시각(unix), 메시지 첫 줄.
pub(crate) const FORMAT: &str = "--format=%x1e%H%x00%h%x00%P%x00%an%x00%ae%x00%at%x00%s";
const RECORD: char = '\x1e';
const FIELDS: usize = 7;

pub(crate) fn parse(text: &str) -> Result<Vec<Commit>> {
    text.split(RECORD)
        .skip(1) // 첫 RS 앞은 빈 문자열
        .map(|record| {
            let record = record.strip_suffix('\n').unwrap_or(record);
            let f: Vec<&str> = record.split('\0').collect();
            let [oid, short_oid, parents, name, email, time, summary] = f[..] else {
                return Err(GitError::Parse(format!(
                    "log 필드 수가 {FIELDS}개가 아닙니다: {record:?}"
                )));
            };
            let author_time = time
                .parse()
                .map_err(|_| GitError::Parse(format!("작성 시각이 숫자가 아닙니다: {time:?}")))?;
            Ok(Commit {
                oid: oid.to_string(),
                short_oid: short_oid.to_string(),
                // 첫 저장은 부모가 없어 빈 필드다.
                parents: parents.split_whitespace().map(str::to_string).collect(),
                author_name: name.to_string(),
                author_email: email.to_string(),
                author_time,
                summary: summary.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{self, git};
    use crate::{GitBackend, LogQuery, RepoPath};
    use std::fs;
    use std::path::Path;

    fn record(fields: [&str; FIELDS]) -> String {
        format!("{RECORD}{}\n", fields.join("\0"))
    }

    #[test]
    fn parses_fixed_output() {
        let text = record([
            "bbb",
            "bb",
            "aaa ccc",
            "이름",
            "a@b.c",
            "1700000000",
            "합침",
        ]) + &record(["aaa", "aa", "", "이름", "a@b.c", "1690000000", ""]);
        let commits = parse(&text).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].parents, ["aaa", "ccc"]);
        assert_eq!(commits[0].summary, "합침");
        assert_eq!(commits[1].parents, Vec::<String>::new());
        assert_eq!(commits[1].author_time, 1_690_000_000);
        assert_eq!(commits[1].summary, "");
    }

    #[test]
    fn empty_output_is_empty() {
        assert_eq!(parse("").unwrap(), vec![]);
    }

    #[test]
    fn rejects_broken_output() {
        assert!(parse(&format!("{RECORD}a\0b\0c\n")).is_err());
        assert!(parse(&record(["a", "a", "", "n", "e", "시각", "s"])).is_err());
    }

    fn commit(p: &Path, file: &str, message: &str) {
        fs::write(p.join(file), message).unwrap();
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "-m", message]);
    }

    fn log(p: &Path, query: LogQuery) -> Vec<Commit> {
        testutil::backend().log(&RepoPath::new(p), &query).unwrap()
    }

    #[test]
    fn empty_repo_has_no_commits() {
        let repo = testutil::init_repo();
        assert!(log(repo.path(), LogQuery::default()).is_empty());
    }

    #[test]
    fn newest_first_with_korean() {
        let repo = testutil::init_repo();
        let p = repo.path();
        commit(p, "a.txt", "첫 저장");
        commit(p, "b.txt", "두 번째 저장");
        commit(p, "c.txt", "세 번째: 공백과 기호 \"따옴표\"");

        let commits = log(p, LogQuery::default());
        let summaries: Vec<_> = commits.iter().map(|c| c.summary.as_str()).collect();
        assert_eq!(
            summaries,
            ["세 번째: 공백과 기호 \"따옴표\"", "두 번째 저장", "첫 저장"]
        );
        assert_eq!(commits[0].author_name, "테스트");
        assert_eq!(commits[0].author_email, "test@example.com");
        assert_eq!(commits[0].oid.len(), 40);
        assert!(commits[0].oid.starts_with(&commits[0].short_oid));
        assert_eq!(commits[0].parents, [commits[1].oid.clone()]);
        assert!(commits[2].parents.is_empty());
        assert!(commits[0].author_time > 1_600_000_000);
    }

    #[test]
    fn merge_commit_has_two_parents() {
        let repo = testutil::init_repo();
        let p = repo.path();
        commit(p, "a.txt", "처음");
        git(p, &["switch", "-q", "-c", "곁가지"]);
        commit(p, "b.txt", "곁가지 작업");
        git(p, &["switch", "-q", "main"]);
        commit(p, "c.txt", "메인 작업");
        git(p, &["merge", "-q", "--no-edit", "곁가지"]);

        let commits = log(p, LogQuery::default());
        assert_eq!(commits[0].parents.len(), 2);
        assert_eq!(commits.len(), 4);
    }

    #[test]
    fn empty_message_commits() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("a.txt"), "a").unwrap();
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "--allow-empty-message", "-m", ""]);
        commit(p, "b.txt", "가운데");
        git(
            p,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "--allow-empty-message",
                "-m",
                "",
            ],
        );

        let commits = log(p, LogQuery::default());
        let summaries: Vec<_> = commits.iter().map(|c| c.summary.as_str()).collect();
        assert_eq!(summaries, ["", "가운데", ""]);
        assert!(commits[2].parents.is_empty());
    }

    #[test]
    fn skip_and_limit() {
        let repo = testutil::init_repo();
        let p = repo.path();
        for i in 1..=5 {
            commit(p, &format!("{i}.txt"), &format!("저장 {i}"));
        }
        let page = log(
            p,
            LogQuery {
                skip: 1,
                limit: 2,
                rev: None,
            },
        );
        let summaries: Vec<_> = page.iter().map(|c| c.summary.as_str()).collect();
        assert_eq!(summaries, ["저장 4", "저장 3"]);

        let past_end = log(
            p,
            LogQuery {
                skip: 5,
                limit: 2,
                rev: None,
            },
        );
        assert!(past_end.is_empty());
    }
}
