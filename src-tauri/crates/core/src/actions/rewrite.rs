//! 기록 정리(`rebase-i`, D-100, D-106). 고른 구간의 순서 바꾸기·묶기·빼기·메시지 고치기.
//!
//! 순서: 입력 검사 → 구간 다시 계산(`guard::rewrite_chain`, 화면의 계획을 믿지 않음) → 바뀐 것이
//! 없으면 `NoChange` → 이름·이메일 확인 → 표식(고친 메시지 포함) → 실행기가 미리 남긴 `rebase-i`
//! 백업 뒤 `rebase -i`. 메시지를 고친 묶음은 `edit`으로 멈추고, 여기서 표식의 메시지로 고친 뒤
//! 이어간다. 충돌로 멈추면 표식을 두고 충돌 해결 화면으로, 충돌이 아닌 까닭(hook 등)으로 멈추면
//! 바로 그만두고 오류를 낸다.

use std::collections::HashSet;

use git::{GitOp, Oid, OperationKind, RepoPath, RewriteKind, RewriteStep};

use super::repo_of;
use crate::guard::{self, MAX_REWRITE};
use crate::operation::{self, OpKind, OpMarker};
use crate::view::{ActionInput, RewriteBlocked, RewriteInput, RewriteKindInput};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 메시지 하나와 전체의 상한.
const MAX_MESSAGE: usize = 16 * 1024;
const MAX_MESSAGES: usize = 24 * 1024;

pub struct RewriteAction {
    meta: ActionMeta,
}

impl RewriteAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "rebase-i",
                glossary_id: "rebase-i",
                // 용어표 rebase-i의 위험도(danger). Ctrl+Z로 원래 기록이 돌아와 확인은 받지 않는다.
                risk: Risk::Danger,
                undo: crate::undo::UndoKind::RestoreTree,
            },
        }
    }
}

impl Default for RewriteAction {
    fn default() -> Self {
        Self::new()
    }
}

/// 입력의 모양만 본다(구간과 맞는지는 뒤에서).
fn check_input(input: &RewriteInput) -> Result<()> {
    if input.steps.is_empty() || input.steps.len() > MAX_REWRITE {
        return Err(CoreError::InvalidInput("정리할 버전 수가 맞지 않아요"));
    }
    let mut total = 0;
    let mut prev: Option<RewriteKindInput> = None;
    for step in &input.steps {
        if step.kind == RewriteKindInput::Squash {
            if matches!(prev, None | Some(RewriteKindInput::Drop)) {
                return Err(CoreError::InvalidInput("묶을 아래 버전이 없어요"));
            }
            if step.message.is_some() {
                return Err(CoreError::InvalidInput("메시지는 묶음의 맨 아래에만 둬요"));
            }
        }
        if let Some(m) = &step.message {
            if step.kind == RewriteKindInput::Drop {
                return Err(CoreError::InvalidInput(
                    "빼는 버전에는 메시지를 두지 않아요",
                ));
            }
            if m.trim().is_empty() {
                return Err(CoreError::InvalidInput("버전 메시지가 비어 있어요"));
            }
            if m.len() > MAX_MESSAGE {
                return Err(CoreError::InvalidInput("버전 메시지가 너무 길어요"));
            }
            total += m.len();
        }
        prev = Some(step.kind);
    }
    if total > MAX_MESSAGES {
        return Err(CoreError::InvalidInput("버전 메시지가 너무 길어요"));
    }
    Ok(())
}

fn blocked_error(b: RewriteBlocked) -> CoreError {
    use RewriteBlocked as B;
    match b {
        B::Published => CoreError::PublishedHistory,
        B::HasChanges => CoreError::HasChanges,
        B::OtherOperation => CoreError::OperationInProgress,
        B::Detached => CoreError::InvalidInput("특정 버전을 보는 중에는 할 수 없어요"),
        B::HasMerge => CoreError::InvalidInput("합친 버전이 있는 기록은 정리할 수 없어요"),
        B::IncludesFirstVersion => CoreError::InvalidInput("첫 버전은 정리할 수 없어요"),
        B::TooMany => CoreError::InvalidInput("정리할 버전이 너무 많아요"),
        B::NotOnBranchLine => CoreError::InvalidInput("지금 브랜치의 이어진 버전만 정리해요"),
    }
}

