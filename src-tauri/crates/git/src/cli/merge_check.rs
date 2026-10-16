//! 두 버전을 합칠 때 겹치는 파일이 있는지 미리 본다. 실제 index와 작업 폴더는 건드리지 않는다(D-048, D-050).
//!
//! git 2.38 이상은 `merge-tree --write-tree`, 그 아래는 임시 index의 3-way `read-tree`와
//! 파일마다 `merge-file`로 판정한다. 아래 방식은 이름 바꿈을 알아보지 못해 겹침을 더 자주 말한다.

use std::ffi::OsStr;
use std::path::PathBuf;

use super::CliBackend;
use super::args::{Cmd, push_revs};
use super::diff;
use crate::{GitError, GitVersion, MergeCheck, RepoPath, Result};

const MERGE_TREE_SINCE: GitVersion = GitVersion::new(2, 38, 0);
/// `merge-tree --merge-base`(기준 지정)가 생긴 버전.
const MERGE_BASE_OPTION_SINCE: GitVersion = GitVersion::new(2, 40, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    MergeTree,
    ReadTree,
}

impl CliBackend {
    pub(super) fn merge_check_impl(
        &self,
        repo: &RepoPath,
        base: Option<&str>,
        ours: &str,
        theirs: &str,
    ) -> Result<MergeCheck> {
        let git = self.git_version();
        let method = match base {
            None if git >= MERGE_TREE_SINCE => Method::MergeTree,
            Some(_) if git >= MERGE_BASE_OPTION_SINCE => Method::MergeTree,
            _ => Method::ReadTree,
        };
        self.merge_check_with(method, repo, base, ours, theirs)
    }

    pub(crate) fn merge_check_with(
        &self,
        method: Method,
        repo: &RepoPath,
        base: Option<&str>,
        ours: &str,
        theirs: &str,
    ) -> Result<MergeCheck> {
        match method {
            Method::MergeTree => self.by_merge_tree(repo, base, ours, theirs),
            Method::ReadTree => self.by_read_tree(repo, base, ours, theirs),
        }
    }

    fn by_merge_tree(
        &self,
        repo: &RepoPath,
        base: Option<&str>,
        ours: &str,
        theirs: &str,
    ) -> Result<MergeCheck> {
        let base_arg = base.map(|b| format!("--merge-base={b}"));
        let mut args = vec![
            "merge-tree",
            "--write-tree",
            "--name-only",
            "-z",
            "--no-messages",
        ];
        if let Some(b) = &base_arg {
            args.push(b);
        }
        push_revs(
            &mut args,
            Cmd::MergeTree,
            self.git_version(),
            &[ours, theirs],
        );
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(_) => Ok(MergeCheck::Clean),
            // 겹침이면 종료 코드 1. 출력은 `트리 해시 NUL 파일 NUL … NUL`.
            Err(GitError::Failed {
                code: Some(1),
                stdout,
                ..
            }) => {
                let mut paths: Vec<String> = stdout
                    .split('\0')
                    .skip(1)
                    .take_while(|p| !p.is_empty())
                    .map(str::to_string)
                    .collect();
                paths.dedup();
                Ok(MergeCheck::Overlap { paths })
            }
            Err(e) => Err(e),
        }
    }

    fn by_read_tree(
        &self,
        repo: &RepoPath,
        base: Option<&str>,
        ours: &str,
        theirs: &str,
    ) -> Result<MergeCheck> {
        let dir = Some(repo.as_path());
        let base = match base {
            Some(b) => b.to_string(),
            None => match crate::GitBackend::merge_base(self, repo, ours, theirs)? {
                Some(oid) => oid.to_string(),
                // 이어지지 않은 기록이면 빈 트리를 기준으로 본다.
                None => self
                    .runner
                    .run_with_stdin(dir, diff::EMPTY_TREE_ARGS, b"")?
                    .stdout_utf8()?
                    .trim()
                    .to_string(),
            },
        };
        let scratch = Scratch::new()?;
        let index = scratch.path("index");
        let env = [("GIT_INDEX_FILE", index.as_os_str())];
        let mut args = vec!["read-tree", "-i", "-m", "--aggressive"];
        push_revs(
            &mut args,
            Cmd::ReadTree,
            self.git_version(),
            &[&base, ours, theirs],
        );
        self.runner.run_env(dir, &args, &env)?;
        let out = self.runner.run_env(dir, &["ls-files", "-u", "-z"], &env)?;
        let mut overlap = Vec::new();
        for (path, stages) in unmerged(&out.stdout_lossy())? {
            if !self.file_merges_cleanly(repo, &scratch, &stages)? {
                overlap.push(path);
            }
        }
        if overlap.is_empty() {
            Ok(MergeCheck::Clean)
        } else {
            Ok(MergeCheck::Overlap { paths: overlap })
        }
    }

    /// 세 쪽 blob이 모두 있으면 `merge-file`로 본다. 한쪽이 지웠거나 둘 다 새로 만들었으면 겹침.
    fn file_merges_cleanly(
        &self,
        repo: &RepoPath,
        scratch: &Scratch,
        stages: &[Option<String>; 3],
    ) -> Result<bool> {
        let [Some(base), Some(ours), Some(theirs)] = stages else {
            // 둘 다 같은 내용으로 새로 만든 경우는 겹치지 않는다.
            return Ok(matches!(stages, [None, Some(a), Some(b)] if a == b));
        };
        let dir = Some(repo.as_path());
        let mut files = Vec::new();
        for (name, sha) in [("ours", ours), ("base", base), ("theirs", theirs)] {
            let mut args = vec!["cat-file", "blob"];
            push_revs(&mut args, Cmd::CatFile, self.git_version(), &[sha]);
            let blob = self.runner.run(dir, &args)?;
            let path = scratch.path(name);
            // 새 파일로만 만든다(이미 있는 파일이나 링크를 따라가 덮어쓰지 않음).
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .and_then(|mut f| std::io::Write::write_all(&mut f, &blob.stdout))
                .map_err(GitError::Spawn)?;
            files.push(path);
        }
        let paths: Vec<&OsStr> = files.iter().map(|p| p.as_os_str()).collect();
        // 이진 파일이나 오류는 255, 겹친 곳 수만큼 1 이상. 0만 깨끗하다.
        self.runner.merge_file_quiet(dir, &paths)
    }
}

