mod args;
#[cfg(test)]
mod branch_tests;
mod branches;
mod diff;
mod log;
#[cfg(test)]
mod ref_tests;
mod refs;
#[cfg(test)]
mod run_tests;
mod runner;
mod status;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use args::{Cmd, push_revs};

pub use refs::RefExpect;
pub use runner::ConfigureCommand;
use runner::Runner;

use crate::{
    Branch, Commit, DiffKind, DiffTarget, FailureKind, FileDiff, GitBackend, GitError, GitOp,
    GitVersion, LogQuery, Oid, OpOutput, RefEntry, Remote, RepoPath, Result, Status,
};

/// 시스템 git 명령을 실행하는 [`GitBackend`] 구현.
pub struct CliBackend {
    runner: Runner,
    /// `version()` 결과. `--end-of-options`를 넣을지 고를 때 쓴다(args.rs).
    version: OnceLock<GitVersion>,
}

impl CliBackend {
    /// `git`: git 실행 파일 경로. `configure`: 실행 전에 OS별 설정을 붙이는 함수(platform이 준다).
    pub fn new(git: impl Into<PathBuf>, configure: ConfigureCommand) -> Self {
        Self {
            runner: Runner::new(git.into(), configure),
            version: OnceLock::new(),
        }
    }

    /// **테스트 전용.** 로컬 경로·`file://` 원격을 허용한다. 앱 코드에서는 부르지 않는다(D-035).
    /// core 통합 테스트가 써야 해서 공개하지만 문서에는 숨긴다.
    #[doc(hidden)]
    pub fn allow_local_remotes_for_tests(mut self) -> Self {
        self.runner.allow_local_remotes();
        self
    }
}

impl CliBackend {
    /// git 버전. 처음 한 번만 읽는다. 읽지 못하면 가장 보수적인 값(표시를 넣지 않음)을 쓴다.
    fn git_version(&self) -> GitVersion {
        *self
            .version
            .get_or_init(|| self.version().unwrap_or(GitVersion::new(0, 0, 0)))
    }

