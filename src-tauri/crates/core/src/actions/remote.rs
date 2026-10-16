//! 원격 확인·가져오기·올리기·동기화(D-045, D-048, 계획 005 H13).
//!
//! 동기화 = 원격 확인 → (받을 것이 있으면) 합치기 → (보낼 것이 있으면) 올리기. 합칠 때 겹치면
//! 합치기를 시작하지 않는다(충돌 해결은 반복 2). 겹침은 작업 폴더를 건드리지 않고 미리 본다.

use git::{AuthMode, GitBackend, GitOp, LogQuery, RepoPath};

use super::{names, needs_repo, repo_of};
use crate::undo::backup;
use crate::view::{ActionInput, SyncInput};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 동기화 결과.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncReport {
    /// 받아 온 버전 수.
    pub received: u32,
    /// 올린 버전 수.
    pub sent: u32,
    /// 다른 사람 버전과 합친 버전을 만들었다.
    pub merged: bool,
    /// 겹친 파일. 비어 있지 않으면 올리지 않았다.
    pub overlap: Vec<String>,
    /// 겹쳐서 합치기를 시작한 채 멈췄다(충돌 해결 화면으로, D-069).
    pub merge_started: bool,
}

fn meta(
    id: &'static str,
    glossary_id: &'static str,
    risk: Risk,
    undo: crate::undo::UndoKind,
) -> ActionMeta {
    ActionMeta {
        id,
        glossary_id,
        risk,
        undo,
    }
}

pub struct RemoteAction {
    meta: ActionMeta,
}

impl RemoteAction {
    pub fn fetch() -> Self {
        Self {
            meta: meta("fetch", "fetch", Risk::Safe, crate::undo::UndoKind::Nothing),
        }
    }

    /// 1b에는 원격 작업 되돌리기(Inverse)가 없다. Ctrl+Z는 "올린 버전은 되돌리지 않아요".
    pub fn pull() -> Self {
        Self {
            meta: meta("pull", "pull", Risk::Normal, crate::undo::UndoKind::Inverse),
        }
    }

    pub fn push() -> Self {
        Self {
            meta: meta("push", "push", Risk::Normal, crate::undo::UndoKind::Inverse),
        }
    }

    pub fn sync() -> Self {
        Self {
            meta: meta("sync", "sync", Risk::Normal, crate::undo::UndoKind::Inverse),
        }
    }
}

impl Action for RemoteAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let git = ctx.git;
        let repo = repo_of(ctx)?;
        match (self.meta.id, input) {
            ("fetch", ActionInput::Fetch(i)) => {
                let remote = pick_remote(git, repo)?;
                git.run(
                    repo,
                    &GitOp::Fetch {
                        remote: Some(remote),
                        background: i.background,
                        auth: AuthMode::UserDefault,
                    },
                )?;
                Ok(Outcome::Fetched)
            }
            ("pull", ActionInput::Pull) => {
                let up = upstream(git, repo)?.ok_or(CoreError::Git(no_upstream()))?;
                fetch_remote(git, repo, &up.remote)?;
                let (_, behind) = git.count_between(repo, "HEAD", &up.rev())?;
                if behind > 0 {
                    git.run(repo, &GitOp::MergeFf { rev: up.rev() })?;
                }
                Ok(Outcome::Synced(SyncReport {
                    received: behind,
                    ..SyncReport::default()
                }))
            }
            ("push", ActionInput::Push) => {
                let sent = push(git, repo)?;
                Ok(Outcome::Synced(SyncReport {
                    sent,
                    ..SyncReport::default()
                }))
            }
            ("sync", ActionInput::Sync(i)) => sync(git, repo, i).map(Outcome::Synced),
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}

/// 지금 브랜치의 upstream. `remote`는 원격 이름, `tracking`은 `origin/main` 같은 원격 추적 이름.
struct Upstream {
    remote: String,
    tracking: String,
}

impl Upstream {
    /// git에 넘길 전체 이름. 짧은 이름(`origin/main`)은 같은 이름의 태그·로컬 브랜치가 먼저 풀린다(D-069).
    fn rev(&self) -> String {
        format!("refs/remotes/{}", self.tracking)
    }