/// `ls-files -u -z`: `모드 해시 단계\t경로\0`. 경로별로 단계 1·2·3 blob을 모은다.
pub(super) fn unmerged(out: &str) -> Result<Vec<(String, [Option<String>; 3])>> {
    let mut found: Vec<(String, [Option<String>; 3])> = Vec::new();
    for record in out.split('\0').filter(|r| !r.is_empty()) {
        let (meta, path) = record
            .split_once('\t')
            .ok_or_else(|| GitError::Parse(format!("ls-files 줄: {record}")))?;
        let mut parts = meta.split(' ');
        let (_mode, sha, stage) = (parts.next(), parts.next(), parts.next());
        let (Some(sha), Some(Ok(stage @ 1..=3))) = (sha, stage.map(str::parse::<usize>)) else {
            return Err(GitError::Parse(format!("ls-files 줄: {record}")));
        };
        let at = match found.iter().position(|(p, _)| p == path) {
            Some(i) => i,
            None => {
                found.push((path.to_string(), [None, None, None]));
                found.len() - 1
            }
        };
        found[at].1[stage - 1] = Some(sha.to_string());
    }
    Ok(found)
}

/// 호출마다 따로 쓰는 임시 폴더. 무작위 이름으로 새로 만들고(남이 미리 만든 폴더·링크를 쓰지 않음)
/// Linux에서는 0700이다. 동기화와 백그라운드 확인이 겹쳐도 부딪히지 않는다.
pub(super) struct Scratch(tempfile::TempDir);

impl Scratch {
    pub(super) fn new() -> Result<Self> {
        tempfile::Builder::new()
            .prefix("higit-merge-")
            .tempdir()
            .map(Self)
            .map_err(GitError::Spawn)
    }

    pub(super) fn path(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unmerged_stages() {
        let out = [
            "100644 aaa 1\ta.txt",
            "100644 bbb 2\ta.txt",
            "100644 ccc 3\ta.txt",
            "100644 ddd 2\t지움.txt",
        ]
        .map(|r| format!("{r}\0"))
        .concat();
        let u = unmerged(&out).unwrap();
        assert_eq!(u.len(), 2);
        assert_eq!(
            u[0].1,
            [Some("aaa".into()), Some("bbb".into()), Some("ccc".into())]
        );
        assert_eq!(u[1].1, [None, Some("ddd".into()), None]);
        assert!(unmerged("garbage\0").is_err());
    }
}
