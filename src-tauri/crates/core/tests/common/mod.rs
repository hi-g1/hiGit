//! 테스트용 가짜 GitBackend와 가짜 액션.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use git::{Commit, GitOp, GitVersion, LogQuery, OpOutput, RepoPath, Status};
use higit_core::{
    Action, ActionMeta, Availability, ExecCtx, GitBackend, Outcome, RepoState, Result, Risk,
};
use serde_json::Value;

/// 호출을 기록하는 가짜 git. `discover`는 `repo`를 그대로 돌려준다.
pub struct FakeGit {
    pub version: GitVersion,
    pub repo: Mutex<Option<RepoPath>>,
    pub calls: Mutex<Vec<String>>,
    pub status: Mutex<Status>,
    /// 최근 것부터.
    pub commits: Mutex<Vec<Commit>>,
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

    fn run(&self, repo: &RepoPath, op: &GitOp) -> git::Result<OpOutput> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{op:?} {}", repo.as_path().display()));
        if *op == GitOp::Init {
            *self.repo.lock().unwrap() = Some(repo.clone());
        }
        Ok(OpOutput::default())
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
                input_schema: Vec::new(),
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

    fn execute(&self, ctx: &ExecCtx<'_>, _input: &Value) -> Result<Outcome> {
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
