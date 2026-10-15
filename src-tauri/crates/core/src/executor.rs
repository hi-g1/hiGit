use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::{Availability, CoreError, ExecCtx, Outcome, Preview, Registry, Result, Risk};

/// 미리보기를 보여줬다는 증표. 실행 때 한 번만 쓸 수 있다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreviewToken(pub String);

struct Ticket {
    action_id: &'static str,
    input: Value,
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
        input: &Value,
    ) -> Result<(Preview, PreviewToken)> {
        let action = self.available_action(ctx, id)?;
        let preview = action.preview(ctx, input)?;

        let token = format!("p{}", self.next.fetch_add(1, Ordering::Relaxed));
        self.tickets.lock().unwrap().insert(
            token.clone(),
            Ticket {
                action_id: action.meta().id,
                input: input.clone(),
                folder: ctx.state.folder.clone(),
                issued_at: Instant::now(),
            },
        );
        Ok((preview, PreviewToken(token)))
    }

    pub fn execute(
        &self,
        ctx: &ExecCtx<'_>,
        id: &str,
        input: &Value,
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
        input: &Value,
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
