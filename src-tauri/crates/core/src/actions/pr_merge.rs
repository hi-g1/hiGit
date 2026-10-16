//! 검토 요청 합치기(pr-merge, D-179). 원격 기본 브랜치가 바뀌고 앱이 되돌릴 수 없어 `Irreversible`이고
//! 실행기가 확인 토큰을 요구한다. 화면이 보여 준 head(`expect_head`)일 때만 합치도록 GitHub에 `sha`를 싣는다.

use crate::view::{ActionInput, MergeMethodView};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, GithubFailure, Outcome, RepoState,
    Result, Risk,
};

use super::{needs_repo, repo_of};

pub struct PrMergeAction {
    meta: ActionMeta,
}

impl PrMergeAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "pr-merge",
                glossary_id: "pr-merge",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Irreversible,
            },
        }
    }
}

impl Default for PrMergeAction {
    fn default() -> Self {
        Self::new()
    }
}

fn failure(kind: GithubFailure) -> CoreError {
    CoreError::Github { kind, detail: None }
}

fn valid_sha(sha: &str) -> bool {
    (sha.len() == 40 || sha.len() == 64) && sha.bytes().all(|b| b.is_ascii_hexdigit())
}

impl Action for PrMergeAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::PrMerge(i) = input else {
            return Err(CoreError::InputMismatch("pr-merge"));
        };
        if i.number == 0 {
            return Err(CoreError::InvalidInput("검토 요청 번호"));
        }
        if !valid_sha(&i.expect_head) {
            return Err(CoreError::InvalidInput("버전"));
        }
        let repo = repo_of(ctx)?;
        let github_repo = crate::github::pr::github_repo(ctx.git, repo)?;
        // 방법은 화면과 같은 규칙으로 저장소가 허용한 것만(D-179).
        let info = ctx
            .github
            .call(|provider, token| provider.repo_info(token, &github_repo))?;
        let allowed = crate::github::pr::repo_view(&info, None).merge_methods;
        if !allowed.contains(&i.method) {
            return Err(failure(GithubFailure::PrNotMergeable));
        }
        let request = hosting::MergeRequest {
            method: match i.method {
                MergeMethodView::Merge => hosting::MergeMethod::Merge,
                MergeMethodView::Squash => hosting::MergeMethod::Squash,
            },
            sha: i.expect_head.to_ascii_lowercase(),
        };
        let merged = ctx.github.call(|provider, token| {
            Ok(
                match provider.merge_pr(token, &github_repo, u64::from(i.number), &request) {
                    Ok(result) => Ok(result),
                    // 합칠 수 없음·그사이 head가 바뀜은 맥락이 있는 문구로(D-176).
                    Err(e) if e.kind() == hosting::HostingErrorKind::NotAllowed => {
                        Err(GithubFailure::PrNotMergeable)
                    }
                    Err(e) if e.kind() == hosting::HostingErrorKind::Conflict => {
                        Err(GithubFailure::PrHeadChanged)
                    }
                    Err(e) => return Err(e),
                },
            )
        })?;
        match merged {
            Ok(result) if result.merged => Ok(Outcome::PrMerged { number: i.number }),
            Ok(_) => Err(failure(GithubFailure::PrNotMergeable)),
            Err(kind) => Err(failure(kind)),
        }
    }
}
