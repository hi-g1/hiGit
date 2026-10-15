//! 브랜치·원격·ref 목록과 브랜치 이름 검사. 출력은 NUL로 나눈 필드만 읽는다.

use crate::{Branch, BranchKind, GitError, RefEntry, Remote, Result};

/// `for-each-ref` 한 줄의 필드. 레코드는 줄로 나뉜다(ref 이름과 사람 이름에는 줄바꿈이 없다).
pub(crate) const BRANCH_FORMAT: &str = "--format=%(refname)%00%(objectname)%00%(HEAD)%00%(upstream:short)%00%(upstream:track,nobracket)%00%(authorname)%00%(committerdate:unix)";

pub(crate) const REF_FORMAT: &str =
    "--format=%(refname)%00%(objectname)%00%(creatordate:unix)%00%(contents:subject)";

/// 태그: 이름, 객체 종류, 해시, 벗긴 종류, 벗긴 해시(주석 태그만).
pub(crate) const TAG_FORMAT: &str =
    "--format=%(refname)%00%(objecttype)%00%(objectname)%00%(*objecttype)%00%(*objectname)";

/// 태그 출력에서 버전을 가리키는 것만. 가벼운 태그는 그 해시, 주석 태그는 한 단계 벗긴 해시.
pub(crate) fn parse_tags(out: &str) -> Result<Vec<crate::TagRef>> {
    let mut tags = Vec::new();
    for line in out.lines().filter(|l| !l.is_empty()) {
        let f: Vec<&str> = line.split('\0').collect();
        let [refname, kind, oid, peeled_kind, peeled] = f[..] else {
            return Err(GitError::Parse(format!("태그 줄 필드 수: {}", f.len())));
        };
        let Some(name) = refname.strip_prefix("refs/tags/") else {
            continue;
        };
        let commit = match (kind, peeled_kind) {
            ("commit", _) => oid,
            ("tag", "commit") => peeled,
            _ => continue,
        };
        let Some(commit) = crate::Oid::parse(commit) else {
            return Err(GitError::Parse(format!("태그 해시: {commit}")));
        };
        tags.push(crate::TagRef {
            name: name.to_string(),
            commit,
        });
    }
    Ok(tags)
}

/// `for-each-ref` 출력을 브랜치 목록으로. 원격 이름은 `remotes`로 가린다(이름에 `/`가 있을 수 있다).
pub(crate) fn parse_branches(out: &str, remotes: &[Remote]) -> Result<Vec<Branch>> {
    let mut branches = Vec::new();
    for line in out.lines().filter(|l| !l.is_empty()) {
        let f: Vec<&str> = line.split('\0').collect();
        let [refname, oid, head, upstream, track, author, time] = f[..] else {
            return Err(GitError::Parse(format!("브랜치 줄 필드 수: {}", f.len())));
        };
        let kind = if let Some(name) = refname.strip_prefix("refs/heads/") {
            (BranchKind::Local, name.to_string())
        } else if let Some(rest) = refname.strip_prefix("refs/remotes/") {
            // refs/remotes/<원격>/HEAD는 기본 브랜치를 가리키는 별칭이라 뺀다.
            let Some(remote) = remotes
                .iter()
                .filter(|r| rest.starts_with(&format!("{}/", r.name)))
                .max_by_key(|r| r.name.len())
            else {
                continue;
            };
            let branch = &rest[remote.name.len() + 1..];
            if branch == "HEAD" {
                continue;
            }
            (
                BranchKind::Remote {
                    remote: remote.name.clone(),
                    branch: branch.to_string(),
                },
                rest.to_string(),
            )
        } else {
            continue;
        };
        let (ahead, behind, gone) = parse_track(track);
        branches.push(Branch {
            name: kind.1,
            refname: refname.to_string(),
            kind: kind.0,
            oid: oid.to_string(),
            is_head: head == "*",
            upstream: (!upstream.is_empty()).then(|| upstream.to_string()),
            ahead,
            behind,
            gone,
            last_author: author.to_string(),
            last_time: time.parse().unwrap_or(0),
        });
    }
    Ok(branches)
}

/// `ahead 1, behind 2`, `gone`, 빈 값.
fn parse_track(track: &str) -> (u32, u32, bool) {
    if track == "gone" {
        return (0, 0, true);
    }
    let mut ahead = 0;
    let mut behind = 0;
    for part in track.split(", ") {
        if let Some(n) = part.strip_prefix("ahead ") {
            ahead = n.parse().unwrap_or(0);
        } else if let Some(n) = part.strip_prefix("behind ") {
            behind = n.parse().unwrap_or(0);
        }
    }
    (ahead, behind, false)
}