    /// 올릴 곳을 앱이 정한다: `<원격> HEAD:refs/heads/<upstream 브랜치>`. 인자 없는 `git push`는
    /// 저장소 설정(`remote.<n>.push`, `push.default=matching`)이 무엇을 올릴지 정해, 받은 저장소의
    /// 설정이 다른 브랜치까지 덮어쓸 수 있다.
    fn push_target(&self, git: &dyn GitBackend) -> Result<(String, String)> {
        let branch = self
            .tracking
            .strip_prefix(&format!("{}/", self.remote))
            .unwrap_or(&self.tracking)
            .to_string();
        if !git.check_branch_name(&branch)? {
            return Err(CoreError::InvalidInput("올릴 수 없는 브랜치 이름이에요"));
        }
        Ok((self.remote.clone(), branch))
    }
}

fn upstream(git: &dyn GitBackend, repo: &RepoPath) -> Result<Option<Upstream>> {
    let Some(tracking) = git.status(repo)?.branch.upstream else {
        return Ok(None);
    };
    // upstream이 원격에서 사라졌으면(gone) 없는 것으로 본다.
    if git
        .resolve_ref(repo, &format!("refs/remotes/{tracking}"))?
        .is_none()
    {
        return Ok(None);
    }
    let remote = git
        .remotes(repo)?
        .into_iter()
        .map(|r| r.name)
        .filter(|name| tracking.starts_with(&format!("{name}/")))
        .max_by_key(String::len)
        .ok_or(CoreError::NoRemote)?;
    names::validate_ref_like(&remote)?;
    Ok(Some(Upstream { remote, tracking }))
}

/// upstream이 없을 때 쓸 원격: 하나뿐이면 그것, 여럿이면 origin.
fn pick_remote(git: &dyn GitBackend, repo: &RepoPath) -> Result<String> {
    if let Some(up) = upstream(git, repo)? {
        return Ok(up.remote);
    }
    let remotes = git.remotes(repo)?;
    let name = match remotes.as_slice() {
        [] => return Err(CoreError::NoRemote),
        [only] => only.name.clone(),
        many => many
            .iter()
            .find(|r| r.name == "origin")
            .map(|r| r.name.clone())
            .ok_or(CoreError::NoRemote)?,
    };
    names::validate_ref_like(&name)?;
    Ok(name)
}

fn fetch_remote(git: &dyn GitBackend, repo: &RepoPath, remote: &str) -> Result<()> {
    git.run(
        repo,
        &GitOp::Fetch {
            remote: Some(remote.to_string()),
            background: false,
            auth: AuthMode::UserDefault,
        },
    )?;
    Ok(())
}

fn no_upstream() -> git::GitError {
    git::GitError::Refused {
        kind: git::FailureKind::NoUpstream,
        detail: "no upstream".into(),
    }
}

/// 올린다. upstream이 없으면 원격을 골라 같은 이름으로 올리고 연결한다. 올린 버전 수.
fn push(git: &dyn GitBackend, repo: &RepoPath) -> Result<u32> {
    let branch = git.current_branch(repo)?.ok_or(CoreError::InvalidInput(
        "특정 버전을 보는 중에는 올릴 수 없어요",
    ))?;
    match upstream(git, repo)? {
        Some(up) => {
            let (ahead, _) = git.count_between(repo, "HEAD", &up.rev())?;
            if ahead > 0 {
                git.run(
                    repo,
                    &GitOp::Push {
                        target: Some(up.push_target(git)?),
                        set_upstream: false,
                        auth: AuthMode::UserDefault,
                    },
                )?;
            }
            Ok(ahead)
        }
        None => {
            let remote = pick_remote(git, repo)?;
            if !git.check_branch_name(&branch)? {
                return Err(CoreError::InvalidInput("올릴 수 없는 브랜치 이름이에요"));
            }
            git.run(
                repo,
                &GitOp::Push {
                    target: Some((remote, branch)),
                    set_upstream: true,
                    auth: AuthMode::UserDefault,
                },
            )?;
            let count = git
                .log(
                    repo,
                    &LogQuery {
                        skip: 0,
                        limit: 10_000,
                        rev: None,
                    },
                )?
                .len();
            Ok(u32::try_from(count).unwrap_or(u32::MAX))
        }
    }
}

