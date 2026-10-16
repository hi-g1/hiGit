//! 테스트용 가짜 GitBackend와 가짜 액션.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use git::{
    Branch, Commit, DiffTarget, FileDiff, GitOp, GitVersion, LogQuery, MergeCheck, Oid, OpOutput,
    RefEntry, RefExpect, Remote, RepoPath, Status,
};
use higit_core::view::ActionInput;
use higit_core::{
    Action, ActionMeta, Availability, ExecCtx, GitBackend, Outcome, RepoState, Result, Risk,
};

/// 호출을 기록하는 가짜 git. `discover`는 `repo`를 그대로 돌려준다.
pub struct FakeGit {
    pub version: GitVersion,
    pub repo: Mutex<Option<RepoPath>>,
    pub calls: Mutex<Vec<String>>,
    pub status: Mutex<Status>,
    /// 최근 것부터.
    pub commits: Mutex<Vec<Commit>>,
    pub file_diff: Mutex<FileDiff>,
    /// true이면 run이 false가 될 때까지 기다린다(잠금 시험용).
    pub hold: AtomicBool,
    /// run에 들어왔는지.
    pub entered: AtomicBool,
    pub log_calls: AtomicUsize,
    pub branches: Mutex<Vec<Branch>>,
    pub remotes: Mutex<Vec<Remote>>,
    pub refs: Mutex<Vec<RefEntry>>,
    pub remote_heads: Mutex<Vec<(String, String)>>,
    /// `ls_remote`가 볼 원격의 ref: (원격, 전체 ref 이름, 버전).
    pub server_refs: Mutex<Vec<(String, String, git::Oid)>>,
    /// `ls_remote_head`가 돌려줄 서버 기본 브랜치: (원격, 브랜치).
    pub server_heads: Mutex<Vec<(String, String)>>,
    /// `init_default_branch`가 돌려줄 값.
    pub init_default: Mutex<Option<String>>,
    pub merge_check: Mutex<MergeCheck>,
    pub parked_paths: Mutex<Vec<String>>,
    pub local_config: Mutex<Vec<(String, String)>>,
    /// 참이면 `update_ref`가 실패한다(사용자 hook이 ref 쓰기를 거부한 경우).
    pub fail_update_ref: AtomicBool,
    /// `set_observer`로 받은 진행 알림.
    pub observer: Mutex<Option<git::ProgressObserver>>,
    /// `clear_cancel`을 부른 횟수.
    pub cancel_cleared: AtomicUsize,
    /// `other_git_running`이 돌려줄 값.
    pub git_running: AtomicBool,
    pub tags: Mutex<Vec<git::TagRef>>,
    /// `count_unpublished`가 돌려줄 값.
    pub unpublished: Mutex<Option<u32>>,
    /// `set_helper_program`으로 받은 경로.
    pub helper: Mutex<Option<std::path::PathBuf>>,
    /// 원격별 올리기 주소(`pushurl` 흉내). 없으면 받기 주소와 같다.
    pub push_urls: Mutex<Vec<(String, Vec<String>)>>,
    /// 있으면 다음 `run` 하나가 이 오류로 실패한다.
    pub run_error: Mutex<Option<git::GitError>>,
    /// 전역 이름·이메일(`identity`)과 `run_global`로 받은 작업.
    pub identity: Mutex<git::GitIdentity>,
    pub global_ops: Mutex<Vec<git::GlobalOp>>,
}