/// `refname\0objectname\0시각\0메시지 첫 줄` 줄들.
pub(crate) fn parse_refs(out: &str) -> Result<Vec<RefEntry>> {
    out.lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let mut parts = line.splitn(4, '\0');
            let (Some(name), Some(oid)) = (parts.next(), parts.next()) else {
                return Err(GitError::Parse(format!("ref 줄: {line}")));
            };
            let time = parts.next().and_then(|t| t.parse().ok()).unwrap_or(0);
            let summary = parts.next().unwrap_or("").to_string();
            Ok(RefEntry {
                name: name.to_string(),
                oid: oid.to_string(),
                time,
                summary,
            })
        })
        .collect()
}

/// `config -z --get-regexp '^remote\..*\.url$'` 출력: 레코드마다 `키\n값\0`.
pub(crate) fn parse_remotes(out: &str) -> Vec<Remote> {
    out.split('\0')
        .filter_map(|record| {
            let (key, url) = record.split_once('\n')?;
            let name = key.strip_prefix("remote.")?.strip_suffix(".url")?;
            Some(Remote {
                name: name.to_string(),
                url: url.to_string(),
            })
        })
        .collect()
}

/// git 규칙 밖에서 미리 막는 이름. `-`로 시작(옵션처럼 읽힘), `HEAD`, 제어 문자, 너무 김.
pub(crate) fn obviously_bad_branch_name(name: &str) -> bool {
    name.is_empty()
        || name.len() > 255
        || name.starts_with('-')
        || name == "HEAD"
        || name.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin() -> Vec<Remote> {
        vec![Remote {
            name: "origin".into(),
            url: "git@github.com:a/b.git".into(),
        }]
    }

    #[test]
    fn parses_local_and_remote_branches() {
        let rec = |f: &[&str]| f.join("\0") + "\n";
        let out = [
            rec(&[
                "refs/heads/main",
                "aaa",
                "*",
                "origin/main",
                "ahead 1, behind 2",
                "테스트",
                "100",
            ]),
            rec(&["refs/heads/기능/한글", "bbb", " ", "", "", "민지", "200"]),
            rec(&[
                "refs/remotes/origin/HEAD",
                "aaa",
                " ",
                "",
                "",
                "테스트",
                "100",
            ]),
            rec(&[
                "refs/remotes/origin/main",
                "ccc",
                " ",
                "",
                "",
                "테스트",
                "150",
            ]),
        ]
        .concat();
        let b = parse_branches(&out, &origin()).unwrap();
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].name, "main");
        assert!(b[0].is_head);
        assert_eq!((b[0].ahead, b[0].behind, b[0].gone), (1, 2, false));
        assert_eq!(b[0].upstream.as_deref(), Some("origin/main"));
        assert_eq!(b[1].name, "기능/한글");
        assert_eq!(b[1].last_author, "민지");
        assert_eq!(b[1].last_time, 200);
        assert_eq!(
            b[2].kind,
            BranchKind::Remote {
                remote: "origin".into(),
                branch: "main".into()
            }
        );
        assert_eq!(b[2].name, "origin/main");
    }

    #[test]
    fn parses_gone_and_rejects_short_lines() {
        assert_eq!(parse_track("gone"), (0, 0, true));
        assert_eq!(parse_track("behind 3"), (0, 3, false));
        assert!(parse_branches("refs/heads/x\0a\n", &[]).is_err());
    }

    #[test]
    fn parses_remotes_with_dots_and_slashes() {
        let out = "remote.origin.url\nhttps://github.com/a/b.git\0remote.my.fork.url\n/path/to/x\0";
        let r = parse_remotes(out);
        assert_eq!(r.len(), 2);
        assert_eq!(r[1].name, "my.fork");
        assert_eq!(r[1].url, "/path/to/x");
    }

    #[test]
    fn bad_names() {
        for name in [
            "",
            "-x",
            "--upload-pack=x",
            "HEAD",
            "a\nb",
            &"가".repeat(100),
        ] {
            assert!(obviously_bad_branch_name(name), "{name:?}");
        }
        for name in ["main", "기능/한글", "a-b"] {
            assert!(!obviously_bad_branch_name(name), "{name:?}");
        }
    }
}
