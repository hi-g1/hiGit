mod args;
#[cfg(test)]
mod blob_tests;
mod blobs;
#[cfg(test)]
mod branch_tests;
mod branches;
mod diff;
#[cfg(test)]
mod diff_tests;
#[cfg(test)]
mod graph_tests;
mod history;
#[cfg(test)]
mod history_tests;
mod log;
mod merge_check;
#[cfg(test)]
mod merge_check_tests;
mod network;
#[cfg(test)]
mod network_tests;
mod park;
pub use park::safe_relative;
#[cfg(test)]
mod park_tests;
#[cfg(test)]
mod ref_tests;
mod refs;
#[cfg(test)]
mod remote_tests;
mod resolve;
#[cfg(test)]
mod resolve_tests;
#[cfg(test)]
mod run_tests;
mod runner;
mod segments;
mod stats;
mod status;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use args::{Cmd, push_revs};

pub use network::{BACKGROUND_TIMEOUT, NETWORK_TIMEOUT, NetMode};
pub use refs::RefExpect;
use runner::Runner;
pub use runner::{ConfigureCommand, ProcessControl};

use crate::{
    BlobHead, Branch, Commit, CommitFiles, CommitMessage, CommitStat, DiffKind, DiffTarget,
    FailureKind, FileDiff, GitBackend, GitError, GitOp, GitVersion, GraphLogQuery, IndexEntry,
    LogQuery, MergeCheck, MergeSegment, Oid, OpOutput, RefEntry, ReflogEntry, Remote, RepoPath,
    Result, StashEntry, Status, TagRef,
};

/// 시스템 git 명령을 실행하는 [`GitBackend`] 구현.
pub struct CliBackend {
    runner: Runner,
    /// 원격 작업 시간 제한(사용자가 누름, 백그라운드).
    timeouts: (std::time::Duration, std::time::Duration),
    /// `version()` 결과. `--end-of-options`를 넣을지 고를 때 쓴다(args.rs).
    version: OnceLock<GitVersion>,
    /// 저장소별 취소 깃발(취소 단추·강제로 멈추기, D-080).
    cancels: std::sync::Mutex<
        std::collections::HashMap<PathBuf, std::sync::Arc<std::sync::atomic::AtomicBool>>,
    >,
    /// 오래 걸리는 작업의 진행 알림(저장소, 경과, 마지막 출력 뒤 시간).
    observer: std::sync::Mutex<Option<crate::ProgressObserver>>,
}

impl CliBackend {
    /// `git`: git 실행 파일 경로. `configure`: 실행 전에 OS별 설정을 붙이는 함수(platform이 준다).
    pub fn new(git: impl Into<PathBuf>, configure: ConfigureCommand) -> Self {
        Self {
            runner: Runner::new(git.into(), configure),
            timeouts: (network::NETWORK_TIMEOUT, network::BACKGROUND_TIMEOUT),
            version: OnceLock::new(),
            cancels: std::sync::Mutex::default(),
            observer: std::sync::Mutex::default(),
        }
    }