impl FakeGit {
    pub fn new() -> Self {
        Self {
            version: GitVersion::new(2, 45, 0),
            repo: Mutex::new(None),
            calls: Mutex::new(Vec::new()),
            status: Mutex::new(Status {
                branch: Default::default(),
                entries: Vec::new(),
            }),
            commits: Mutex::new(Vec::new()),
            file_diff: Mutex::new(FileDiff::default()),
            hold: AtomicBool::new(false),
            entered: AtomicBool::new(false),
            log_calls: AtomicUsize::new(0),
            branches: Mutex::new(Vec::new()),
            remotes: Mutex::new(Vec::new()),
            refs: Mutex::new(Vec::new()),
            remote_heads: Mutex::new(Vec::new()),
            server_refs: Mutex::new(Vec::new()),
            server_heads: Mutex::new(Vec::new()),
            init_default: Mutex::new(None),
            merge_check: Mutex::new(MergeCheck::Clean),
            parked_paths: Mutex::new(Vec::new()),
            local_config: Mutex::new(Vec::new()),
            fail_update_ref: AtomicBool::new(false),
            observer: Mutex::new(None),
            cancel_cleared: AtomicUsize::new(0),
            git_running: AtomicBool::new(false),
            tags: Mutex::new(Vec::new()),
            unpublished: Mutex::new(Some(0)),
            helper: Mutex::new(None),
            push_urls: Mutex::new(Vec::new()),
            run_error: Mutex::new(None),
            identity: Mutex::new(git::GitIdentity::default()),
            global_ops: Mutex::new(Vec::new()),
        }
    }

    pub fn with_version(version: GitVersion) -> Self {
        Self {
            version,
            ..Self::new()
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl GitBackend for FakeGit {
    fn version(&self) -> git::Result<GitVersion> {
        Ok(self.version)
    }

    fn discover(&self, _path: &Path) -> git::Result<Option<RepoPath>> {
        Ok(self.repo.lock().unwrap().clone())
    }

    fn status(&self, _repo: &RepoPath) -> git::Result<Status> {
        Ok(self.status.lock().unwrap().clone())
    }

    fn log(&self, _repo: &RepoPath, query: &LogQuery) -> git::Result<Vec<Commit>> {
        self.log_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .commits
            .lock()
            .unwrap()
            .iter()
            .skip(query.skip as usize)
            .take(query.limit as usize)
            .cloned()
            .collect())
    }

    /// 끝점은 보지 않고 `commits`를 그대로(위상 순서라고 본다).
    fn log_graph(&self, _repo: &RepoPath, query: &git::GraphLogQuery) -> git::Result<Vec<Commit>> {
        self.log_calls.fetch_add(1, Ordering::SeqCst);
        if query.tips.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self
            .commits
            .lock()
            .unwrap()
            .iter()
            .skip(query.skip as usize)
            .take(query.limit as usize)
            .cloned()
            .collect())
    }

    fn tags(&self, _repo: &RepoPath) -> git::Result<Vec<git::TagRef>> {
        Ok(self.tags.lock().unwrap().clone())
    }

    /// 올리지 않은 버전 수: `unpublished` 값(기본 0).
    fn count_unpublished(
        &self,
        _repo: &RepoPath,
        _tip: &Oid,
        _base: Option<&Oid>,
    ) -> git::Result<Option<u32>> {
        Ok(*self.unpublished.lock().unwrap())
    }

    fn reachable_from_branches(&self, _repo: &RepoPath, _oid: &Oid) -> git::Result<bool> {
        Ok(true)
    }

    fn rebase_state(&self, _repo: &RepoPath) -> git::Result<Option<git::RebaseState>> {
        Ok(None)
    }

    fn has_identity(&self, _repo: &RepoPath) -> git::Result<bool> {
        Ok(true)
    }

    fn check_tag_name(&self, name: &str) -> git::Result<bool> {
        Ok(!name.is_empty() && !name.starts_with('-') && !name.contains(' '))
    }

    fn commit_files(&self, _repo: &RepoPath, _oid: &Oid) -> git::Result<git::CommitFiles> {
        Ok(git::CommitFiles::default())
    }

    fn commit_messages(
        &self,
        _repo: &RepoPath,
        oids: &[Oid],
    ) -> git::Result<Vec<git::CommitMessage>> {
        let commits = self.commits.lock().unwrap();
        Ok(oids
            .iter()
            .filter_map(|o| {
                commits
                    .iter()
                    .find(|c| c.oid == o.as_str())
                    .map(|c| git::CommitMessage {
                        oid: o.clone(),
                        message: c.summary.clone(),
                    })
            })
            .collect())
    }

    /// 버전마다 파일 1개, 해시 첫 글자 값만큼 더한 줄(시험용 고정값).
    fn commit_stats(&self, _repo: &RepoPath, oids: &[Oid]) -> git::Result<Vec<git::CommitStat>> {
        Ok(oids
            .iter()
            .map(|o| git::CommitStat {
                oid: o.clone(),
                files: 1,
                added: u32::from_str_radix(&o.as_str()[..1], 16).unwrap_or(0),
                removed: 0,
            })
            .collect())
    }

    fn excludes_file(&self, _repo: &RepoPath) -> git::Result<Option<PathBuf>> {
        Ok(None)
    }

    fn diff(&self, _repo: &RepoPath, _target: &DiffTarget) -> git::Result<FileDiff> {
        Ok(self.file_diff.lock().unwrap().clone())
    }

    fn branches(&self, _repo: &RepoPath) -> git::Result<Vec<Branch>> {
        Ok(self.branches.lock().unwrap().clone())
    }

    fn remotes(&self, _repo: &RepoPath) -> git::Result<Vec<Remote>> {
        Ok(self.remotes.lock().unwrap().clone())
    }

    fn remote_urls(&self, _repo: &RepoPath, remote: &str) -> git::Result<git::RemoteUrls> {
        let urls: Vec<String> = self
            .remotes
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.name == remote)
            .map(|r| r.url.clone())
            .collect();
        let push = self
            .push_urls
            .lock()
            .unwrap()
            .iter()
            .find(|(name, _)| name == remote)
            .map(|(_, urls)| urls.clone())
            .unwrap_or_else(|| urls.clone());
        Ok(git::RemoteUrls { fetch: urls, push })
    }

