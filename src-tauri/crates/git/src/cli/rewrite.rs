//! 기록 정리(`rebase -i`, D-100). 할 일 목록은 시작할 때 한 번 상수 편집기 명령과 환경 변수로
//! 넘긴다. 목록에는 해시와 `pick`·`fixup`·`edit`·`drop`만 들고 `exec` 줄은 없다. 메시지 고치기는
//! `edit`으로 멈추면 core가 `Amend`(stdin)로 한 뒤 이어간다.

use std::ffi::OsStr;

use super::CliBackend;
use super::runner::EnvChange;
use crate::{GitError, Oid, OpOutput, RebaseState, RepoPath, Result, RewriteKind, RewriteStep};

/// git이 셸로 부르는 편집기 명령. 사용자 데이터를 담지 않는 상수다. 값(목록)은 환경 변수에 있고
/// 셸은 그 값을 다시 읽지 않는다.
const SEQUENCE_EDITOR: &str = "printf %s \"$HIGIT_TODO\" >";
/// 한 번에 정리하는 버전 수 상한.
pub const MAX_STEPS: usize = 50;

/// 할 일 목록 글자. 첫 줄이 `fixup`이거나 `drop` 바로 뒤에 `fixup`이 오면(묶을 대상이 없음) 오류.
pub(crate) fn todo(steps: &[RewriteStep]) -> Result<String> {
    if steps.is_empty() || steps.len() > MAX_STEPS {
        return Err(GitError::Parse(format!("정리할 버전 수: {}", steps.len())));
    }
    let mut out = String::new();
    let mut prev: Option<RewriteKind> = None;
    for step in steps {
        if step.kind == RewriteKind::Fixup && matches!(prev, None | Some(RewriteKind::Drop)) {
            return Err(GitError::Parse("묶을 버전이 없는 fixup".into()));
        }
        let word = match step.kind {
            RewriteKind::Pick => "pick",
            RewriteKind::Fixup => "fixup",
            RewriteKind::Edit => "edit",
            RewriteKind::Drop => "drop",
        };
        out.push_str(&format!("{word} {}\n", step.oid));
        prev = Some(step.kind);
    }
    Ok(out)
}

/// 이어가기·시작 모두에 주는 설정: 사용자 설정이 목록 모양이나 동작을 바꾸지 않게 고정한다.
pub(crate) const REBASE_CONFIG: &[&str] = &[
    "-c",
    "rebase.abbreviateCommands=false",
    "-c",
    "rebase.autoSquash=false",
    "-c",
    "rebase.missingCommitsCheck=ignore",
    "-c",
    "rebase.autoStash=false",
    "-c",
    "rebase.updateRefs=false",
];

impl CliBackend {
    /// 기록 정리를 시작한다. 끝까지 가면 `rebase-merge/`가 없고, `edit`이나 충돌·hook으로 멈추면
    /// 남는다(core가 `rebase_state`로 본다). 시작하지 못한 오류는 그대로 돌려준다.
    pub(super) fn rewrite_history(
        &self,
        repo: &RepoPath,
        base: &Oid,
        steps: &[RewriteStep],
    ) -> Result<OpOutput> {
        let list = todo(steps)?;
        let mut args: Vec<&str> = REBASE_CONFIG.to_vec();
        args.extend(["rebase", "-i", "--empty=drop", base.as_str()]);
        let out = self.run_local_watched_env(
            repo,
            &args,
            &[
                (
                    "GIT_SEQUENCE_EDITOR",
                    EnvChange::Set(OsStr::new(SEQUENCE_EDITOR)),
                ),
                ("HIGIT_TODO", EnvChange::Set(OsStr::new(&list))),
            ],
            None,
        )?;
        Ok(OpOutput {
            stdout: out.stdout_lossy(),
            stderr: out.stderr_lossy(),
            oid: None,
        })
    }

    /// 멈춘 기록 정리의 상태(`rebase-merge/`). 기록 정리 중이 아니면 `None`.
    pub(super) fn rebase_state_impl(&self, repo: &RepoPath) -> Result<Option<RebaseState>> {
        let dir = crate::GitBackend::git_dir(self, repo)?.join("rebase-merge");
        if !dir.is_dir() {
            return Ok(None);
        }
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).unwrap_or_default();
        let lines = |text: String| -> Vec<String> {
            text.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(str::to_string)
                .collect()
        };
        Ok(Some(RebaseState {
            head_name: read("head-name").trim().to_string(),
            onto: Oid::parse(read("onto").trim()),
            orig_head: Oid::parse(read("orig-head").trim()),
            stopped_at_edit: dir.join("amend").is_file(),
            todo: lines(read("git-rebase-todo")),
            done: lines(read("done")),
        }))
    }

    /// 버전에 남길 이름·이메일이 정해져 있는지(`git var GIT_COMMITTER_IDENT`).
    pub(super) fn has_identity_impl(&self, repo: &RepoPath) -> Result<bool> {
        match self
            .runner
            .run(Some(repo.as_path()), &["var", "GIT_COMMITTER_IDENT"])
        {
            Ok(_) => Ok(true),
            Err(GitError::Failed { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