    /// 이 저장소의 취소 깃발.
    pub(crate) fn cancel_flag(
        &self,
        repo: &RepoPath,
    ) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.cancels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(repo.as_path().to_path_buf())
            .or_default()
            .clone()
    }

    /// 지켜보며 실행할 때 쓰는 진행 알림(이 저장소용으로 묶음).
    pub(crate) fn observer_for(
        &self,
        repo: &RepoPath,
    ) -> Option<Box<dyn Fn(std::time::Duration, std::time::Duration) + Send + Sync>> {
        let observer = self
            .observer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()?;
        let path = repo.as_path().to_path_buf();
        Some(Box::new(move |elapsed, idle| {
            observer(&path, elapsed, idle)
        }))
    }

    /// 사용자 hook이나 서명이 오래 붙잡을 수 있는 로컬 쓰기(저장·합치기·되돌리기). 시간 제한 없이
    /// 돌리되 사람이 확인하면 강제로 멈출 수 있게 한다(D-066, D-080).
    fn run_local_watched(
        &self,
        repo: &RepoPath,
        args: &[&str],
        stdin: Option<&[u8]>,
    ) -> Result<runner::GitOutput> {
        let cancel = self.cancel_flag(repo);
        let observer = self.observer_for(repo);
        self.runner.run_watched(
            Some(repo.as_path()),
            args,
            &[],
            stdin,
            &runner::Watch {
                cancel: Some(&cancel),
                observer: observer.as_deref(),
                ..runner::Watch::default()
            },
        )
    }

    /// 시간 제한으로 git 프로세스 트리를 끝내는 방법(platform이 준다).
    pub fn with_process_control(mut self, control: ProcessControl) -> Self {
        self.runner.process = control;
        self
    }

    /// **테스트 전용.** 원격 작업 시간 제한을 바꾼다.
    #[doc(hidden)]
    pub fn with_network_timeouts_for_tests(
        mut self,
        foreground: std::time::Duration,
        background: std::time::Duration,
    ) -> Self {
        self.timeouts = (foreground, background);
        self
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

    fn log_graph(&self, repo: &RepoPath, query: &GraphLogQuery) -> Result<Vec<Commit>> {
        if query.tips.is_empty() {
            return Ok(Vec::new());
        }
        let skip = format!("--skip={}", query.skip);
        let max = format!("--max-count={}", query.limit);
        let excludes: Vec<String> = query.exclude.iter().map(|o| format!("^{o}")).collect();
        let mut revs: Vec<&str> = query.tips.iter().map(Oid::as_str).collect();
        revs.extend(excludes.iter().map(String::as_str));
        // 개인 설정 log.showSignature가 형식 출력에 서명 글자를 섞지 않게 끈다.
        let mut args = vec![
            "log",
            "--topo-order",
            "--no-show-signature",
            log::FORMAT,
            &skip,
            &max,
        ];
        push_revs(&mut args, Cmd::Log, self.git_version(), &revs);
        args.push("--");
        let out = self.runner.run(Some(repo.as_path()), &args)?;
        log::parse(&out.stdout_lossy())
    }

    fn commit_stats(&self, repo: &RepoPath, oids: &[Oid]) -> Result<Vec<CommitStat>> {
        const CAP: usize = 2 * 1024 * 1024;
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
        if oids.is_empty() {
            return Ok(Vec::new());
        }
        // 해시는 stdin으로(명령줄에 닿지 않음). 합치기 버전은 diff를 내지 않아 0이다.
        let input: String = oids.iter().map(|o| format!("{o}\n")).collect();
        let out = self.runner.run_bounded(
            Some(repo.as_path()),
            &[
                "log",
                "--no-walk=unsorted",
                "--stdin",
                "--root",
                "--numstat",
                "-z",
                "--no-renames",
                "--no-show-signature",
                "--no-ext-diff",
                "--format=%x1e%H",
            ],
            input.as_bytes(),
            CAP,
            TIMEOUT,
        )?;
        Ok(stats::parse(
            &String::from_utf8_lossy(&out.stdout),
            out.truncated,
        ))
    }

    fn count_unpublished(
        &self,
        repo: &RepoPath,
        tip: &Oid,
        base: Option<&Oid>,
    ) -> Result<Option<u32>> {
        self.count_unpublished_impl(repo, tip, base)
    }

    fn reachable_from_branches(&self, repo: &RepoPath, oid: &Oid) -> Result<bool> {
        self.reachable_from_branches_impl(repo, oid)
    }

    fn commit_files(&self, repo: &RepoPath, oid: &Oid) -> Result<CommitFiles> {
        self.commit_files_impl(repo, oid)
    }

    fn commit_messages(&self, repo: &RepoPath, oids: &[Oid]) -> Result<Vec<CommitMessage>> {
        self.commit_messages_impl(repo, oids)
    }

    fn tags(&self, repo: &RepoPath) -> Result<Vec<TagRef>> {
        let out = self.runner.run(
            Some(repo.as_path()),
            &[
                "for-each-ref",
                "--count=1000",
                branches::TAG_FORMAT,
                "refs/tags/",
            ],
        )?;
        branches::parse_tags(&out.stdout_lossy())
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
        let out = match &target.kind {
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
                self.runner.run_capped(dir, &args, diff::MAX_OUTPUT)?
            }
            DiffKind::Untracked => {
                // 아주 큰 새 파일은 git이 통째로 읽지 않게 앞부분만 직접 읽는다.
                let full = repo.as_path().join(&target.path);
                if let Ok(meta) = std::fs::symlink_metadata(&full)
                    && meta.is_file()
                    && meta.len() > diff::DIRECT_READ_OVER
                {
                    return diff::untracked_prefix(&full);
                }
                let mut args = vec!["diff", "--no-index"];
                args.extend_from_slice(diff::COMMON_ARGS);
                args.extend(["--", "/dev/null", &target.path]);
                // --no-index는 차이가 있으면 종료 코드 1이다(오류가 아님). run_capped가 받는다.
                self.runner.run_capped(dir, &args, diff::MAX_OUTPUT)?
            }
            DiffKind::Commit { oid } => {
                // 그 버전에서 바뀐 것: 첫 부모(첫 버전이면 빈 트리)와 비교한다(D-105).
                let revs = self.commit_diff_revs(repo, oid)?;
                let mut args = vec!["--literal-pathspecs", "diff-tree", "-p", "--no-commit-id"];
                args.extend_from_slice(diff::COMMON_ARGS);
                if revs.len() == 1 {
                    args.push("--root");
                }
                let rev_refs: Vec<&str> = revs.iter().map(String::as_str).collect();
                push_revs(&mut args, Cmd::DiffTree, self.git_version(), &rev_refs);
                args.push("--");
                args.push(&target.path);
                if let Some(orig) = &target.orig_path {
                    args.push(orig);
                }
                self.runner.run_capped(dir, &args, diff::MAX_OUTPUT)?
            }
        };
        let mut parsed = diff::parse(&String::from_utf8_lossy(&out.stdout))?;
        parsed.truncated |= out.truncated;
        Ok(parsed)
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

    fn common_dir(&self, repo: &RepoPath) -> Result<PathBuf> {
        // `--path-format=absolute`는 2.31부터라, 상대 경로면 저장소 최상위(실행 위치) 기준으로 붙인다.
        let out = self
            .runner
            .run(Some(repo.as_path()), &["rev-parse", "--git-common-dir"])?;
        let text = out.stdout_utf8()?;
        let path = PathBuf::from(text.trim_end_matches(['\n', '\r']));
        Ok(if path.is_absolute() {
            path
        } else {
            repo.as_path().join(path)
        })
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

    fn merge_check(
        &self,
        repo: &RepoPath,
        base: Option<&str>,
        ours: &str,
        theirs: &str,
    ) -> Result<MergeCheck> {
        self.merge_check_impl(repo, base, ours, theirs)
    }

    fn write_tree(&self, repo: &RepoPath) -> Result<Option<Oid>> {
        match self.runner.run(Some(repo.as_path()), &["write-tree"]) {
            Ok(out) => Ok(Oid::parse(&out.stdout_lossy())),
            // 합치지 못한 파일이 index에 있으면 트리를 만들 수 없다.
            Err(GitError::Failed {
                code: Some(128), ..
            }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn current_branch(&self, repo: &RepoPath) -> Result<Option<String>> {
        let mut args = vec!["symbolic-ref", "-q", "--short"];
        push_revs(&mut args, Cmd::SymbolicRef, self.git_version(), &["HEAD"]);
        match self.runner.run(Some(repo.as_path()), &args) {
            Ok(out) => {
                let name = out
                    .stdout_lossy()
                    .trim_end_matches(['\n', '\r'])
                    .to_string();
                Ok((!name.is_empty()).then_some(name))
            }
            // 특정 버전을 보는 중이면 종료 코드 1(-q).
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn parked_paths(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>> {
        self.parked_paths_impl(repo, oid)
    }

    fn local_config(&self, repo: &RepoPath) -> Result<Vec<(String, String)>> {
        // `--local`은 `extensions.worktreeConfig`의 config.worktree를 빠뜨리므로 범위를 함께 받아
        // local·worktree만 남긴다. 전역·시스템과 앱이 붙인 `-c`(command)는 뺀다.
        // 레코드는 `범위\0키\n값\0`. 설정이 하나도 없으면 종료 코드 1.
        let out = match self.runner.run(
            Some(repo.as_path()),
            &["config", "--show-scope", "--includes", "--list", "-z"],
        ) {
            Ok(out) => out,
            Err(GitError::Failed { code: Some(1), .. }) => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let text = out.stdout_lossy();
        let fields: Vec<&str> = text.split('\0').collect();
        let (pairs, _) = fields.as_chunks::<2>();
        Ok(pairs
            .iter()
            .filter(|[scope, _]| matches!(*scope, "local" | "worktree"))
            .map(|[_, record]| match record.split_once('\n') {
                Some((k, v)) => (k.to_string(), v.to_string()),
                // 값 없이 키만 쓴 불리언(`[core] bare`)
                None => (record.to_string(), String::new()),
            })
            .collect())
    }

    fn reflog(&self, repo: &RepoPath, limit: u32) -> Result<Vec<ReflogEntry>> {
        if crate::GitBackend::resolve_ref(self, repo, "HEAD")?.is_none() {
            return Ok(Vec::new());
        }
        let count = format!("-n{limit}");
        let out = self.runner.run(
            Some(repo.as_path()),
            &[
                "log",
                "-g",
                "-z",
                &count,
                "--format=%H%x00%ct%x00%gs%x00%s",
                "HEAD",
            ],
        )?;
        let text = out.stdout_lossy();
        let fields: Vec<&str> = text.split('\0').collect();
        let (records, _) = fields.as_chunks::<4>();
        Ok(records
            .iter()
            .filter_map(|[oid, at, subject, summary]| {
                Some(ReflogEntry {
                    oid: Oid::parse(oid.trim_start_matches('\n'))?,
                    at: at.parse().ok()?,
                    subject: subject.to_string(),
                    summary: summary.to_string(),
                })
            })
            .collect())
    }

    fn stash_list(&self, repo: &RepoPath) -> Result<Vec<StashEntry>> {
        self.stashes(repo)
    }

    fn index_entries(&self, repo: &RepoPath) -> Result<Vec<IndexEntry>> {
        self.ls_index(repo)
    }

    fn read_blobs(&self, repo: &RepoPath, oids: &[Oid], max_bytes: usize) -> Result<Vec<BlobHead>> {
        self.blob_heads(repo, oids, max_bytes)
    }

    fn merge_segments(
        &self,
        repo: &RepoPath,
        base: &[u8],
        ours: &[u8],
        theirs: &[u8],
    ) -> Result<Vec<MergeSegment>> {
        self.segments(repo, base, ours, theirs)
    }

    fn request_cancel(&self, repo: &RepoPath) {
        self.cancel_flag(repo)
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    fn clear_cancel(&self, repo: &RepoPath) {
        self.cancel_flag(repo)
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    fn set_observer(&self, observer: Option<crate::ProgressObserver>) {
        *self
            .observer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = observer;
    }

    fn other_git_running(&self) -> bool {
        // 목록을 읽는 사이에 시작하거나 끝난 앱의 git도 빼도록 앞뒤 기록을 함께 본다.
        let before = self.runner.live_pids();
        let Some(pids) = (self.runner.process.git_pids)() else {
            return true;
        };
        let after = self.runner.live_pids();
        pids.iter()
            .any(|p| !before.contains(p) && !after.contains(p))
    }

    fn set_repo_overrides(&self, repo: &RepoPath, pairs: Vec<String>) {
        self.runner.set_overrides(repo.as_path(), pairs);
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
                self.run_local_watched(repo, &["commit", "-F", "-"], Some(message.as_bytes()))?
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
            GitOp::Park { message } => return self.park_push(repo, message),
            GitOp::StashDropIf { oid } => return self.stash_drop_if(repo, oid),
            GitOp::StashDropOid { oid } => return self.stash_drop_oid(repo, oid),
            GitOp::DiscardPaths { paths, message } => {
                return self.discard_paths(repo, paths, message);
            }
            GitOp::Unpark { oid } => return self.unpark(repo, oid),
            GitOp::UnparkKeep { oid } => return self.unpark_keep(repo, oid),
            GitOp::UnparkAbort { oid } => return self.unpark_abort(repo, oid),
            GitOp::Fetch { remote, background } => {
                let mode = if *background {
                    NetMode::Background
                } else {
                    NetMode::Foreground
                };
                // FETCH_HEAD는 쓰지 않는다(바뀔 때마다 파일 감시가 알림을 낸다). 서브모듈까지 받지
                // 않는다(서브모듈 설정은 신뢰 확인 대상이 아니고, 그 안에서 명령이 돌 수 있다).
                // `--progress`: 내용은 읽지 않고 출력이 오는지로 멈춤을 본다(D-080).
                let mut args = vec![
                    "fetch",
                    "--progress",
                    "--no-prune",
                    "--no-write-fetch-head",
                    "--no-recurse-submodules",
                ];
                match remote {
                    Some(r) => push_revs(&mut args, Cmd::Fetch, self.git_version(), &[r]),
                    None => args.push("--all"),
                }
                self.run_network(repo, &args, mode)?
            }
            GitOp::AddSafeDirectory { path } => {
                if path.is_empty() || path == "*" || path.starts_with('-') {
                    return Err(GitError::Parse(format!("safe.directory 값: {path}")));
                }
                // 저장소 밖에서 돌려 그 저장소 설정의 영향을 받지 않게 한다.
                self.runner.run(
                    None,
                    &["config", "--global", "--add", "safe.directory", path],
                )?
            }
            GitOp::StageAll => self.runner.run(dir, &["add", "-A"])?,
            // reset은 2.34.1에서 `--end-of-options`를 받지 못한다. 해시는 `Oid`가 형식을 보장한다.
            GitOp::ResetSoft { oid } => self
                .runner
                .run(dir, &["reset", "--soft", "-q", oid.as_str()])?,
            GitOp::ResolveFile { path, mode, bytes } => {
                self.resolve_file(repo, path, mode, bytes)?
            }
            GitOp::ResolveDelete { path } => self.resolve_delete(repo, path)?,
            GitOp::ResolveSide { path, ours } => self.resolve_side(repo, path, *ours)?,
            GitOp::SnapshotFiles { paths, message } => {
                return self.snapshot_files(repo, paths, message);
            }
            GitOp::Revert { oid, mainline } => {
                let number;
                let mut args = vec!["revert", "--no-edit"];
                if let Some(n) = mainline {
                    number = n.to_string();
                    args.extend(["-m", number.as_str()]);
                }
                // `Oid`는 16진수만이라 옵션으로 읽히지 않는다.
                args.push(oid.as_str());
                match self.run_local_watched(repo, &args, None) {
                    Ok(out) => out,
                    Err(e) => {
                        // 충돌이나 hook 거부로 멈췄으면 시작 전으로 돌린다. 버전 하나 되돌리기가 저장 단계에서
                        // 멈추면 되돌리기 중 표시 없이 바뀐 내용만 index에 남아 `--abort`가 듣지 않는다.
                        // 그때는 넣은 내용이 있을 때만 `reset --merge`로 지운다(고친 파일은 남긴다).
                        if self.runner.run(dir, &["revert", "--abort"]).is_err()
                            && self
                                .runner
                                .run(dir, &["diff", "--cached", "--quiet"])
                                .is_err()
                        {
                            let _ = self.runner.run(dir, &["reset", "--merge", "-q"]);
                        }
                        return Err(e);
                    }
                }
            }
            // `reset`에는 `--end-of-options`를 넣지 않는다(args.rs). `Oid`는 16진수만이라 옵션으로 읽히지 않는다.
            GitOp::ResetKeep { oid } => self
                .runner
                .run(dir, &["reset", "--keep", "-q", oid.as_str()])?,
            GitOp::ReadTree { tree } => {
                let mut args = vec!["read-tree"];
                push_revs(
                    &mut args,
                    Cmd::ReadTree,
                    self.git_version(),
                    &[tree.as_str()],
                );
                self.runner.run(dir, &args)?
            }
            GitOp::MergeFf { rev } => {
                let mut args = vec!["merge", "--ff-only", "--no-edit"];
                push_revs(&mut args, Cmd::Merge, self.git_version(), &[rev]);
                self.runner.run(dir, &args)?
            }
            // 2.34.1의 merge는 `-F -`(stdin)를 읽지 못해 `-m`으로 넘긴다. 인자는 배열이라 한글도 그대로다.
            GitOp::Merge { rev, message } => {
                let mut args = vec!["merge", "--no-ff", "--no-edit", "-m", message];
                push_revs(&mut args, Cmd::Merge, self.git_version(), &[rev]);
                self.run_local_watched(repo, &args, None)?
            }
            GitOp::MergeAbort => {
                if !self.git_dir(repo)?.join("MERGE_HEAD").exists() {
                    return Ok(OpOutput::default());
                }
                self.runner.run(dir, &["merge", "--abort"])?
            }
            GitOp::Push {
                target,
                set_upstream,
            } => {
                // 태그·서브모듈·서명은 저장소 설정(push.followTags 등)을 따르지 않는다.
                let mut args = vec![
                    "push",
                    "--progress",
                    "--no-follow-tags",
                    "--no-recurse-submodules",
                    "--no-signed",
                ];
                if *set_upstream {
                    args.push("-u");
                }
                let refspec;
                if let Some((remote, branch)) = target {
                    refspec = format!("HEAD:refs/heads/{branch}");
                    push_revs(
                        &mut args,
                        Cmd::Push,
                        self.git_version(),
                        &[remote, &refspec],
                    );
                }
                self.run_network(repo, &args, NetMode::Foreground)?
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
            oid: None,
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
    fn local_config_and_repo_overrides() {
        let repo = testutil::init_repo();
        testutil::git(repo.path(), &["config", "core.fsmonitor", "touch pwned"]);
        testutil::git(repo.path(), &["config", "filter.lfs.clean", "lfs clean %f"]);
        let backend = testutil::backend();
        let path = RepoPath::new(repo.path());
        let config = backend.local_config(&path).unwrap();
        assert!(config.contains(&("core.fsmonitor".into(), "touch pwned".into())));
        assert!(config.contains(&("filter.lfs.clean".into(), "lfs clean %f".into())));

        // 덮어쓰기가 이 저장소의 호출에만 붙는다.
        backend.set_repo_overrides(&path, vec!["core.fsmonitor=false".into()]);
        let out = backend
            .runner
            .run(Some(repo.path()), &["config", "--get", "core.fsmonitor"])
            .unwrap();
        assert_eq!(out.stdout_utf8().unwrap().trim(), "false");
        // 앱이 붙인 덮어쓰기(command 범위)는 저장소 설정으로 읽지 않는다.
        assert!(
            !backend
                .local_config(&path)
                .unwrap()
                .contains(&("core.fsmonitor".into(), "false".into()))
        );
        backend.set_repo_overrides(&path, Vec::new());
        let out = backend
            .runner
            .run(Some(repo.path()), &["config", "--get", "core.fsmonitor"])
            .unwrap();
        assert_eq!(out.stdout_utf8().unwrap().trim(), "touch pwned");

        // worktree 설정도 읽는다.
        testutil::git(
            repo.path(),
            &["config", "extensions.worktreeConfig", "true"],
        );
        testutil::git(
            repo.path(),
            &["config", "--worktree", "core.sshCommand", "evil"],
        );
        let config = backend.local_config(&path).unwrap();
        assert!(config.contains(&("core.sshcommand".into(), "evil".into())));
    }

    #[test]
    fn common_dir_of_a_linked_worktree_is_the_main_git_folder() {
        let repo = testutil::init_repo();
        testutil::git(
            repo.path(),
            &["commit", "-q", "--allow-empty", "-m", "첫 버전"],
        );
        let backend = testutil::backend();
        let main = backend.common_dir(&RepoPath::new(repo.path())).unwrap();
        assert!(main.join("HEAD").is_file(), "{main:?}");
        let wt = tempfile::tempdir().unwrap();
        let wt_path = wt.path().join("wt");
        testutil::git(
            repo.path(),
            &[
                "worktree",
                "add",
                "-q",
                &wt_path.to_string_lossy(),
                "-b",
                "wt",
            ],
        );
        let common = backend.common_dir(&RepoPath::new(&wt_path)).unwrap();
        assert_eq!(common.canonicalize().unwrap(), main.canonicalize().unwrap());
        assert_ne!(
            backend
                .git_dir(&RepoPath::new(&wt_path))
                .unwrap()
                .canonicalize()
                .unwrap(),
            main.canonicalize().unwrap()
        );
    }

    #[test]
    fn dubious_ownership_is_classified_and_safe_directory_added() {
        let repo = testutil::init_repo();
        let global = tempfile::NamedTempFile::new().unwrap();
        let global_path = global.path().to_path_buf();
        let configure: ConfigureCommand =
            std::sync::Arc::new(move |cmd: &mut std::process::Command| {
                cmd.env("GIT_CONFIG_GLOBAL", &global_path)
                    .env("GIT_CONFIG_NOSYSTEM", "1")
                    // git 2.35.2부터 있는 시험용 스위치: 주인이 다른 것처럼 다룬다.
                    .env("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1");
            });
        let backend = CliBackend::new("git", configure);
        let err = backend.discover(repo.path()).unwrap_err();
        assert_eq!(
            err.kind(),
            FailureKind::DubiousOwnership,
            "{}",
            err.detail()
        );

        let top = std::fs::canonicalize(repo.path()).unwrap();
        let path = top.to_string_lossy().replace('\\', "/");
        backend
            .run(
                &RepoPath::new(repo.path()),
                &GitOp::AddSafeDirectory { path: path.clone() },
            )
            .unwrap();
        let written = std::fs::read_to_string(global.path()).unwrap();
        assert!(written.contains(&path), "{written}");
        assert!(
            backend
                .run(
                    &RepoPath::new(repo.path()),
                    &GitOp::AddSafeDirectory { path: "*".into() }
                )
                .is_err()
        );
    }

    #[test]
    fn missing_git_is_spawn_error() {
        let backend = CliBackend::new("/없는/git", std::sync::Arc::new(|_| {}));
        assert!(matches!(backend.version(), Err(GitError::Spawn(_))));
    }
}