    fn list_refs(&self, _repo: &RepoPath, prefix: &str) -> git::Result<Vec<RefEntry>> {
        Ok(self
            .refs
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.name.starts_with(prefix))
            .cloned()
            .collect())
    }

    fn init_default_branch(&self, _repo: &RepoPath) -> git::Result<Option<String>> {
        Ok(self.init_default.lock().unwrap().clone())
    }

    fn ls_remote(
        &self,
        _repo: &RepoPath,
        remote: &str,
        refname: &str,
        _auth: git::AuthMode,
    ) -> git::Result<Option<git::Oid>> {
        Ok(self
            .server_refs
            .lock()
            .unwrap()
            .iter()
            .find(|(r, name, _)| r == remote && name == refname)
            .map(|(_, _, oid)| oid.clone()))
    }

    fn ls_remote_head(
        &self,
        _repo: &RepoPath,
        remote: &str,
        _auth: git::AuthMode,
    ) -> git::Result<Option<String>> {
        Ok(self
            .server_heads
            .lock()
            .unwrap()
            .iter()
            .find(|(r, _)| r == remote)
            .map(|(_, b)| b.clone()))
    }

    fn remote_head(&self, _repo: &RepoPath, remote: &str) -> git::Result<Option<String>> {
        Ok(self
            .remote_heads
            .lock()
            .unwrap()
            .iter()
            .find(|(r, _)| r == remote)
            .map(|(_, b)| b.clone()))
    }

    fn check_branch_name(&self, name: &str) -> git::Result<bool> {
        Ok(!name.is_empty()
            && !name.starts_with('-')
            && !name.contains(' ')
            && !name.contains(".."))
    }

    fn resolve_ref(&self, _repo: &RepoPath, name: &str) -> git::Result<Option<Oid>> {
        if name == "HEAD" {
            let oid = self.status.lock().unwrap().branch.oid.clone();
            return Ok(oid.as_deref().and_then(Oid::parse));
        }
        Ok(self
            .refs
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.name == name)
            .and_then(|r| Oid::parse(&r.oid)))
    }

    fn update_ref(
        &self,
        _repo: &RepoPath,
        name: &str,
        new: &Oid,
        expect: &RefExpect,
    ) -> git::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("update-ref {name} {new}"));
        if self.fail_update_ref.load(Ordering::SeqCst) {
            return Err(fake_failure("hook declined"));
        }
        let mut refs = self.refs.lock().unwrap();
        let current = refs.iter().position(|r| r.name == name);
        let ok = match expect {
            RefExpect::Any => true,
            RefExpect::Absent => current.is_none(),
            RefExpect::Is(oid) => current.is_some_and(|i| refs[i].oid == oid.as_str()),
        };
        if !ok {
            return Err(fake_failure("cannot lock ref"));
        }
        match current {
            Some(i) => refs[i].oid = new.to_string(),
            None => refs.push(RefEntry {
                name: name.to_string(),
                oid: new.to_string(),
                time: 0,
                summary: String::new(),
            }),
        }
        Ok(())
    }

    fn delete_ref(&self, _repo: &RepoPath, name: &str, expect: &RefExpect) -> git::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("delete-ref {name}"));
        let mut refs = self.refs.lock().unwrap();
        let current = refs.iter().position(|r| r.name == name);
        if let (RefExpect::Is(oid), Some(i)) = (expect, current)
            && refs[i].oid != oid.as_str()
        {
            return Err(fake_failure("cannot lock ref"));
        }
        if let Some(i) = current {
            refs.remove(i);
        }
        Ok(())
    }

    fn git_dir(&self, repo: &RepoPath) -> git::Result<PathBuf> {
        Ok(repo.as_path().join(".git"))
    }

    fn common_dir(&self, repo: &RepoPath) -> git::Result<PathBuf> {
        self.git_dir(repo)
    }

    fn is_ancestor(&self, _repo: &RepoPath, _a: &str, _b: &str) -> git::Result<bool> {
        Ok(false)
    }

    fn count_between(&self, _repo: &RepoPath, _l: &str, _r: &str) -> git::Result<(u32, u32)> {
        Ok((0, 0))
    }

    fn merge_base(&self, _repo: &RepoPath, _a: &str, _b: &str) -> git::Result<Option<Oid>> {
        Ok(None)
    }

    fn merge_check(
        &self,
        _repo: &RepoPath,
        _base: Option<&str>,
        _ours: &str,
        _theirs: &str,
    ) -> git::Result<MergeCheck> {
        Ok(self.merge_check.lock().unwrap().clone())
    }

    fn write_tree(&self, _repo: &RepoPath) -> git::Result<Option<Oid>> {
        Ok(Oid::parse(&"e".repeat(40)))
    }

    fn current_branch(&self, _repo: &RepoPath) -> git::Result<Option<String>> {
        Ok(self.status.lock().unwrap().branch.head.clone())
    }

    fn parked_paths(&self, _repo: &RepoPath, _oid: &Oid) -> git::Result<Vec<String>> {
        Ok(self.parked_paths.lock().unwrap().clone())
    }

    fn reflog(&self, _repo: &RepoPath, _limit: u32) -> git::Result<Vec<git::ReflogEntry>> {
        Ok(Vec::new())
    }

    fn stash_list(&self, _repo: &RepoPath) -> git::Result<Vec<git::StashEntry>> {
        Ok(Vec::new())
    }

    fn index_entries(&self, _repo: &RepoPath) -> git::Result<Vec<git::IndexEntry>> {
        Ok(Vec::new())
    }

    fn read_blobs(
        &self,
        _repo: &RepoPath,
        _oids: &[Oid],
        _max_bytes: usize,
    ) -> git::Result<Vec<git::BlobHead>> {
        Ok(Vec::new())
    }

    fn merge_segments(
        &self,
        _repo: &RepoPath,
        _base: &[u8],
        _ours: &[u8],
        _theirs: &[u8],
    ) -> git::Result<Vec<git::MergeSegment>> {
        Ok(Vec::new())
    }

    fn local_config(&self, _repo: &RepoPath) -> git::Result<Vec<(String, String)>> {
        Ok(self.local_config.lock().unwrap().clone())
    }

    fn request_cancel(&self, repo: &RepoPath) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("cancel {}", repo.as_path().display()));
    }

    fn clear_cancel(&self, _repo: &RepoPath) {
        self.cancel_cleared.fetch_add(1, Ordering::SeqCst);
    }

    fn set_helper_program(&self, path: Option<std::path::PathBuf>) {
        *self.helper.lock().unwrap() = path;
    }

    fn set_observer(&self, observer: Option<git::ProgressObserver>) {
        *self.observer.lock().unwrap() = observer;
    }

    fn other_git_running(&self) -> bool {
        self.git_running.load(Ordering::SeqCst)
    }

    fn set_repo_overrides(&self, repo: &RepoPath, pairs: Vec<String>) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("overrides {} {pairs:?}", repo.as_path().display()));
    }

    fn run(&self, repo: &RepoPath, op: &GitOp) -> git::Result<OpOutput> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{op:?} {}", repo.as_path().display()));
        self.entered.store(true, Ordering::SeqCst);
        while self.hold.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        if let Some(err) = self.run_error.lock().unwrap().take() {
            return Err(err);
        }
        match op {
            GitOp::Init => *self.repo.lock().unwrap() = Some(repo.clone()),
            GitOp::Commit { message } => {
                let mut commits = self.commits.lock().unwrap();
                let n = commits.len() as u32 + 1;
                let mut c = commit(n);
                c.summary = message.lines().next().unwrap_or("").to_string();
                commits.insert(0, c);
            }
            _ => {}
        }
        Ok(OpOutput::default())
    }

    fn identity(&self) -> git::Result<git::GitIdentity> {
        Ok(self.identity.lock().unwrap().clone())
    }

    fn run_global(&self, op: &git::GlobalOp) -> git::Result<OpOutput> {
        self.global_ops.lock().unwrap().push(op.clone());
        let git::GlobalOp::SetIdentity { name, email } = op;
        *self.identity.lock().unwrap() = git::GitIdentity {
            name: Some(name.clone()),
            email: Some(email.clone()),
        };
        Ok(OpOutput::default())
    }
}

