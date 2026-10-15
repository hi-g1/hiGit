use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::view::ActionInput;
use crate::{Availability, CoreError, ExecCtx, Outcome, Preview, Registry, Result, Risk};

/// 미리보기를 보여줬다는 증표. 실행 때 한 번만 쓸 수 있다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreviewToken(pub String);

struct Ticket {
    action_id: &'static str,
    input: ActionInput,
    folder: std::path::PathBuf,
    issued_at: Instant,
}

/// 위험도에 따라 공통 흐름을 강제한다(D-007). 액션이 안전장치를 따로 구현하지 않는다.
///
/// 지금 흐름: `caution` 이상은 미리보기 토큰을 확인한 뒤, 백업·되돌리기가 생기는
/// 반복 2 전까지 실행을 거부한다(D-011).
pub struct Executor {
    registry: Registry,
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
            tickets: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
            ttl,
        }
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
        // 토큰은 실행 때 검사하는 caution 이상만 보관한다. 보관할 때 만료된 것을 치운다.
        if action.meta().risk >= Risk::Caution {
            let mut tickets = self.tickets.lock().unwrap();
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

        if action.meta().risk >= Risk::Caution {
            let token = token.ok_or(CoreError::PreviewRequired)?;
            self.take_ticket(ctx, action.meta().id, input, token)?;
            // 여기서 백업 참조를 만들고 UndoJournal에 남긴 뒤 실행해야 한다(반복 2).
            return Err(CoreError::NotYetSupported);
        }

        action.execute(ctx, input)
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
            .unwrap()
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
        fn diff(&self, _: &RepoPath, _: &git::DiffTarget) -> git::Result<git::FileDiff> {
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
            };
            registry.register(Box::new(Noop(meta))).unwrap();
        }
        Executor::with_ttl(registry, ttl)
    }

    fn pending(exec: &Executor) -> usize {
        exec.tickets.lock().unwrap().len()
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
