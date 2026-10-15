mod log;
mod runner;
mod status;

use std::path::{Path, PathBuf};

pub use runner::ConfigureCommand;
use runner::Runner;

use crate::{Commit, GitBackend, GitError, GitVersion, LogQuery, RepoPath, Result, Status};

/// 시스템 git 명령을 실행하는 [`GitBackend`] 구현.
pub struct CliBackend {
    runner: Runner,
}

impl CliBackend {
    /// `git`: git 실행 파일 경로. `configure`: 실행 전에 OS별 설정을 붙이는 함수(platform이 준다).
    pub fn new(git: impl Into<PathBuf>, configure: ConfigureCommand) -> Self {
        Self {
            runner: Runner::new(git.into(), configure),
        }
    }
}

impl GitBackend for CliBackend {
    fn version(&self) -> Result<GitVersion> {
        let out = self.runner.run(None, &["--version"])?;
        let text = out.stdout_utf8()?;
        GitVersion::parse(&text).ok_or_else(|| GitError::Parse(text.trim().to_string()))
    }

    fn discover(&self, path: &Path) -> Result<Option<RepoPath>> {
        match self
            .runner
            .run(Some(path), &["rev-parse", "--show-toplevel"])
        {
            Ok(out) => {
                let top = out.stdout_utf8()?;
                Ok(Some(RepoPath::new(top.trim_end_matches(['\n', '\r']))))
            }
            // LC_ALL=C로 실행하므로 영어 원문으로 판별한다. 그 밖의 실패
            // (예: safe.directory 소유자 검사)는 저장소가 아닌 것으로 숨기지 않는다.
            Err(GitError::Failed { ref stderr, .. }) if stderr.contains("not a git repository") => {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    fn status(&self, repo: &RepoPath) -> Result<Status> {
        let out = self.runner.run(
            Some(repo.as_path()),
            &[
                "status",
                "--porcelain=v2",
                "-z",
                "--branch",
                "--untracked-files=all",
            ],
        )?;
        status::parse(&out.stdout_lossy())
    }

    fn log(&self, repo: &RepoPath, query: &LogQuery) -> Result<Vec<Commit>> {
        let dir = Some(repo.as_path());
        let rev = match &query.rev {
            Some(rev) => rev.as_str(),
            None => {
                // 저장이 없는 저장소는 오류 문구를 읽지 않고 HEAD 존재 여부로 판단한다.
                // -q: 없으면 출력 없이 종료 코드 1.
                match self
                    .runner
                    .run(dir, &["rev-parse", "--verify", "-q", "HEAD^{commit}"])
                {
                    Ok(_) => "HEAD",
                    Err(GitError::Failed { code: Some(1), .. }) => return Ok(Vec::new()),
                    Err(e) => return Err(e),
                }
            }
        };
        let skip = format!("--skip={}", query.skip);
        let max = format!("--max-count={}", query.limit);
        let out = self
            .runner
            .run(dir, &["log", "-z", log::FORMAT, &skip, &max, rev, "--"])?;
        log::parse(&out.stdout_lossy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;

    #[test]
    fn version_is_supported() {
        let v = testutil::backend().version().unwrap();
        assert!(v >= GitVersion::new(2, 30, 0), "{v}");
    }

    #[test]
    fn discover_finds_top_from_subfolder() {
        let repo = testutil::init_repo();
        let sub = repo.path().join("한글 폴더").join("안쪽");
        std::fs::create_dir_all(&sub).unwrap();

        let backend = testutil::backend();
        let from_top = backend.discover(repo.path()).unwrap().unwrap();
        let from_sub = backend.discover(&sub).unwrap().unwrap();
        assert_eq!(from_top, from_sub);
        assert_eq!(
            testutil::canonical(from_top.as_path()),
            testutil::canonical(repo.path())
        );
    }

    #[test]
    fn discover_returns_none_outside_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(testutil::backend().discover(dir.path()).unwrap(), None);
    }

    #[test]
    fn missing_git_is_spawn_error() {
        let backend = CliBackend::new("/없는/git", std::sync::Arc::new(|_| {}));
        assert!(matches!(backend.version(), Err(GitError::Spawn(_))));
    }
}