/// 분류할 수 있는 가짜 git 실패.
pub fn fake_failure(stderr: &str) -> git::GitError {
    git::GitError::Failed {
        args: Vec::new(),
        code: Some(1),
        stdout: String::new(),
        stderr: stderr.to_string(),
    }
}

/// 위험도만 정할 수 있는 가짜 액션. 실행 횟수를 센다.
pub struct FakeAction {
    meta: ActionMeta,
    pub executed: AtomicUsize,
}

impl FakeAction {
    pub fn new(id: &'static str, risk: Risk) -> Self {
        Self {
            meta: ActionMeta {
                id,
                glossary_id: "commit",
                risk,
                // caution 이상은 되돌릴 수 없는 것으로 시험한다(확인 흐름, D-043).
                undo: if risk >= Risk::Caution {
                    higit_core::undo::UndoKind::Irreversible
                } else {
                    higit_core::undo::UndoKind::Nothing
                },
            },
            executed: AtomicUsize::new(0),
        }
    }
}

impl Action for FakeAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, _state: &RepoState) -> Availability {
        Availability::Available
    }

    fn execute(&self, ctx: &ExecCtx<'_>, _input: &ActionInput) -> Result<Outcome> {
        self.executed.fetch_add(1, Ordering::SeqCst);
        Ok(Outcome::RepoCreated(RepoPath::new(&ctx.state.folder)))
    }
}

pub fn folder_state(folder: &str) -> RepoState {
    RepoState {
        folder: PathBuf::from(folder),
        repo: None,
    }
}

pub fn repo_state(folder: &str) -> RepoState {
    RepoState {
        folder: PathBuf::from(folder),
        repo: Some(RepoPath::new(folder)),
    }
}

pub fn commit(n: u32) -> Commit {
    Commit {
        oid: format!("{n:040}"),
        short_oid: format!("{n:07}"),
        parents: Vec::new(),
        author_name: "테스트".into(),
        author_email: "t@example.com".into(),
        author_time: 1_700_000_000 + i64::from(n),
        summary: format!("저장 {n}"),
    }
}

pub fn shared(git: FakeGit) -> std::sync::Arc<FakeGit> {
    std::sync::Arc::new(git)
}
