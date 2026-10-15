mod diff;
mod log;
mod runner;
mod status;

use std::path::{Path, PathBuf};

pub use runner::ConfigureCommand;
use runner::Runner;

use crate::{
    Commit, DiffKind, DiffTarget, FailureKind, FileDiff, GitBackend, GitError, GitOp, GitVersion,
    LogQuery, OpOutput, RepoPath, Result, Status,
};

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
            // 그 밖의 실패(예: safe.directory 소유자 검사)는 저장소가 아닌 것으로 숨기지 않는다.
            Err(e) if e.kind() == FailureKind::NotARepository => Ok(None),
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
            .run(dir, &["log", log::FORMAT, &skip, &max, rev, "--"])?;
        log::parse(&out.stdout_lossy())
    }

    fn diff(&self, repo: &RepoPath, target: &DiffTarget) -> Result<FileDiff> {
        let dir = Some(repo.as_path());
        let out = match target.kind {
            DiffKind::Tracked => {
                // 저장이 없으면 HEAD 대신 빈 트리와 비교한다(해시 방식에 맞게 git이 만든다).
                let base = match self
                    .runner
                    .run(dir, &["rev-parse", "--verify", "-q", "HEAD^{commit}"])
                {
                    Ok(_) => "HEAD".to_string(),
                    Err(GitError::Failed { code: Some(1), .. }) => self
                        .runner
                        .run_with_stdin(dir, diff::EMPTY_TREE_ARGS, b"")?
                        .stdout_utf8()?
                        .trim()
                        .to_string(),
                    Err(e) => return Err(e),
                };
                let mut args = vec!["--literal-pathspecs", "diff"];
                args.extend_from_slice(diff::COMMON_ARGS);
                args.push(&base);
                args.push("--");
                args.push(&target.path);
                if let Some(orig) = &target.orig_path {
                    args.push(orig);
                }
                self.runner.run(dir, &args)?.stdout_lossy()
            }
            DiffKind::Untracked => {
                let mut args = vec!["diff", "--no-index"];
                args.extend_from_slice(diff::COMMON_ARGS);
                args.extend(["--", "/dev/null", &target.path]);
                // --no-index는 차이가 있으면 종료 코드 1이다(오류가 아님).
                match self.runner.run(dir, &args) {
                    Ok(out) => out.stdout_lossy(),
                    Err(GitError::Failed {
                        code: Some(1),
                        stdout,
                        ..
                    }) => stdout,
                    Err(e) => return Err(e),
                }
            }
        };
        diff::parse(&out)
    }

    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput> {
        let args: &[&str] = match op {
            GitOp::Init => &["init"],
        };
        let out = self.runner.run(Some(repo.as_path()), args)?;
        Ok(OpOutput {
            stdout: out.stdout_lossy(),
            stderr: out.stderr_lossy(),
        })
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
    fn init_makes_folder_a_repo() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("새 프로젝트");
        std::fs::create_dir(&folder).unwrap();

        let backend = testutil::backend();
        assert_eq!(backend.discover(&folder).unwrap(), None);
        backend.run(&RepoPath::new(&folder), &GitOp::Init).unwrap();

        let repo = backend
            .discover(&folder)
            .unwrap()
            .expect("init 뒤에는 저장소");
        assert_eq!(
            testutil::canonical(repo.as_path()),
            testutil::canonical(&folder)
        );
        // 새 저장소는 저장이 없다.
        assert!(backend.log(&repo, &LogQuery::default()).unwrap().is_empty());
    }

    #[test]
    fn init_in_missing_folder_fails() {
        let dir = tempfile::tempdir().unwrap();
        let missing = RepoPath::new(dir.path().join("없는 폴더"));
        assert!(testutil::backend().run(&missing, &GitOp::Init).is_err());
    }

    #[test]
    fn errors_stay_english_in_korean_locale() {
        // 사용자 환경이 한국어라고 가정한다. 앱 설정(LC_ALL=C, LANGUAGE 지움)이 이겨야 한다.
        let korean: ConfigureCommand = std::sync::Arc::new(|cmd: &mut std::process::Command| {
            cmd.env("LANG", "ko_KR.UTF-8")
                .env("LANGUAGE", "ko")
                .env("LC_MESSAGES", "ko_KR.UTF-8");
        });
        let dir = tempfile::tempdir().unwrap();

        // 대조군: 앱 설정 없이 같은 환경이면 한국어가 나오는지(번역이 설치된 환경인지) 본다.
        let control = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(dir.path())
            .env_remove("LC_ALL")
            .env("LANG", "ko_KR.UTF-8")
            .env("LANGUAGE", "ko")
            .env("LC_MESSAGES", "ko_KR.UTF-8")
            .output()
            .unwrap();
        let control = String::from_utf8_lossy(&control.stderr);
        if control.contains("not a git repository") {
            eprintln!("참고: 이 환경에는 git 한국어 번역이 없어 대조군도 영어입니다.");
        } else {
            eprintln!("대조군(한국어 확인됨): {}", control.trim());
        }

        let backend = CliBackend::new("git", korean);
        let err = backend
            .runner
            .run(Some(dir.path()), &["rev-parse", "--show-toplevel"])
            .unwrap_err();
        assert_eq!(err.kind(), FailureKind::NotARepository, "{}", err.detail());
        assert!(err.detail().contains("not a git repository"));
        assert_eq!(backend.discover(dir.path()).unwrap(), None);
    }

    #[test]
    fn paths_go_through_stdin() {
        let repo = testutil::init_repo();
        let names = ["한글 파일.txt", "공백 있는 이름.md", "별*표[1].txt"];
        for name in names {
            std::fs::write(repo.path().join(name), "내용").unwrap();
        }
        let paths: Vec<String> = names.iter().map(|n| n.to_string()).collect();
        let backend = testutil::backend();
        backend
            .runner
            .run_with_stdin(
                Some(repo.path()),
                &[
                    "--literal-pathspecs",
                    "add",
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                &runner::nul_separated(&paths),
            )
            .unwrap();

        let status = backend.status(&RepoPath::new(repo.path())).unwrap();
        let mut staged: Vec<_> = status
            .entries
            .iter()
            .filter(|e| e.index == crate::Change::Added)
            .map(|e| e.path.as_str())
            .collect();
        staged.sort();
        let mut want = names.to_vec();
        want.sort();
        assert_eq!(staged, want);
    }

    #[test]
    fn large_stdin_does_not_block() {
        // 파이프 버퍼보다 큰 입력과 출력이 동시에 오가도 멈추지 않아야 한다.
        let dir = tempfile::tempdir().unwrap();
        let input = "가".repeat(200_000);
        let out = testutil::backend()
            .runner
            .run_with_stdin(
                Some(dir.path()),
                &["hash-object", "--stdin"],
                input.as_bytes(),
            )
            .unwrap();
        assert_eq!(out.stdout_utf8().unwrap().trim().len(), 40);
    }

    #[test]
    fn missing_git_is_spawn_error() {
        let backend = CliBackend::new("/없는/git", std::sync::Arc::new(|_| {}));
        assert!(matches!(backend.version(), Err(GitError::Spawn(_))));
    }
}
