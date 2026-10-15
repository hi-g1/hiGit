//! `git log -z` 파서. 필드는 NUL로 나누고, 저장 사이도 `-z`로 NUL이다.

use crate::{Commit, GitError, Result};

/// 해시, 짧은 해시, 부모들, 작성자 이름, 이메일, 작성 시각(unix), 메시지 첫 줄.
pub(crate) const FORMAT: &str = "--format=%H%x00%h%x00%P%x00%an%x00%ae%x00%at%x00%s";
const FIELDS: usize = 7;

pub(crate) fn parse(text: &str) -> Result<Vec<Commit>> {
    let text = text.trim_end_matches(['\0', '\n']);
    if text.is_empty() {
        return Ok(Vec::new());
    }
    // 부모가 없는 첫 저장은 %P가 빈 필드라서 빈 조각도 그대로 센다.
    let parts: Vec<&str> = text.split('\0').collect();
    if !parts.len().is_multiple_of(FIELDS) {
        return Err(GitError::Parse(format!(
            "log 필드 수가 {FIELDS}의 배수가 아닙니다: {}",
            parts.len()
        )));
    }
    let (chunks, _) = parts.as_chunks::<FIELDS>();
    chunks
        .iter()
        .map(|f| {
            let author_time = f[5]
                .parse()
                .map_err(|_| GitError::Parse(format!("작성 시각이 숫자가 아닙니다: {:?}", f[5])))?;
            Ok(Commit {
                // -z의 저장 구분 NUL 뒤에 줄바꿈이 붙는 git 버전도 견딘다.
                oid: f[0].trim_start_matches('\n').to_string(),
                short_oid: f[1].to_string(),
                parents: f[2].split_whitespace().map(str::to_string).collect(),
                author_name: f[3].to_string(),
                author_email: f[4].to_string(),
                author_time,
                summary: f[6].to_string(),
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

    #[test]
    fn parses_fixed_output() {
        let text = [
            "bbb",
            "bb",
            "aaa ccc",
            "이름",
            "a@b.c",
            "1700000000",
            "합침", //
            "aaa",
            "aa",
            "",
            "이름",
            "a@b.c",
            "1690000000",
            "처음 저장",
        ]
        .join("\0");
        let commits = parse(&text).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].parents, ["aaa", "ccc"]);
        assert_eq!(commits[0].summary, "합침");
        assert_eq!(commits[1].parents, Vec::<String>::new());
        assert_eq!(commits[1].author_time, 1_690_000_000);
    }

    #[test]
    fn empty_output_is_empty() {
        assert_eq!(parse("").unwrap(), vec![]);
    }

    #[test]
    fn rejects_broken_output() {
        assert!(parse("a\0b\0c").is_err());
        assert!(parse(&["a", "a", "", "n", "e", "시각", "s"].join("\0")).is_err());
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