fn sync(git: &dyn GitBackend, repo: &RepoPath, input: &SyncInput) -> Result<SyncReport> {
    if git.remotes(repo)?.is_empty() {
        return Err(CoreError::NoRemote);
    }
    let Some(up) = upstream(git, repo)? else {
        // 처음 올리는 브랜치: 받을 것이 없으니 올리고 연결만 한다.
        let sent = push(git, repo)?;
        return Ok(SyncReport {
            sent,
            ..SyncReport::default()
        });
    };
    // 합치는 중에는 동기화하지 않는다(충돌 해결 화면에서 마치거나 그만둔 뒤에).
    crate::actions::switch::refuse_during_operation(git, repo)?;
    let mut report = SyncReport::default();
    let rev = up.rev();
    // 그사이 누가 올려 거부되면 한 번만 처음부터 다시 한다.
    for attempt in 0..2 {
        fetch_remote(git, repo, &up.remote)?;
        let (ahead, behind) = git.count_between(repo, "HEAD", &rev)?;
        if behind > 0 {
            if ahead == 0 {
                git.run(repo, &GitOp::MergeFf { rev: rev.clone() })?;
            } else {
                // 넣은 파일이 있으면 합친 버전에 섞이지 않게 시작하지 않는다(D-069).
                crate::guard::refuse_staged(git, repo)?;
                if !merge(git, repo, &rev, behind, &input.locale)? {
                    // 겹쳤다: 합치는 중으로 두고 충돌 해결 화면으로 간다. 올리지 않는다.
                    report.received = 0;
                    report.merge_started = true;
                    report.overlap = crate::actions::merge::conflict_paths(git, repo)?;
                    return Ok(report);
                }
                report.merged = true;
            }
            report.received += behind;
        }
        let (ahead, _) = git.count_between(repo, "HEAD", &rev)?;
        if ahead == 0 {
            return Ok(report);
        }
        match git.run(
            repo,
            &GitOp::Push {
                target: Some(up.push_target(git)?),
                set_upstream: false,
                auth: AuthMode::UserDefault,
            },
        ) {
            Ok(_) => {
                report.sent = ahead;
                return Ok(report);
            }
            Err(e) if e.kind() == git::FailureKind::PushRejected && attempt == 0 => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(report)
}

/// 다른 사람 버전과 합친 버전을 만든다. 합쳤으면 참, 겹쳐서(또는 hook이 막아) 합치는 중으로
/// 멈췄으면 거짓. 그 밖의 실패는 오류다.
fn merge(
    git: &dyn GitBackend,
    repo: &RepoPath,
    rev: &str,
    count: u32,
    locale: &str,
) -> Result<bool> {
    if let Some(head) = git.resolve_ref(repo, "HEAD")? {
        backup::backup_oid(git, repo, &head, "sync")?;
    }
    let message = merge_message(&incoming_authors(git, repo, rev)?, count, locale);
    match git.run(
        repo,
        &GitOp::Merge {
            rev: rev.to_string(),
            message,
        },
    ) {
        Ok(_) => Ok(true),
        Err(_) if crate::actions::merge::merging(git, repo)? => Ok(false),
        Err(e) => Err(e.into()),
    }
}

fn incoming_authors(git: &dyn GitBackend, repo: &RepoPath, tracking: &str) -> Result<Vec<String>> {
    let commits = git.log(
        repo,
        &LogQuery {
            skip: 0,
            limit: 200,
            rev: Some(format!("HEAD..{tracking}")),
        },
    )?;
    let mut authors: Vec<String> = Vec::new();
    for c in commits {
        if !authors.contains(&c.author_name) {
            authors.push(c.author_name);
        }
    }
    Ok(authors)
}

/// "민지의 버전 1개 가져와 합침", "민지 외 1명의 버전 3개 가져와 합침"(앱 언어로).
pub(crate) fn merge_message(authors: &[String], count: u32, locale: &str) -> String {
    let first = authors.first().map(String::as_str).unwrap_or("");
    let others = authors.len().saturating_sub(1);
    if locale == "en" {
        match others {
            0 => format!("Merge {count} version(s) from {first}"),
            n => format!("Merge {count} versions from {first} and {n} more"),
        }
    } else {
        match others {
            0 => format!("{first}의 버전 {count}개 가져와 합침"),
            n => format!("{first} 외 {n}명의 버전 {count}개 가져와 합침"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_messages() {
        assert_eq!(
            merge_message(&["민지".into()], 1, "ko"),
            "민지의 버전 1개 가져와 합침"
        );
        assert_eq!(
            merge_message(&["민지".into(), "서연".into()], 3, "ko"),
            "민지 외 1명의 버전 3개 가져와 합침"
        );
        assert_eq!(
            merge_message(&["Kim".into()], 2, "en"),
            "Merge 2 version(s) from Kim"
        );
    }
}
