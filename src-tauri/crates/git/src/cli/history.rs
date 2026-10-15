//! 기록을 바꾸는 액션이 쓰는 조회(D-097, D-099): 올리지 않은 버전 수, 브랜치에서 닿는지, 전체 메시지.

use std::time::Duration;

use super::CliBackend;
use super::args::{Cmd, push_revs};
use crate::{Change, CommitFile, CommitFiles, CommitMessage, GitError, Oid, RepoPath, Result};

/// 원격 추적 브랜치 끝점을 이만큼까지만 본다. 넘으면 올림 여부를 "모름"으로 돌려준다.
const MAX_REMOTE_TIPS: usize = 1000;
/// 한 번에 읽는 메시지 수와 출력 상한.
pub(crate) const MAX_MESSAGES: usize = 100;
const MESSAGE_CAP: usize = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(10);

impl CliBackend {
    /// `tip`의 조상 중 어떤 원격 추적 브랜치(`refs/remotes/*`)에도 닿지 않는 버전 수. `base`가
    /// 있으면 그 조상은 세지 않는다. 원격 끝점이 너무 많으면 `None`(모름, 부르는 쪽이 거절한다).
    pub(super) fn count_unpublished_impl(
        &self,
        repo: &RepoPath,
        tip: &Oid,
        base: Option<&Oid>,
    ) -> Result<Option<u32>> {
        let dir = Some(repo.as_path());
        let out = self.runner.run(
            dir,
            &[
                "for-each-ref",
                &format!("--count={}", MAX_REMOTE_TIPS + 1),
                "--format=%(objectname)",
                "refs/remotes/",
            ],
        )?;
        let remotes: Vec<Oid> = out.stdout_lossy().lines().filter_map(Oid::parse).collect();
        if remotes.len() > MAX_REMOTE_TIPS {
            return Ok(None);
        }
        // 끝점은 명령줄 대신 stdin으로(길이 한도), 해시만 넘긴다.
        let mut input = format!("{tip}\n");
        for oid in base.into_iter().chain(remotes.iter()) {
            input.push_str(&format!("^{oid}\n"));
        }
        let out = self.runner.run_with_stdin(
            dir,
            &["rev-list", "--count", "--stdin"],
            input.as_bytes(),
        )?;
        let text = out.stdout_lossy();
        let count = text
            .trim()
            .parse()
            .map_err(|_| GitError::Parse(format!("rev-list --count: {text:?}")))?;
        Ok(Some(count))
    }

    /// 로컬 브랜치 끝에서 닿는 버전인지.
    pub(super) fn reachable_from_branches_impl(&self, repo: &RepoPath, oid: &Oid) -> Result<bool> {
        let out = self.runner.run(
            Some(repo.as_path()),
            &[
                "for-each-ref",
                "--count=1",
                &format!("--contains={oid}"),
                "--format=%(refname)",
                "refs/heads/",
            ],
        )?;
        Ok(!out.stdout_lossy().trim().is_empty())
    }

    /// 버전의 전체 메시지. 100개까지, 출력 1MB·10초 안에 읽은 것만(뒤의 버전은 결과에 없다).
    pub(super) fn commit_messages_impl(
        &self,
        repo: &RepoPath,
        oids: &[Oid],
    ) -> Result<Vec<CommitMessage>> {
        if oids.is_empty() {
            return Ok(Vec::new());
        }
        let input: String = oids
            .iter()
            .take(MAX_MESSAGES)
            .map(|o| format!("{o}\n"))
            .collect();
        let out = self.runner.run_bounded(
            Some(repo.as_path()),
            &[
                "log",
                "--no-walk=unsorted",
                "--stdin",
                "--no-show-signature",
                "--format=%x1e%H%x00%B",
            ],
            input.as_bytes(),
            MESSAGE_CAP,
            TIMEOUT,
        )?;
        Ok(parse_messages(
            &String::from_utf8_lossy(&out.stdout),
            out.truncated,
        ))
    }
}

