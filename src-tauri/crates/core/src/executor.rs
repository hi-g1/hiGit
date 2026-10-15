use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use crate::undo::journal::{self, Body, Journal};
use crate::undo::{JournalKey, UndoKind, UndoState};
use crate::view::ActionInput;
use crate::{
    Availability, CoreError, ExecCtx, JournalMode, Outcome, Preview, Registry, Result, Risk,
};

/// journal에 남길 줄 하나의 재료.
struct Line<'a> {
    action: &'static str,
    kind: UndoKind,
    input: &'a ActionInput,
    /// 실행 전 상태. 없으면 경계 줄이거나 남기지 않는다.
    before: Option<UndoState>,
    /// 실행 전후 값을 함께 남길 다른 ref.
    tracked: &'a [String],
    /// 상태가 그대로여도 남긴다.
    always: bool,
}

/// 미리보기를 보여줬다는 증표. 실행 때 한 번만 쓸 수 있다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreviewToken(pub String);

struct Ticket {
    action_id: &'static str,
    input: ActionInput,
    folder: std::path::PathBuf,
    issued_at: Instant,
}

/// 위험도와 되돌리는 방법에 따라 공통 흐름을 강제한다(D-043). 액션이 안전장치를 따로 구현하지 않는다.
///
/// - 되돌릴 수 없는(`Irreversible`) 액션은 미리보기 토큰(사용자 확인)이 있어야 한다.
/// - `caution` 이상은 실행 전에 HEAD를 백업 ref로 남기고, 못 남기면 실행하지 않는다(D-043).
/// - 되돌릴 수 있는 액션은 전후 상태를 journal에 남기고, 되돌리지 못하는 상태 변경은
///   경계 줄을 남긴다(D-065).
pub struct Executor {
    registry: Registry,
    journal_key: JournalKey,
    tickets: Mutex<HashMap<String, Ticket>>,
    next: AtomicU64,
    ttl: Duration,
}

impl Executor {
    pub const DEFAULT_TTL: Duration = Duration::from_secs(10 * 60);

    pub fn new(registry: Registry) -> Self {
        Self::with_ttl(registry, Self::DEFAULT_TTL)
    }

    pub fn with_ttl(registry: Registry, ttl: Duration) -> Self {
        Self {
            registry,
            journal_key: JournalKey::ephemeral(),
            tickets: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
            ttl,
        }
    }

    /// journal 서명 키를 정한다(앱은 앱 데이터의 키, D-063).
    pub fn with_journal_key(mut self, key: JournalKey) -> Self {
        self.journal_key = key;
        self
    }

    pub(crate) fn set_journal_key(&mut self, key: JournalKey) {
        self.journal_key = key;
    }

    /// 마지막 작업을 되돌린다(D-042). 저장소 잠금은 부르는 쪽(Core)이 잡는다.
    pub fn undo(&self, ctx: &ExecCtx<'_>) -> Result<crate::undo::UndoOutcome> {
        let repo = ctx.state.repo.as_ref().ok_or(CoreError::NotARepo)?;
        let journal = Journal::at(&ctx.git.git_dir(repo)?, self.journal_key.clone());
        crate::undo::run::undo(ctx.git, repo, &journal)
    }

    /// 작업 내역(D-085). journal 서명 키를 쓰므로 실행기가 맡는다.
    pub fn recovery(&self, ctx: &ExecCtx<'_>) -> Result<Vec<crate::view::RecoveryItem>> {
        let repo = ctx.state.repo.as_ref().ok_or(CoreError::NotARepo)?;
        let journal = Journal::at(&ctx.git.git_dir(repo)?, self.journal_key.clone());
        crate::recovery::collect(ctx.git, repo, &journal)
    }

