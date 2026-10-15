//! `git status --porcelain=v2 -z --branch` 파서.
//! 형식: https://git-scm.com/docs/git-status#_porcelain_format_version_2

use crate::{BranchInfo, Change, EntryKind, GitError, Result, Status, StatusEntry};

pub(crate) fn parse(text: &str) -> Result<Status> {
    let mut branch = BranchInfo::default();
    let mut entries = Vec::new();
    let mut records = text.split('\0').filter(|r| !r.is_empty());

    while let Some(record) = records.next() {
        let (tag, rest) = record.split_once(' ').ok_or_else(|| bad(record))?;
        match tag {
            "#" => parse_header(rest, &mut branch),
            "1" => {
                // XY sub mH mI mW hH hI path
                let f = fields(rest, 8, record)?;
                let (index, worktree) = parse_xy(f[0], record)?;
                entries.push(StatusEntry {
                    path: f[7].to_string(),
                    orig_path: None,
                    index,
                    worktree,
                    kind: EntryKind::Ordinary,
                });
            }
            "2" => {
                // XY sub mH mI mW hH hI Xscore path, 다음 레코드가 원래 경로
                let f = fields(rest, 9, record)?;
                let (index, worktree) = parse_xy(f[0], record)?;
                let orig = records.next().ok_or_else(|| bad(record))?;
                entries.push(StatusEntry {
                    path: f[8].to_string(),
                    orig_path: Some(orig.to_string()),
                    index,
                    worktree,
                    kind: EntryKind::Renamed,
                });
            }
            "u" => {
                // XY sub m1 m2 m3 mW h1 h2 h3 path
                let f = fields(rest, 10, record)?;
                let (index, worktree) = parse_xy(f[0], record)?;
                entries.push(StatusEntry {
                    path: f[9].to_string(),
                    orig_path: None,
                    index,
                    worktree,
                    kind: EntryKind::Unmerged,
                });
            }
            "?" => entries.push(StatusEntry {
                path: rest.to_string(),
                orig_path: None,
                index: Change::Unmodified,
                worktree: Change::Untracked,
                kind: EntryKind::Untracked,
            }),
            // "!"(무시된 파일)는 요청하지 않는다. 모르는 레코드는 앞으로의 git 확장으로 보고 건너뛴다.
            _ => {}
        }
    }
    Ok(Status { branch, entries })
}

fn parse_header(rest: &str, branch: &mut BranchInfo) {
    let Some((key, value)) = rest.split_once(' ') else {
        return;
    };
    match key {
        "branch.oid" => branch.oid = (value != "(initial)").then(|| value.to_string()),
        "branch.head" => branch.head = (value != "(detached)").then(|| value.to_string()),
        "branch.upstream" => branch.upstream = Some(value.to_string()),
        "branch.ab" => {
            for part in value.split(' ') {
                if let Some(n) = part.strip_prefix('+') {
                    branch.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix('-') {
                    branch.behind = n.parse().unwrap_or(0);
                }
            }
        }
        _ => {}
    }
}

/// 공백으로 구분된 앞 필드 `n - 1`개와, 공백을 포함할 수 있는 마지막 경로 필드를 나눈다.
fn fields<'a>(rest: &'a str, n: usize, record: &str) -> Result<Vec<&'a str>> {
    let f: Vec<&str> = rest.splitn(n, ' ').collect();
    if f.len() == n {
        Ok(f)
    } else {
        Err(bad(record))
    }
}

fn parse_xy(xy: &str, record: &str) -> Result<(Change, Change)> {
    let mut chars = xy.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some(x), Some(y), None) => Ok((change(x, record)?, change(y, record)?)),
        _ => Err(bad(record)),
    }
}

fn change(c: char, record: &str) -> Result<Change> {
    Ok(match c {
        '.' => Change::Unmodified,
        'M' => Change::Modified,
        'T' => Change::TypeChanged,
        'A' => Change::Added,
        'D' => Change::Deleted,
        'R' => Change::Renamed,
        'C' => Change::Copied,
        'U' => Change::Unmerged,
        _ => return Err(bad(record)),
    })
}

