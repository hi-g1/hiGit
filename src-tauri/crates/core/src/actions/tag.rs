//! 태그 만들기·지우기(`tag`·`tag-delete`, D-104). 가벼운 태그만 다룬다. 둘 다 HEAD를 바꾸지 않고
//! `refs/tags/<이름>`을 journal에 추적해 Ctrl+Z가 태그를 지우거나 되살린다.

use git::{GitOp, Oid};

use super::repo_of;
use crate::guard::{self, Scope};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

pub struct TagAction {
    meta: ActionMeta,
}

impl TagAction {
    pub fn create() -> Self {
        Self::with_id("tag")
    }

    pub fn delete() -> Self {
        Self::with_id("tag-delete")
    }

    fn with_id(id: &'static str) -> Self {
        Self {
            meta: ActionMeta {
                id,
                glossary_id: "tag",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::RestoreRef,
            },
        }
    }
}

/// 태그 이름 검사(D-104): 앞뒤 공백을 뗀 뒤 255바이트 이하, `-`로 시작하지 않고 git의 이름 규칙에 맞음.
pub(crate) fn tag_name(git: &dyn git::GitBackend, raw: &str) -> Result<String> {
    let name = raw.trim();
    crate::actions::names::validate_ref_like(name)?;
    if !git.check_tag_name(name)? {
        return Err(CoreError::InvalidInput(
            "태그 이름으로 쓸 수 없는 글자가 있어요",
        ));
    }
    Ok(name.to_string())
}

impl Action for TagAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn tracked_refs(&self, input: &ActionInput) -> Vec<String> {
        let name = match input {
            ActionInput::Tag(i) => i.name.trim(),
            ActionInput::TagDelete(i) => i.name.trim(),
            _ => return Vec::new(),
        };
        // 이름 검사는 실행 때 한다. 여기서는 journal에 담을 ref 이름만 만든다(이상한 이름은 줄의
        // `is_valid`가 거른다).
        vec![format!("refs/tags/{name}")]
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let repo = repo_of(ctx)?;
        let git = ctx.git;
        let existing = git.list_refs(repo, "refs/tags/")?;
        match (self.meta.id, input) {
            ("tag", ActionInput::Tag(i)) => {
                let name = tag_name(git, &i.name)?;
                // Windows처럼 대소문자를 가리지 않는 파일 시스템에서는 `V1`과 `v1`이 같은 파일이다.
                let lower = name.to_lowercase();
                if existing.iter().any(|r| {
                    r.name.strip_prefix("refs/tags/").map(str::to_lowercase) == Some(lower.clone())
                }) {
                    return Err(CoreError::TagExists);
                }
                let oid = guard::verified_commit(git, repo, &i.oid, Scope::BranchOrActivity)?;
                git.run(
                    repo,
                    &GitOp::TagCreate {
                        name: name.clone(),
                        oid,
                    },
                )?;
                Ok(Outcome::Tagged { name })
            }
            ("tag-delete", ActionInput::TagDelete(i)) => {
                let name = tag_name(git, &i.name)?;
                let full = format!("refs/tags/{name}");
                let entry = existing
                    .iter()
                    .find(|r| r.name == full)
                    .ok_or(CoreError::InvalidInput("그 태그가 없어요"))?;
                // 화면이 본 버전을 그대로 가리킬 때만 지운다.
                let expect =
                    Oid::parse(&i.oid).ok_or(CoreError::InvalidInput("버전이 해시가 아니에요"))?;
                if git.resolve_ref(repo, &full)?.as_ref() != Some(&expect) {
                    return Err(CoreError::InvalidInput("태그가 다른 버전으로 옮겨졌어요"));
                }
                // 주석 태그(태그 객체)는 지우면 객체가 정리되어 Ctrl+Z가 늦게 실패할 수 있다.
                if entry.oid != expect.as_str() {
                    return Err(CoreError::AnnotatedTag);
                }
                git.run(repo, &GitOp::TagDelete { name: name.clone() })?;
                Ok(Outcome::TagDeleted { name })
            }
            _ => Err(CoreError::InputMismatch(self.meta.id)),
        }
    }
}