    /// 저장이 하나라도 있는지. 오류 문구를 읽지 않고 종료 코드로 판단한다.
    fn has_commit(&self, repo: &RepoPath) -> Result<bool> {
        let mut args = vec!["rev-parse", "--verify", "-q"];
        push_revs(
            &mut args,
            Cmd::RevParse,
            self.git_version(),
            &["HEAD^{commit}"],
        );
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(_) => Ok(true),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(false),
            Err(e) => Err(e),
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
                if !self.has_commit(repo)? {
                    return Ok(Vec::new());
                }
                "HEAD"
            }
        };
        let skip = format!("--skip={}", query.skip);
        let max = format!("--max-count={}", query.limit);
        let mut args = vec!["log", log::FORMAT, &skip, &max];
        push_revs(&mut args, Cmd::Log, self.git_version(), &[rev]);
        args.push("--");
        let out = self.runner.run(dir, &args)?;
        log::parse(&out.stdout_lossy())
    }

    fn excludes_file(&self, repo: &RepoPath) -> Result<Option<PathBuf>> {
        // --path: `~/` 같은 경로를 git이 풀어 준다. 설정이 없으면 종료 코드 1.
        match self.runner.run(
            Some(repo.as_path()),
            &["config", "--path", "--get", "core.excludesFile"],
        ) {
            Ok(out) => {
                let path = out.stdout_utf8()?;
                let path = path.trim_end_matches(['\n', '\r']);
                Ok((!path.is_empty()).then(|| PathBuf::from(path)))
            }
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn diff(&self, repo: &RepoPath, target: &DiffTarget) -> Result<FileDiff> {
        let dir = Some(repo.as_path());
        let out = match target.kind {
            DiffKind::Tracked => {
                // 저장이 없으면 HEAD 대신 빈 트리와 비교한다(해시 방식에 맞게 git이 만든다).
                let base = if self.has_commit(repo)? {
                    "HEAD".to_string()
                } else {
                    self.runner
                        .run_with_stdin(dir, diff::EMPTY_TREE_ARGS, b"")?
                        .stdout_utf8()?
                        .trim()
                        .to_string()
                };
                let mut args = vec!["--literal-pathspecs", "diff"];
                args.extend_from_slice(diff::COMMON_ARGS);
                push_revs(&mut args, Cmd::Diff, self.git_version(), &[&base]);
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

    fn branches(&self, repo: &RepoPath) -> Result<Vec<Branch>> {
        let remotes = self.remotes(repo)?;
        let out = self.runner.run(
            Some(repo.as_path()),
            &[
                "for-each-ref",
                branches::BRANCH_FORMAT,
                "refs/heads",
                "refs/remotes",
            ],
        )?;
        branches::parse_branches(&out.stdout_lossy(), &remotes)
    }

    fn remotes(&self, repo: &RepoPath) -> Result<Vec<Remote>> {
        // 원격이 없으면 종료 코드 1.
        match self.runner.run(
            Some(repo.as_path()),
            &["config", "-z", "--get-regexp", r"^remote\..*\.url$"],
        ) {
            Ok(out) => Ok(branches::parse_remotes(&out.stdout_lossy())),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    fn list_refs(&self, repo: &RepoPath, prefix: &str) -> Result<Vec<RefEntry>> {
        if !prefix.starts_with("refs/") {
            return Err(GitError::Parse(format!("ref 접두사: {prefix}")));
        }
        let out = self.runner.run(
            Some(repo.as_path()),
            &["for-each-ref", branches::REF_FORMAT, prefix],
        )?;
        branches::parse_refs(&out.stdout_lossy())
    }

    fn remote_head(&self, repo: &RepoPath, remote: &str) -> Result<Option<String>> {
        let name = format!("refs/remotes/{remote}/HEAD");
        let mut args = vec!["symbolic-ref", "-q"];
        push_revs(&mut args, Cmd::SymbolicRef, self.git_version(), &[&name]);
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(out) => {
                let target = out.stdout_lossy();
                let prefix = format!("refs/remotes/{remote}/");
                Ok(target
                    .trim_end_matches(['\n', '\r'])
                    .strip_prefix(&prefix)
                    .map(str::to_string))
            }
            // 별칭이 없으면 종료 코드 1(-q).
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn check_branch_name(&self, name: &str) -> Result<bool> {
        if branches::obviously_bad_branch_name(name) {
            return Ok(false);
        }
        // 전체 형식으로 검사한다. `--branch`는 `@{-1}`을 이전 브랜치 이름으로 풀어 통과시킨다.
        let full = format!("refs/heads/{name}");
        match self.runner.run(None, &["check-ref-format", &full]) {
            Ok(_) => Ok(true),
            Err(GitError::Failed { code: Some(_), .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn resolve_ref(&self, repo: &RepoPath, name: &str) -> Result<Option<Oid>> {
        let target = format!("{name}^{{commit}}");
        let mut args = vec!["rev-parse", "--verify", "-q"];
        push_revs(&mut args, Cmd::RevParse, self.git_version(), &[&target]);
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(out) => {
                let text = out.stdout_lossy();
                Oid::parse(&text)
                    .map(Some)
                    .ok_or_else(|| GitError::Parse(format!("해시: {text:?}")))
            }
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn update_ref(&self, repo: &RepoPath, name: &str, new: &Oid, expect: &RefExpect) -> Result<()> {
        refs::check_ref_name(name)?;
        let old = expect.arg(new.as_str().len());
        let mut revs = vec![name, new.as_str()];
        if let Some(old) = &old {
            revs.push(old);
        }
        let mut args = vec!["update-ref", "-m", "higit"];
        push_revs(&mut args, Cmd::UpdateRef, self.git_version(), &revs);
        self.runner.run(Some(repo.as_path()), &args).map(|_| ())
    }

    fn delete_ref(&self, repo: &RepoPath, name: &str, expect: &RefExpect) -> Result<()> {
        refs::check_ref_name(name)?;
        // 지울 때 "없어야 함"은 뜻이 없으므로 Any처럼 다룬다.
        let old = match expect {
            RefExpect::Is(oid) => Some(oid.as_str().to_string()),
            _ => None,
        };
        let mut revs = vec![name];
        if let Some(old) = &old {
            revs.push(old);
        }
        let mut args = vec!["update-ref", "-m", "higit", "-d"];
        push_revs(&mut args, Cmd::UpdateRef, self.git_version(), &revs);
        self.runner.run(Some(repo.as_path()), &args).map(|_| ())
    }

    fn git_dir(&self, repo: &RepoPath) -> Result<PathBuf> {
        let out = self
            .runner
            .run(Some(repo.as_path()), &["rev-parse", "--absolute-git-dir"])?;
        let text = out.stdout_utf8()?;
        Ok(PathBuf::from(text.trim_end_matches(['\n', '\r'])))
    }

    fn is_ancestor(&self, repo: &RepoPath, ancestor: &str, descendant: &str) -> Result<bool> {
        let mut args = vec!["merge-base", "--is-ancestor"];
        push_revs(
            &mut args,
            Cmd::MergeBase,
            self.git_version(),
            &[ancestor, descendant],
        );
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(_) => Ok(true),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn count_between(&self, repo: &RepoPath, left: &str, right: &str) -> Result<(u32, u32)> {
        let range = format!("{left}...{right}");
        let mut args = vec!["rev-list", "--left-right", "--count"];
        push_revs(&mut args, Cmd::RevList, self.git_version(), &[&range]);
        let out = self.runner.run(Some(repo.as_path()), &args)?;
        refs::parse_counts(&out.stdout_lossy())
    }

    fn merge_base(&self, repo: &RepoPath, a: &str, b: &str) -> Result<Option<Oid>> {
        let mut args = vec!["merge-base"];
        push_revs(&mut args, Cmd::MergeBase, self.git_version(), &[a, b]);
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(out) => Ok(Oid::parse(&out.stdout_lossy())),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput> {
        let dir = Some(repo.as_path());
        // 경로 목록은 명령줄 대신 stdin으로 넘긴다(Windows 명령줄 길이 제한, 파일 수천 개).
        const FROM_STDIN: [&str; 2] = ["--pathspec-from-file=-", "--pathspec-file-nul"];
        let out = match op {
            GitOp::Init => self.runner.run(dir, &["init"])?,
            GitOp::Stage { paths } => self.runner.run_with_stdin(
                dir,
                &[
                    "--literal-pathspecs",
                    "add",
                    "-A",
                    FROM_STDIN[0],
                    FROM_STDIN[1],
                ],
                &runner::nul_separated(paths),
            )?,
            GitOp::Unstage { paths } => {
                let has_commit = self.has_commit(repo)?;
                // 저장이 없으면 되돌릴 HEAD가 없어 restore --staged가 실패한다. 이때는 index에서만 지운다.
                let args: &[&str] = if has_commit {
                    &[
                        "--literal-pathspecs",
                        "restore",
                        "--staged",
                        FROM_STDIN[0],
                        FROM_STDIN[1],
                    ]
                } else {
                    &[
                        "--literal-pathspecs",
                        "rm",
                        "--cached",
                        "-r",
                        "-q",
                        FROM_STDIN[0],
                        FROM_STDIN[1],
                    ]
                };
                self.runner
                    .run_with_stdin(dir, args, &runner::nul_separated(paths))?
            }
            // 메시지는 stdin으로 넘겨 한글·여러 줄·따옴표를 그대로 저장한다.
            GitOp::Commit { message } => {
                self.runner
                    .run_with_stdin(dir, &["commit", "-F", "-"], message.as_bytes())?
            }
            // `-c`의 값으로 넘기므로 `-`로 시작해도 옵션으로 읽히지 않는다(이름은 core가 이미 검사).
            GitOp::CreateAndSwitch { name } => self
                .runner
                .run(dir, &["switch", "--no-guess", "-c", name])?,
            GitOp::Switch { branch } => {
                let mut args = vec!["switch", "--no-guess"];
                push_revs(&mut args, Cmd::Switch, self.git_version(), &[branch]);
                self.runner.run(dir, &args)?
            }
            GitOp::SwitchTrack {
                local,
                remote_branch,
            } => {
                let mut args = vec!["switch", "-c", local, "--track"];
                push_revs(&mut args, Cmd::Switch, self.git_version(), &[remote_branch]);
                self.runner.run(dir, &args)?
            }
        };
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
        let names = ["한글 파일.txt", "공백 있는 이름.md", "괄호[1].txt"];
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
    fn excludes_file_from_config() {
        let repo = testutil::init_repo();
        let backend = testutil::backend();
        let path = RepoPath::new(repo.path());
        assert_eq!(backend.excludes_file(&path).unwrap(), None);
        testutil::git(
            repo.path(),
            &["config", "core.excludesFile", "/곳/전역 무시"],
        );
        assert_eq!(
            backend.excludes_file(&path).unwrap(),
            Some(PathBuf::from("/곳/전역 무시"))
        );
    }

    #[test]
    fn every_call_disables_optional_locks() {
        // 조회가 index를 쓰지 않아야 다른 작업과 index.lock이 부딪히지 않는다(감시 되먹임도 막음).
        // git 별칭으로 자식 셸의 환경 변수를 읽는다(Git for Windows도 sh를 함께 깐다).
        let out = testutil::backend()
            .runner
            .run(
                None,
                &[
                    "-c",
                    "alias.envcheck=!printf %s \"$GIT_OPTIONAL_LOCKS\"",
                    "envcheck",
                ],
            )
            .unwrap();
        assert_eq!(out.stdout_utf8().unwrap(), "0");
    }

    /// git 별칭으로 자식 셸이 받은 환경 변수 값을 읽는다.
    fn child_env(backend: &CliBackend, var: &str) -> String {
        let alias = format!("alias.envcheck=!printf %s \"${var}\"");
        backend
            .runner
            .run(None, &["-c", &alias, "envcheck"])
            .unwrap()
            .stdout_utf8()
            .unwrap()
    }

    #[test]
    fn child_env_has_no_pager_editor_or_unsafe_transport() {
        let backend = testutil::backend();
        assert_eq!(child_env(&backend, "PAGER"), "cat");
        assert_eq!(child_env(&backend, "GIT_PAGER"), "cat");
        assert_eq!(child_env(&backend, "GIT_EDITOR"), ":");
        assert_eq!(child_env(&backend, "GIT_ALLOW_PROTOCOL"), "https:ssh:git");
        let local = testutil::backend().allow_local_remotes_for_tests();
        assert_eq!(
            child_env(&local, "GIT_ALLOW_PROTOCOL"),
            "file:https:ssh:git"
        );
    }

    #[test]
    fn ext_transport_is_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let pwned = dir.path().join("pwned");
        let url = format!(
            "ext::sh -c touch% {}",
            pwned.display().to_string().replace(' ', "% ")
        );
        let err = testutil::backend()
            .runner
            .run(Some(dir.path()), &["ls-remote", &url])
            .unwrap_err();
        assert_eq!(
            err.kind(),
            FailureKind::ProtocolNotAllowed,
            "{}",
            err.detail()
        );
        assert!(!pwned.exists());
    }

    #[test]
    fn local_remotes_need_the_test_switch() {
        let remote = tempfile::tempdir().unwrap();
        testutil::git(remote.path(), &["init", "-q", "--bare"]);
        let path = remote.path().to_string_lossy().into_owned();
        let file_url = format!("file://{}", path.replace('\\', "/"));
        for url in [path.as_str(), file_url.as_str()] {
            let err = testutil::backend()
                .runner
                .run(None, &["ls-remote", url])
                .unwrap_err();
            assert_eq!(
                err.kind(),
                FailureKind::ProtocolNotAllowed,
                "{url}: {}",
                err.detail()
            );
            testutil::backend()
                .allow_local_remotes_for_tests()
                .runner
                .run(None, &["ls-remote", url])
                .unwrap();
        }
    }

    #[test]
    fn repo_config_editor_does_not_run() {
        let repo = testutil::init_repo();
        let pwned = repo.path().join("pwned");
        let cmd = format!("touch '{}'", pwned.display());
        testutil::git(repo.path(), &["config", "core.editor", &cmd]);
        std::fs::write(repo.path().join("a.txt"), "a").unwrap();
        testutil::git(repo.path(), &["add", "a.txt"]);

        // 대조군: GIT_EDITOR를 비우면 설정의 편집기가 실제로 돈다.
        let control: ConfigureCommand = std::sync::Arc::new(|cmd: &mut std::process::Command| {
            testutil::isolate(cmd);
        });
        let mut raw = std::process::Command::new("git");
        control(&mut raw);
        let _ = raw
            .args(["commit"])
            .current_dir(repo.path())
            .env_remove("GIT_EDITOR")
            .env_remove("VISUAL")
            .env_remove("EDITOR")
            .output()
            .unwrap();
        assert!(pwned.exists(), "대조군에서 core.editor가 돌지 않았다");
        std::fs::remove_file(&pwned).unwrap();

        // 앱 백엔드는 GIT_EDITOR=":"라 빈 메시지로 실패하고 설정의 명령은 돌지 않는다.
        let backend = testutil::backend();
        assert!(backend.runner.run(Some(repo.path()), &["commit"]).is_err());
        assert!(!pwned.exists());
    }

    #[test]
    fn inherited_git_dir_is_ignored() {
        let repo = testutil::init_repo();
        let elsewhere = tempfile::tempdir().unwrap();
        let missing = elsewhere.path().join("없는 곳");
        let configure: ConfigureCommand =
            std::sync::Arc::new(move |cmd: &mut std::process::Command| {
                testutil::isolate(cmd);
                cmd.env("GIT_DIR", &missing).env("GIT_WORK_TREE", &missing);
            });
        let backend = CliBackend::new("git", configure);
        let found = backend.discover(repo.path()).unwrap().unwrap();
        assert_eq!(
            testutil::canonical(found.as_path()),
            testutil::canonical(repo.path())
        );
    }

    #[test]
    fn option_like_branch_name_is_a_revision() {
        let repo = testutil::init_repo();
        std::fs::write(repo.path().join("a.txt"), "a").unwrap();
        testutil::git(repo.path(), &["add", "a.txt"]);
        testutil::git(repo.path(), &["commit", "-q", "-m", "첫 버전"]);
        // 받은 저장소에 이런 ref가 있을 수 있다. git branch는 막지만 update-ref는 만든다.
        testutil::git(repo.path(), &["update-ref", "refs/heads/-x", "HEAD"]);
        let backend = testutil::backend();
        if !args::supports_end_of_options(Cmd::Log, backend.version().unwrap()) {
            eprintln!("이 git은 --end-of-options 확인 범위 밖이라 건너뜁니다.");
            return;
        }
        let path = RepoPath::new(repo.path());
        let query = LogQuery {
            rev: Some("-x".into()),
            ..LogQuery::default()
        };
        let commits = backend.log(&path, &query).unwrap();
        assert_eq!(commits.len(), 1);

        let pwned = repo.path().join("pwned");
        let query = LogQuery {
            rev: Some(format!("--output={}", pwned.display())),
            ..LogQuery::default()
        };
        assert!(backend.log(&path, &query).is_err());
        assert!(!pwned.exists());
    }

    #[test]
    fn missing_git_is_spawn_error() {
        let backend = CliBackend::new("/없는/git", std::sync::Arc::new(|_| {}));
        assert!(matches!(backend.version(), Err(GitError::Spawn(_))));
    }
}