impl Action for RewriteAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::RebaseI(input) = input else {
            return Err(CoreError::InputMismatch("rebase-i"));
        };
        check_input(input)?;
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let oids: Vec<String> = input.steps.iter().map(|s| s.oid.clone()).collect();
        let rule = guard::published_rule(git, repo, ctx.settings.allow_published_rewrite)?;
        let check = guard::rewrite_chain(git, repo, &oids, rule)?;
        if let Some(b) = check.blocked {
            return Err(blocked_error(b));
        }
        let chain = check
            .chain
            .ok_or(CoreError::InvalidInput("정리할 버전이 없어요"))?;
        let base = chain
            .base
            .clone()
            .ok_or(CoreError::InvalidInput("첫 버전은 정리할 수 없어요"))?;
        // 같은 해시가 두 번 오면 구간 검사(집합)가 못 알아챈다.
        if oids.iter().collect::<HashSet<_>>().len() != oids.len() {
            return Err(CoreError::InvalidInput("같은 버전이 두 번 있어요"));
        }
        let originals = git.commit_messages(
            repo,
            &chain
                .selected
                .iter()
                .filter_map(|c| Oid::parse(&c.oid))
                .collect::<Vec<_>>(),
        )?;
        let original = |oid: &str| {
            originals
                .iter()
                .find(|m| m.oid.as_str() == oid)
                .map(|m| m.message.trim_end().to_string())
        };
        // 실제로 고친 메시지만 남긴다(처음 값 그대로면 고친 것이 아니다).
        let mut messages = Vec::new();
        let mut steps = Vec::new();
        for step in &input.steps {
            let oid =
                Oid::parse(&step.oid).ok_or(CoreError::InvalidInput("버전이 해시가 아니에요"))?;
            let changed = step
                .message
                .as_ref()
                .filter(|m| Some(m.trim_end().to_string()) != original(oid.as_str()));
            let kind = match step.kind {
                RewriteKindInput::Drop => RewriteKind::Drop,
                RewriteKindInput::Squash => RewriteKind::Fixup,
                RewriteKindInput::Pick if changed.is_some() => RewriteKind::Edit,
                RewriteKindInput::Pick => RewriteKind::Pick,
            };
            if let Some(m) = changed {
                messages.push((oid.to_string(), m.clone()));
            }
            steps.push(RewriteStep { oid, kind });
        }
        // 고른 구간의 원래 순서(옛 것부터)와 같고 모두 그대로면 할 일이 없다.
        let same_order = chain
            .selected
            .iter()
            .rev()
            .map(|c| c.oid.as_str())
            .eq(steps.iter().map(|s| s.oid.as_str()));
        if same_order && steps.iter().all(|s| s.kind == RewriteKind::Pick) {
            return Err(CoreError::NoChange);
        }
        let joined = steps
            .iter()
            .filter(|s| s.kind == RewriteKind::Fixup)
            .count();
        let dropped = steps.iter().filter(|s| s.kind == RewriteKind::Drop).count();
        let kept = steps.len() - joined - dropped;
        // 구간 위(새 쪽)의 버전은 그대로 이어 붙인다.
        for c in chain.above.iter().rev() {
            let oid =
                Oid::parse(&c.oid).ok_or(CoreError::InvalidInput("버전이 해시가 아니에요"))?;
            steps.push(RewriteStep {
                oid,
                kind: RewriteKind::Pick,
            });
        }
        if !git.has_identity(repo)? {
            return Err(CoreError::IdentityMissing);
        }
        if rule == guard::PublishedRule::Allow
            && guard::touches_published(git, repo, &chain.head, &base)?
        {
            // 바꾸기 전에 남긴다: 못 남기면 바꾸지 않는다(D-129).
            crate::rewrote::record(git, repo, ctx.key)?;
        }
        let git_dir = git.git_dir(repo)?;
        let marker = OpMarker {
            kind: OpKind::Rebase,
            branch: chain.branch.clone(),
            head_before: chain.head.to_string(),
            base: Some(base.to_string()),
            oids: steps.iter().map(|s| s.oid.to_string()).collect(),
            started_at: crate::undo::journal::now(),
            messages,
        };
        operation::write(&git_dir, ctx.key, &marker)?;
        let started = git.run(
            repo,
            &GitOp::RewriteHistory {
                base: base.clone(),
                steps,
            },
        );
        let error = started.err();
        match drive(ctx, repo, &marker, error)? {
            Some(paths) => Ok(Outcome::OperationConflicts {
                context: OpKind::Rebase.context(),
                paths,
            }),
            None => Ok(Outcome::Rewritten {
                kept: u32::try_from(kept).unwrap_or(u32::MAX),
                dropped: u32::try_from(dropped).unwrap_or(u32::MAX),
                joined: u32::try_from(joined).unwrap_or(u32::MAX),
            }),
        }
    }
}

