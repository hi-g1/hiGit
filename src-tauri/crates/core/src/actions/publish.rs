//! GitHub에 올리기(`publish`, D-132). 내 계정에 새 저장소를 만들고 `origin`을 잇는 데까지 한다. 올리기는
//! 화면이 이어서 기존 `push`를 부른다(위험도·진행·취소를 그대로 쓴다). `origin` 주소는 서버가 준 글자가
//! 아니라 검사한 소유자·이름으로 core가 만든다. 만든 뒤 잇기가 실패해도 저장소는 지우지 않는다(권한 없음).

use git::{GitOp, LogQuery};

use super::{needs_repo, repo_of};
use crate::view::{ActionInput, Visibility};
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, GithubFailure, Outcome, RepoState,
    Result, Risk,
};

pub struct PublishAction {
    meta: ActionMeta,
}

impl PublishAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "publish",
                glossary_id: "publish",
                risk: Risk::Normal,
                undo: crate::undo::UndoKind::Nothing,
            },
        }
    }
}

impl Default for PublishAction {
    fn default() -> Self {
        Self::new()
    }
}

/// 저장소 이름 제안: `[A-Za-z0-9._-]` 밖은 `-`로, 앞뒤 `-`·`.`를 떼고 100자까지. 못 만들면 빈 값.
pub(crate) fn suggest_name(folder: &str) -> String {
    let mapped: String = folder
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = mapped.trim_matches(|c| c == '-' || c == '.');
    let cut: String = trimmed.chars().take(100).collect();
    let cut = cut.trim_end_matches(['-', '.']).to_string();
    if hosting::valid_name(&cut) {
        cut
    } else {
        String::new()
    }
}

impl Action for PublishAction {
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
        let ActionInput::Publish(i) = input else {
            return Err(CoreError::InputMismatch("publish"));
        };
        let git = ctx.git;
        let repo = repo_of(ctx)?;
        crate::guard::require_branch(git, repo)?;
        crate::actions::switch::refuse_during_operation(git, repo)?;
        let has_version = !git
            .log(
                repo,
                &LogQuery {
                    skip: 0,
                    limit: 1,
                    rev: None,
                },
            )?
            .is_empty();
        if !has_version {
            return Err(CoreError::PublishNoVersions);
        }
        if !git.remotes(repo)?.is_empty() {
            return Err(CoreError::PublishHasRemote);
        }
        let name = i.name.trim().to_string();
        if !hosting::valid_name(&name) {
            return Err(CoreError::InvalidInput("저장소 이름"));
        }
        let description = i.description.as_deref().unwrap_or("").trim().to_string();
        if description.len() > hosting::REPO_DESCRIPTION_MAX_BYTES
            || description.chars().any(char::is_control)
        {
            return Err(CoreError::InvalidInput("저장소 설명"));
        }
        // 공개는 대화상자 안에서 한 번 더 확인한 경우만 받는다.
        if i.visibility == Visibility::Public && !i.acknowledge_public {
            return Err(CoreError::InvalidInput(
                "공개 저장소는 한 번 더 확인해 주세요",
            ));
        }
        // 만들 곳: 없으면 내 계정, 있으면 조직(D-184). 이름은 보내기 전에 검사한다.
        let org = match i.owner.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(org) if hosting::valid_owner(org) => Some(org.to_string()),
            Some(_) => return Err(CoreError::InvalidInput("조직 이름")),
        };
        let status = ctx.github.status();
        let login = match (status.connected, status.login) {
            (true, Some(login)) => login,
            _ => {
                return Err(CoreError::Github {
                    kind: GithubFailure::NotConnected,
                    detail: None,
                });
            }
        };
        let new = hosting::NewRepo {
            name,
            description,
            private: i.visibility == Visibility::Private,
            owner: org
                .clone()
                .map_or(hosting::RepoOwner::Me, hosting::RepoOwner::Org),
        };
        let created = ctx
            .github
            .call(|provider, token| provider.create_repo(token, &new))
            .map_err(|e| match e {
                CoreError::Github {
                    kind: GithubFailure::PrAlreadyExists,
                    detail,
                } => CoreError::Github {
                    kind: GithubFailure::RepoExists,
                    detail,
                },
                CoreError::Github {
                    kind: GithubFailure::PrInvalid,
                    detail,
                } => CoreError::Github {
                    kind: GithubFailure::RepoInvalid,
                    detail,
                },
                // 조직이 앱을 막았거나 만들 권한이 없다.
                CoreError::Github {
                    kind: GithubFailure::Forbidden,
                    detail,
                } if org.is_some() => CoreError::Github {
                    kind: GithubFailure::OrgForbidden,
                    detail,
                },
                other => other,
            })?;
        // 고른 곳(내 계정이나 조직)의 저장소여야 한다. 주소는 검사한 소유자·이름으로만 만든다.
        let expected_owner = org.as_deref().unwrap_or(&login);
        if !created.owner.eq_ignore_ascii_case(expected_owner) {
            return Err(CoreError::Github {
                kind: GithubFailure::BadResponse,
                detail: Some("unexpected owner".into()),
            });
        }
        let url = format!("https://github.com/{}/{}.git", created.owner, created.name);
        let parsed = hosting::GitHub::detect(&url);
        if parsed.as_ref().map(|r| (r.owner.as_str(), r.name.as_str()))
            != Some((created.owner.as_str(), created.name.as_str()))
        {
            return Err(CoreError::Github {
                kind: GithubFailure::BadResponse,
                detail: Some("unexpected repository".into()),
            });
        }
        git.run(
            repo,
            &GitOp::RemoteAdd {
                name: "origin".into(),
                url,
            },
        )?;
        Ok(Outcome::Published {
            owner: created.owner,
            name: created.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_suggested_from_the_folder() {
        for (folder, want) in [
            ("todo-app", "todo-app"),
            ("내 프로젝트", ""),
            ("my project 2", "my-project-2"),
            ("..hidden.", "hidden"),
            ("한글 notes", "notes"),
            ("a_b.c", "a_b.c"),
        ] {
            assert_eq!(suggest_name(folder), want, "{folder}");
        }
        assert_eq!(suggest_name(&"x".repeat(150)).len(), 100);
    }
}