fn bad(record: &str) -> GitError {
    GitError::Parse(format!("status 레코드 형식이 다릅니다: {record:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{self, git};
    use crate::{GitBackend, RepoPath};
    use std::fs;

    const OID: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn parses_fixed_output() {
        let text = format!(
            "# branch.oid {OID}\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +2 -1\0\
             1 .M N... 100644 100644 100644 {OID} {OID} src/한글 파일.txt\0\
             2 R. N... 100644 100644 100644 {OID} {OID} R100 new name.md\0old name.md\0\
             u UU N... 100644 100644 100644 100644 {OID} {OID} {OID} conflict.txt\0\
             ? 새 파일.txt\0"
        );
        let status = parse(&text).unwrap();
        assert_eq!(
            status.branch,
            BranchInfo {
                oid: Some(OID.into()),
                head: Some("main".into()),
                upstream: Some("origin/main".into()),
                ahead: 2,
                behind: 1,
            }
        );
        assert_eq!(status.entries.len(), 4);
        assert_eq!(status.entries[0].path, "src/한글 파일.txt");
        assert_eq!(status.entries[0].worktree, Change::Modified);
        assert_eq!(status.entries[1].path, "new name.md");
        assert_eq!(status.entries[1].orig_path.as_deref(), Some("old name.md"));
        assert_eq!(status.entries[1].index, Change::Renamed);
        assert_eq!(status.entries[2].kind, EntryKind::Unmerged);
        assert_eq!(status.entries[3].worktree, Change::Untracked);
    }

    #[test]
    fn parses_initial_and_detached() {
        let status = parse("# branch.oid (initial)\0# branch.head (detached)\0").unwrap();
        assert_eq!(status.branch.oid, None);
        assert_eq!(status.branch.head, None);
        assert!(status.entries.is_empty());
    }

    #[test]
    fn rejects_broken_records() {
        assert!(parse("1 .M N...\0").is_err());
        assert!(
            parse(&format!(
                "2 R. N... 100644 100644 100644 {OID} {OID} R100 a\0"
            ))
            .is_err()
        );
        assert!(parse(&format!("1 ZZ N... 100644 100644 100644 {OID} {OID} a\0")).is_err());
    }

    fn status_of(dir: &std::path::Path) -> Status {
        testutil::backend().status(&RepoPath::new(dir)).unwrap()
    }

    fn find<'a>(status: &'a Status, path: &str) -> &'a StatusEntry {
        status
            .entries
            .iter()
            .find(|e| e.path == path)
            .unwrap_or_else(|| panic!("{path} 없음: {:?}", status.entries))
    }

    #[test]
    fn empty_repo_has_no_commit() {
        let repo = testutil::init_repo();
        let status = status_of(repo.path());
        assert_eq!(status.branch.oid, None);
        assert_eq!(status.branch.head.as_deref(), Some("main"));
        assert!(status.entries.is_empty());
    }

    #[test]
    fn real_repo_changes() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::create_dir_all(p.join("하위/폴더")).unwrap();
        for name in ["수정.txt", "삭제.txt", "stage.txt", "옮길 파일.txt"] {
            fs::write(p.join(name), "처음\n").unwrap();
        }
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "-m", "처음 저장"]);

        fs::write(p.join("수정.txt"), "바꿈\n").unwrap();
        fs::remove_file(p.join("삭제.txt")).unwrap();
        fs::write(p.join("stage.txt"), "바꿈\n").unwrap();
        git(p, &["add", "stage.txt"]);
        git(p, &["mv", "옮길 파일.txt", "옮긴 파일.txt"]);
        fs::write(p.join("한글 새 파일 (1).txt"), "새\n").unwrap();
        fs::write(p.join("하위/폴더/깊은 파일.md"), "새\n").unwrap();

        let status = status_of(p);
        assert!(status.branch.oid.is_some());

        let e = find(&status, "수정.txt");
        assert_eq!(
            (e.index, e.worktree),
            (Change::Unmodified, Change::Modified)
        );
        let e = find(&status, "삭제.txt");
        assert_eq!(e.worktree, Change::Deleted);
        let e = find(&status, "stage.txt");
        assert_eq!(
            (e.index, e.worktree),
            (Change::Modified, Change::Unmodified)
        );
        let e = find(&status, "옮긴 파일.txt");
        assert_eq!(e.kind, EntryKind::Renamed);
        assert_eq!(e.orig_path.as_deref(), Some("옮길 파일.txt"));
        let e = find(&status, "한글 새 파일 (1).txt");
        assert_eq!(e.kind, EntryKind::Untracked);
        let e = find(&status, "하위/폴더/깊은 파일.md");
        assert_eq!(e.kind, EntryKind::Untracked);
        assert_eq!(status.entries.len(), 6);
    }

    #[test]
    fn real_repo_conflict() {
        let repo = testutil::init_repo();
        let p = repo.path();
        fs::write(p.join("겹침.txt"), "처음\n").unwrap();
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "-m", "처음"]);
        git(p, &["switch", "-q", "-c", "다른갈래"]);
        fs::write(p.join("겹침.txt"), "다른 갈래\n").unwrap();
        git(p, &["commit", "-q", "-am", "다른 갈래에서 고침"]);
        git(p, &["switch", "-q", "main"]);
        fs::write(p.join("겹침.txt"), "메인\n").unwrap();
        git(p, &["commit", "-q", "-am", "메인에서 고침"]);
        // 충돌로 merge가 실패하는 것이 정상이다.
        let out = std::process::Command::new("git")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@e",
                "merge",
                "다른갈래",
            ])
            .current_dir(p)
            .output()
            .unwrap();
        assert!(!out.status.success());

        let status = status_of(p);
        let e = find(&status, "겹침.txt");
        assert_eq!(e.kind, EntryKind::Unmerged);
        assert_eq!((e.index, e.worktree), (Change::Unmerged, Change::Unmerged));
    }
}