/// 버전 목록에 담는 파일 수와 이름 목록 출력 상한.
const MAX_FILES: usize = 500;
const FILES_CAP: usize = 4 * 1024 * 1024;

impl CliBackend {
    /// 버전의 첫 부모. 첫 버전이면 `None`.
    pub(super) fn first_parent(&self, repo: &RepoPath, oid: &Oid) -> Result<Option<Oid>> {
        let rev = format!("{oid}^1");
        let mut args = vec!["rev-parse", "--verify", "-q"];
        push_revs(&mut args, Cmd::RevParse, self.git_version(), &[&rev]);
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(out) => Ok(Oid::parse(out.stdout_lossy().trim())),
            // 부모가 없으면 종료 코드 1이고 아무것도 쓰지 않는다.
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// `diff-tree`의 비교 대상: 첫 부모와 그 버전, 첫 버전이면 `--root`와 그 버전.
    pub(super) fn commit_diff_revs(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>> {
        Ok(match self.first_parent(repo, oid)? {
            Some(parent) => vec![parent.to_string(), oid.to_string()],
            None => vec![oid.to_string()],
        })
    }

    pub(super) fn commit_files_impl(&self, repo: &RepoPath, oid: &Oid) -> Result<CommitFiles> {
        let revs = self.commit_diff_revs(repo, oid)?;
        let mut args = vec![
            "diff-tree",
            "-r",
            "-z",
            "--name-status",
            "-M",
            "--no-commit-id",
        ];
        if revs.len() == 1 {
            args.push("--root");
        }
        let rev_refs: Vec<&str> = revs.iter().map(String::as_str).collect();
        push_revs(&mut args, Cmd::DiffTree, self.git_version(), &rev_refs);
        args.push("--");
        let out = self
            .runner
            .run_capped(Some(repo.as_path()), &args, FILES_CAP)?;
        Ok(parse_name_status(&String::from_utf8_lossy(&out.stdout)))
    }
}

/// `--name-status -z`: `상태 NUL 경로 NUL`, 이름 바꿈·복사는 `R100 NUL 옛 경로 NUL 새 경로 NUL`.
pub(crate) fn parse_name_status(text: &str) -> CommitFiles {
    let mut tokens = text.split('\0').filter(|t| !t.is_empty());
    let mut out = CommitFiles::default();
    while let Some(status) = tokens.next() {
        let change = match status.as_bytes().first() {
            Some(b'A') => Change::Added,
            Some(b'D') => Change::Deleted,
            Some(b'T') => Change::TypeChanged,
            Some(b'R') => Change::Renamed,
            Some(b'C') => Change::Copied,
            _ => Change::Modified,
        };
        let (path, orig_path) = if matches!(change, Change::Renamed | Change::Copied) {
            let (Some(old), Some(new)) = (tokens.next(), tokens.next()) else {
                break;
            };
            (new, Some(old.to_string()))
        } else {
            let Some(path) = tokens.next() else {
                break;
            };
            (path, None)
        };
        out.total = out.total.saturating_add(1);
        if out.files.len() < MAX_FILES {
            out.files.push(CommitFile {
                path: path.to_string(),
                orig_path,
                change,
            });
        }
    }
    out
}

/// `RS 해시 NUL 메시지` 레코드들. 잘린 출력이면 마지막(덜 받은) 레코드는 뺀다. 끝의 줄바꿈은
/// 뜻이 없어(git이 메시지 끝과 레코드 끝에 붙인다) 뗀다.
pub(crate) fn parse_messages(text: &str, truncated: bool) -> Vec<CommitMessage> {
    let records: Vec<&str> = text.split('\x1e').skip(1).collect();
    let keep = if truncated {
        records.len().saturating_sub(1)
    } else {
        records.len()
    };
    records[..keep]
        .iter()
        .filter_map(|r| {
            let (oid, message) = r.split_once('\0')?;
            let message = message.trim_end_matches('\n');
            Some(CommitMessage {
                oid: Oid::parse(oid)?,
                message: message.to_string(),
            })
        })
        .collect()
}