/// 기록 정리를 끝까지 몬다: `edit`으로 멈추면 표식의 메시지로 고치고 이어간다. 끝나면 표식을
/// 지우고 `None`, 충돌로 멈추면 표식을 두고 충돌 경로. 충돌이 아닌 까닭으로 멈추거나 서명 키가
/// 이번 실행만의 것이면 그만두고(원래 기록으로) 오류. `error`는 방금 부른 git의 실패.
pub(crate) fn drive(
    ctx: &ExecCtx<'_>,
    repo: &RepoPath,
    marker: &OpMarker,
    mut error: Option<git::GitError>,
) -> Result<Option<Vec<String>>> {
    let git = ctx.git;
    let git_dir = git.git_dir(repo)?;
    let abort = |e: CoreError| -> Result<Option<Vec<String>>> {
        let _ = git.run(
            repo,
            &GitOp::OperationAbort {
                kind: OperationKind::Rebase,
            },
        );
        operation::clear(&git_dir);
        Err(e)
    };
    // 할 일 목록 수만큼만 돈다(멈출 때마다 한 줄 이상 나아간다).
    for _ in 0..=marker.oids.len() {
        let Some(state) = git.rebase_state(repo)? else {
            operation::clear(&git_dir);
            return match error {
                // 시작하지 못했다(고친 파일·잠금 등). 아무것도 바뀌지 않았다.
                Some(e) => Err(e.into()),
                None => Ok(None),
            };
        };
        // 표식과 다른 정리이거나 남은 할 일 목록이 앱이 쓴 모양이 아니면 건드리지 않는다(D-102).
        if !operation::verify_state(git, repo, marker)? || !operation::verify_todo(&state, marker) {
            return Err(CoreError::OperationInProgress);
        }
        let paths = crate::actions::merge::conflict_paths(git, repo)?;
        if !paths.is_empty() {
            if !ctx.key.is_persistent() {
                return abort(
                    error
                        .map(CoreError::from)
                        .unwrap_or(CoreError::ConflictChanged),
                );
            }
            return Ok(Some(paths));
        }
        let stopped = state
            .done
            .last()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|oid| marker.message_for(oid));
        let message = match (state.stopped_at_edit, stopped) {
            (true, Some(m)) => m.to_string(),
            // 충돌도 아니고 고칠 메시지도 없는 멈춤(hook, 서명 실패 등).
            _ => {
                return abort(
                    error
                        .map(CoreError::from)
                        .unwrap_or(CoreError::ConflictChanged),
                );
            }
        };
        if let Err(e) = git.run(
            repo,
            &GitOp::Amend {
                message: Some(message),
            },
        ) {
            return abort(e.into());
        }
        error = git
            .run(
                repo,
                &GitOp::OperationContinue {
                    kind: OperationKind::Rebase,
                },
            )
            .err();
    }
    abort(CoreError::ConflictChanged)
}