    /// 마지막으로 되돌린 작업을 다시 한다. 그 뒤에 새 작업이 있으면 할 것이 없다.
    pub fn redo(&self, ctx: &ExecCtx<'_>) -> Result<crate::undo::UndoOutcome> {
        let repo = ctx.state.repo.as_ref().ok_or(CoreError::NotARepo)?;
        let journal = Journal::at(&ctx.git.git_dir(repo)?, self.journal_key.clone());
        crate::undo::run::redo(ctx.git, repo, &journal)
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn preview(
        &self,
        ctx: &ExecCtx<'_>,
        id: &str,
        input: &ActionInput,
    ) -> Result<(Preview, PreviewToken)> {
        let action = self.available_action(ctx, id)?;
        let preview = action.preview(ctx, input)?;

        let token = format!("p{}", self.next.fetch_add(1, Ordering::Relaxed));
        // 토큰은 실행 때 검사하는 되돌릴 수 없는 액션만 보관한다. 보관할 때 만료된 것을 치운다.
        if action.meta().undo == UndoKind::Irreversible {
            let mut tickets = self.tickets.lock().unwrap_or_else(PoisonError::into_inner);
            tickets.retain(|_, t| t.issued_at.elapsed() < self.ttl);
            tickets.insert(
                token.clone(),
                Ticket {
                    action_id: action.meta().id,
                    input: input.clone(),
                    folder: ctx.state.folder.clone(),
                    issued_at: Instant::now(),
                },
            );
        }
        Ok((preview, PreviewToken(token)))
    }

    pub fn execute(
        &self,
        ctx: &ExecCtx<'_>,
        id: &str,
        input: &ActionInput,
        token: Option<&PreviewToken>,
    ) -> Result<Outcome> {
        let action = self.available_action(ctx, id)?;
        let meta = action.meta();

        if meta.undo == UndoKind::Irreversible {
            let token = token.ok_or(CoreError::PreviewRequired)?;
            self.take_ticket(ctx, meta.id, input, token)?;
        }
        let repo = ctx.state.repo.as_ref();
        if meta.risk >= Risk::Caution
            && action.moves_head()
            && let Some(repo) = repo
            && let Some(head) = ctx.git.resolve_ref(repo, "HEAD")?
        {
            // 지우거나 덮어쓰기 전에 지금 버전을 살려 둔다. 못 남기면 실행하지 않는다(D-034 4번).
            crate::undo::backup::backup_oid(ctx.git, repo, &head, meta.id)?;
        }

        let tracked = action.tracked_refs(input);
        let before = match repo {
            Some(r) if meta.undo.is_recorded() => {
                Some(UndoState::capture_with(ctx.git, r, &tracked)?)
            }
            _ => None,
        };
        let outcome = action.execute(ctx, input)?;
        if let Some(repo) = repo {
            // 작업은 이미 끝났다. 기록을 못 남기면 되돌리기만 안 될 뿐이라 결과는 그대로 돌려준다.
            let mode = outcome.journal_mode();
            let line = Line {
                action: meta.id,
                kind: if mode == JournalMode::Barrier {
                    UndoKind::Barrier
                } else {
                    meta.undo
                },
                input,
                before: before.filter(|_| mode != JournalMode::Barrier),
                tracked: &tracked,
                always: mode == JournalMode::Always,
            };
            if mode != JournalMode::Skip {
                let _ = self.record(ctx, repo, line);
            }
        }
        Ok(outcome)
    }

    fn record(&self, ctx: &ExecCtx<'_>, repo: &git::RepoPath, line: Line<'_>) -> Result<()> {
        let at = journal::now();
        let action = line.action;
        let body = match line.before {
            Some(before) => {
                let after = UndoState::capture_with(ctx.git, repo, line.tracked)?;
                // 아무것도 바뀌지 않았으면(주고받을 것이 없거나 겹쳐서 멈춘 동기화, 올리기만 한 경우)
                // 되돌릴 것도 없다. 줄을 남기면 그 앞의 작업을 되돌리지 못하게 막는다.
                if after == before && !line.always {
                    return Ok(());
                }
                // 충돌 중이라 index 트리를 만들 수 없으면 넣기·빼기를 되돌릴 수 없다(경계).
                if line.kind == UndoKind::RestoreIndex
                    && (before.index_tree.is_none() || after.index_tree.is_none())
                {
                    Body::Barrier {
                        at,
                        action: action.to_string(),
                    }
                } else {
                    Body::Action {
                        id: journal::new_id(at),
                        at,
                        action: action.to_string(),
                        input: line.input.journal_copy(),
                        before,
                        after,
                        kind: line.kind,
                    }
                }
            }
            None if line.kind.is_barrier() => Body::Barrier {
                at,
                action: action.to_string(),
            },
            None => return Ok(()),
        };
        Journal::at(&ctx.git.git_dir(repo)?, self.journal_key.clone()).append(&body)
    }

    fn available_action(&self, ctx: &ExecCtx<'_>, id: &str) -> Result<&dyn crate::Action> {
        let action = self
            .registry
            .get(id)
            .ok_or_else(|| CoreError::UnknownAction(id.to_string()))?;
        match action.available(ctx.state) {
            Availability::Available => Ok(action),
            Availability::Unavailable(reason) => Err(CoreError::Unavailable(reason)),
        }
    }

    /// 토큰을 꺼내 쓰고(한 번만), 같은 액션·입력·폴더이며 만료되지 않았는지 확인한다.
    fn take_ticket(
        &self,
        ctx: &ExecCtx<'_>,
        action_id: &str,
        input: &ActionInput,
        token: &PreviewToken,
    ) -> Result<()> {
        let ticket = self
            .tickets
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&token.0)
            .ok_or(CoreError::PreviewMismatch)?;
        let matches = ticket.action_id == action_id
            && ticket.input == *input
            && ticket.folder == ctx.state.folder
            && ticket.issued_at.elapsed() < self.ttl;
        if matches {
            Ok(())
        } else {
            Err(CoreError::PreviewMismatch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, ActionMeta, RepoState};
    use git::{Commit, GitBackend, GitOp, GitVersion, LogQuery, OpOutput, RepoPath, Status};

    struct NoGit;
    impl GitBackend for NoGit {
        fn version(&self) -> git::Result<GitVersion> {
            unreachable!()
        }
        fn discover(&self, _: &std::path::Path) -> git::Result<Option<RepoPath>> {
            unreachable!()
        }
        fn status(&self, _: &RepoPath) -> git::Result<Status> {
            unreachable!()
        }
        fn log(&self, _: &RepoPath, _: &LogQuery) -> git::Result<Vec<Commit>> {
            unreachable!()
        }
        fn excludes_file(&self, _: &RepoPath) -> git::Result<Option<std::path::PathBuf>> {
            unreachable!()
        }
        fn diff(&self, _: &RepoPath, _: &git::DiffTarget) -> git::Result<git::FileDiff> {
            unreachable!()
        }
        fn branches(&self, _: &RepoPath) -> git::Result<Vec<git::Branch>> {
            unreachable!()
        }
        fn remotes(&self, _: &RepoPath) -> git::Result<Vec<git::Remote>> {
            unreachable!()
        }
        fn list_refs(&self, _: &RepoPath, _: &str) -> git::Result<Vec<git::RefEntry>> {
            unreachable!()
        }
        fn remote_head(&self, _: &RepoPath, _: &str) -> git::Result<Option<String>> {
            unreachable!()
        }
        fn check_branch_name(&self, _: &str) -> git::Result<bool> {
            unreachable!()
        }
        fn resolve_ref(&self, _: &RepoPath, _: &str) -> git::Result<Option<git::Oid>> {
            unreachable!()
        }
        fn update_ref(
            &self,
            _: &RepoPath,
            _: &str,
            _: &git::Oid,
            _: &git::RefExpect,
        ) -> git::Result<()> {
            unreachable!()
        }
        fn delete_ref(&self, _: &RepoPath, _: &str, _: &git::RefExpect) -> git::Result<()> {
            unreachable!()
        }
        fn git_dir(&self, _: &RepoPath) -> git::Result<std::path::PathBuf> {
            unreachable!()
        }
        fn common_dir(&self, _: &RepoPath) -> git::Result<std::path::PathBuf> {
            unreachable!()
        }
        fn is_ancestor(&self, _: &RepoPath, _: &str, _: &str) -> git::Result<bool> {
            unreachable!()
        }
        fn count_between(&self, _: &RepoPath, _: &str, _: &str) -> git::Result<(u32, u32)> {
            unreachable!()
        }
        fn merge_base(&self, _: &RepoPath, _: &str, _: &str) -> git::Result<Option<git::Oid>> {
            unreachable!()
        }
        fn merge_check(
            &self,
            _: &RepoPath,
            _: Option<&str>,
            _: &str,
            _: &str,
        ) -> git::Result<git::MergeCheck> {
            unreachable!()
        }
        fn parked_paths(&self, _: &RepoPath, _: &git::Oid) -> git::Result<Vec<String>> {
            unreachable!()
        }
        fn write_tree(&self, _: &RepoPath) -> git::Result<Option<git::Oid>> {
            unreachable!()
        }
        fn current_branch(&self, _: &RepoPath) -> git::Result<Option<String>> {
            unreachable!()
        }
        fn reflog(&self, _: &RepoPath, _: u32) -> git::Result<Vec<git::ReflogEntry>> {
            unreachable!()
        }
        fn stash_list(&self, _: &RepoPath) -> git::Result<Vec<git::StashEntry>> {
            unreachable!()
        }
        fn index_entries(&self, _: &RepoPath) -> git::Result<Vec<git::IndexEntry>> {
            unreachable!()
        }
        fn read_blobs(
            &self,
            _: &RepoPath,
            _: &[git::Oid],
            _: usize,
        ) -> git::Result<Vec<git::BlobHead>> {
            unreachable!()
        }
        fn merge_segments(
            &self,
            _: &RepoPath,
            _: &[u8],
            _: &[u8],
            _: &[u8],
        ) -> git::Result<Vec<git::MergeSegment>> {
            unreachable!()
        }
        fn local_config(&self, _: &RepoPath) -> git::Result<Vec<(String, String)>> {
            unreachable!()
        }
        fn set_repo_overrides(&self, _: &RepoPath, _: Vec<String>) {
            unreachable!()
        }
        fn run(&self, _: &RepoPath, _: &GitOp) -> git::Result<OpOutput> {
            unreachable!()
        }
    }

    struct Noop(ActionMeta);
    impl Action for Noop {
        fn meta(&self) -> &ActionMeta {
            &self.0
        }
        fn available(&self, _: &RepoState) -> Availability {
            Availability::Available
        }
        fn execute(&self, _: &ExecCtx<'_>, _: &ActionInput) -> Result<Outcome> {
            unreachable!()
        }
    }

    fn executor(ttl: Duration) -> Executor {
        let mut registry = Registry::new();
        for (id, risk) in [("safe", Risk::Safe), ("caution", Risk::Caution)] {
            let meta = ActionMeta {
                id,
                glossary_id: "commit",
                risk,
                undo: if risk >= Risk::Caution {
                    crate::undo::UndoKind::Irreversible
                } else {
                    crate::undo::UndoKind::Nothing
                },
            };
            registry.register(Box::new(Noop(meta))).unwrap();
        }
        Executor::with_ttl(registry, ttl)
    }

    fn pending(exec: &Executor) -> usize {
        exec.tickets
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    #[test]
    fn safe_previews_are_not_kept() {
        let exec = executor(Executor::DEFAULT_TTL);
        let state = RepoState {
            folder: "/p".into(),
            repo: None,
        };
        let ctx = ExecCtx {
            git: &NoGit,
            state: &state,
        };
        for _ in 0..3 {
            exec.preview(&ctx, "safe", &ActionInput::Init).unwrap();
        }
        assert_eq!(pending(&exec), 0);
        exec.preview(&ctx, "caution", &ActionInput::Init).unwrap();
        assert_eq!(pending(&exec), 1);
    }

    #[test]
    fn expired_previews_are_cleared() {
        let exec = executor(Duration::ZERO);
        let state = RepoState {
            folder: "/p".into(),
            repo: None,
        };
        let ctx = ExecCtx {
            git: &NoGit,
            state: &state,
        };
        for _ in 0..3 {
            exec.preview(&ctx, "caution", &ActionInput::Init).unwrap();
        }
        // 새로 넣을 때마다 만료된 것을 치우므로 마지막 하나만 남는다.
        assert_eq!(pending(&exec), 1);
    }
}
