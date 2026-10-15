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
    pub merge_check: Mutex<MergeCheck>,
    pub parked_paths: Mutex<Vec<String>>,
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
            merge_check: Mutex::new(MergeCheck::Clean),
            parked_paths: Mutex::new(Vec::new()),
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

    fn parked_paths(&self, _repo: &RepoPath, _oid: &Oid) -> git::Result<Vec<String>> {
        Ok(self.parked_paths.lock().unwrap().clone())
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
