//! 저장소 만들기·목록(D-132, D-133). 본문은 `serde_json`으로만 만들고 서버가 준 값은 모양을 확인한
//! 뒤에만 쓴다. 서버가 준 URL(`clone_url` 등)은 읽지 않는다.

use serde::{Deserialize, Serialize};

use crate::error::{HostingError, HostingErrorKind, ValidationKind};
use crate::remote::{valid_name, valid_owner};

/// 저장소 설명의 바이트 상한(GitHub 화면의 한도 안).
pub const REPO_DESCRIPTION_MAX_BYTES: usize = 350;
/// 저장소 목록 한 쪽의 최대 수.
pub const REPO_PAGE_MAX: u32 = 50;

/// 저장소를 만들 곳(D-184).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoOwner {
    /// 토큰 주인의 계정.
    Me,
    /// 조직(이름은 `valid_owner`).
    Org(String),
}

/// 새 저장소(빈 저장소, 초기 파일 없음).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRepo {
    pub name: String,
    pub description: String,
    pub private: bool,
    pub owner: RepoOwner,
}

/// 만든 저장소(소유자·이름만).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedRepo {
    pub owner: String,
    pub name: String,
}

/// 목록의 저장소 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSummary {
    pub owner: String,
    pub name: String,
    pub private: bool,
    /// 마지막으로 올린 시각(unix 초). 없으면 `None`.
    pub pushed_at: Option<i64>,
    /// 소유자가 조직인지(`owner.type == "Organization"`, D-184).
    pub owner_is_org: bool,
}

/// 저장소 정보(D-177). 합치기 방법 칸이 응답에 없으면 `None`(쓰기 권한이 없으면 빠질 수 있다).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoInfo {
    /// 토큰 주인이 올릴 수 있는지(`permissions.push`, 없으면 거짓).
    pub can_push: bool,
    pub archived: bool,
    pub fork: bool,
    pub allow_merge_commit: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    /// 포크면 원래 저장소(소유자·이름을 검사한 것).
    pub parent: Option<crate::remote::RemoteRepo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPage {
    pub items: Vec<RepoSummary>,
    pub has_more: bool,
}

#[derive(Serialize)]
struct NewRepoJson<'a> {
    name: &'a str,
    description: &'a str,
    private: bool,
    auto_init: bool,
}

#[derive(Deserialize)]
struct OwnerJson {
    login: String,
    #[serde(rename = "type", default)]
    kind: Option<String>,
}

#[derive(Deserialize)]
struct RepoJson {
    name: String,
    owner: OwnerJson,
    #[serde(default)]
    private: bool,
    pushed_at: Option<String>,
}

#[derive(Deserialize)]
struct PermissionsJson {
    #[serde(default)]
    push: bool,
}

#[derive(Deserialize)]
struct ParentJson {
    name: String,
    owner: OwnerJson,
}

#[derive(Deserialize)]
struct InfoJson {
    #[serde(default)]
    archived: bool,
    #[serde(default)]
    fork: bool,
    permissions: Option<PermissionsJson>,
    allow_merge_commit: Option<bool>,
    allow_squash_merge: Option<bool>,
    allow_rebase_merge: Option<bool>,
    parent: Option<ParentJson>,
}

#[derive(Deserialize)]
struct BranchJson {
    protected: Option<bool>,
}

fn invalid(detail: &str) -> HostingError {
    HostingError::with_detail(
        HostingErrorKind::Validation(ValidationKind::Invalid),
        detail,
    )
}

fn bad(detail: &str) -> HostingError {
    HostingError::with_detail(HostingErrorKind::BadResponse, detail)
}

/// 보내기 전 입력 검사. 틀리면 보내지 않는다.
pub(crate) fn check_new_repo(new: &NewRepo) -> Result<(), HostingError> {
    if !valid_name(&new.name) {
        return Err(invalid("repository name"));
    }
    if let RepoOwner::Org(org) = &new.owner
        && !valid_owner(org)
    {
        return Err(invalid("organization name"));
    }
    if new.description.len() > REPO_DESCRIPTION_MAX_BYTES
        || new.description.chars().any(char::is_control)
    {
        return Err(invalid("repository description"));
    }
    Ok(())
}

/// `POST /user/repos` 본문. 템플릿·기능 칸은 보내지 않고 초기 파일도 만들지 않는다.
pub(crate) fn new_repo_json(new: &NewRepo) -> Result<Vec<u8>, HostingError> {
    serde_json::to_vec(&NewRepoJson {
        name: &new.name,
        description: &new.description,
        private: new.private,
        auto_init: false,
    })
    .map_err(|_| invalid("repository"))
}

pub(crate) fn parse_created(body: &[u8]) -> Result<CreatedRepo, HostingError> {
    let parsed: RepoJson =
        serde_json::from_slice(body).map_err(|_| bad("repository is not JSON"))?;
    if !valid_owner(&parsed.owner.login) || !valid_name(&parsed.name) {
        return Err(bad("unexpected repository"));
    }
    Ok(CreatedRepo {
        owner: parsed.owner.login,
        name: parsed.name,
    })
}

/// `GET repos/{o}/{r}`. 원래 저장소의 소유자·이름이 이상하면 `BadResponse`.
pub(crate) fn parse_info(body: &[u8]) -> Result<RepoInfo, HostingError> {
    let parsed: InfoJson =
        serde_json::from_slice(body).map_err(|_| bad("repository is not JSON"))?;
    let parent = match parsed.parent {
        Some(p) if valid_owner(&p.owner.login) && valid_name(&p.name) => {
            Some(crate::remote::RemoteRepo {
                host: crate::remote::GITHUB_HOST.to_owned(),
                owner: p.owner.login,
                name: p.name,
            })
        }
        Some(_) => return Err(bad("unexpected parent repository")),
        None => None,
    };
    Ok(RepoInfo {
        can_push: parsed.permissions.is_some_and(|p| p.push),
        archived: parsed.archived,
        fork: parsed.fork,
        allow_merge_commit: parsed.allow_merge_commit,
        allow_squash_merge: parsed.allow_squash_merge,
        allow_rebase_merge: parsed.allow_rebase_merge,
        parent,
    })
}

/// `GET repos/{o}/{r}/branches/{b}`의 `protected`. 읽지 못하면 `None`.
pub(crate) fn parse_protected(body: &[u8]) -> Option<bool> {
    serde_json::from_slice::<BranchJson>(body).ok()?.protected
}

/// 목록. 소유자·이름 모양이 틀린 항목은 뺀다.
pub(crate) fn parse_list(body: &[u8]) -> Result<Vec<RepoSummary>, HostingError> {
    let parsed: Vec<RepoJson> =
        serde_json::from_slice(body).map_err(|_| bad("repositories are not JSON"))?;
    Ok(parsed
        .into_iter()
        .filter(|r| valid_owner(&r.owner.login) && valid_name(&r.name))
        .map(|r| RepoSummary {
            pushed_at: r.pushed_at.as_deref().and_then(crate::pr::parse_time),
            owner_is_org: r.owner.kind.as_deref() == Some("Organization"),
            owner: r.owner.login,
            name: r.name,
            private: r.private,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new(name: &str, description: &str) -> NewRepo {
        NewRepo {
            name: name.into(),
            description: description.into(),
            private: true,
            owner: RepoOwner::Me,
        }
    }

    #[test]
    fn body_is_json_without_init_files() {
        let body = new_repo_json(&new("hello", "따옴표 \" 와 \\ 줄")).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["name"], "hello");
        assert_eq!(value["description"], "따옴표 \" 와 \\ 줄");
        assert_eq!(value["private"], true);
        assert_eq!(value["auto_init"], false);
        assert_eq!(value.as_object().unwrap().len(), 4);
    }

    #[test]
    fn input_is_checked_before_sending() {
        check_new_repo(&new("hello-world.1", &"가".repeat(116))).unwrap();
        for bad in [
            new("", ""),
            new("../x", ""),
            new("a b", ""),
            new("ok", &"a".repeat(351)),
            new("ok", "줄\u{0}"),
            new("ok", "줄\n바꿈"),
        ] {
            assert!(check_new_repo(&bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn created_owner_and_name_are_checked() {
        let ok = parse_created(br#"{"name":"hello","owner":{"login":"octocat"},"private":true,"clone_url":"https://evil"}"#).unwrap();
        assert_eq!(
            ok,
            CreatedRepo {
                owner: "octocat".into(),
                name: "hello".into()
            }
        );
        let long = format!(r#"{{"name":"x","owner":{{"login":"{}"}}}}"#, "a".repeat(40));
        for body in [
            r#"{"name":"hello","owner":{"login":"../x"}}"#.to_string(),
            r#"{"name":"hello","owner":{"login":""}}"#.to_string(),
            r#"{"name":"..","owner":{"login":"octocat"}}"#.to_string(),
            long,
            "not json".to_string(),
        ] {
            assert_eq!(
                parse_created(body.as_bytes()).unwrap_err().kind(),
                HostingErrorKind::BadResponse,
                "{body}"
            );
        }
    }

    #[test]
    fn info_reads_permissions_methods_and_parent() {
        let mine = parse_info(
            br#"{"name":"a","owner":{"login":"me"},"archived":false,"fork":false,
            "permissions":{"admin":true,"push":true,"pull":true},
            "allow_merge_commit":true,"allow_squash_merge":false,"allow_rebase_merge":true}"#,
        )
        .unwrap();
        assert!(mine.can_push && !mine.fork && mine.parent.is_none());
        assert_eq!(
            (
                mine.allow_merge_commit,
                mine.allow_squash_merge,
                mine.allow_rebase_merge
            ),
            (Some(true), Some(false), Some(true))
        );
        // 쓰기 권한이 없으면 방법 칸이 빠질 수 있다.
        let theirs =
            parse_info(br#"{"name":"a","owner":{"login":"o"},"permissions":{"push":false}}"#)
                .unwrap();
        assert!(!theirs.can_push && theirs.allow_merge_commit.is_none());
        let no_perm = parse_info(br#"{"name":"a","owner":{"login":"o"},"archived":true}"#).unwrap();
        assert!(!no_perm.can_push && no_perm.archived);
        let fork = parse_info(
            br#"{"fork":true,"parent":{"name":"hello","owner":{"login":"octo"},"clone_url":"https://evil"}}"#,
        )
        .unwrap();
        let parent = fork.parent.unwrap();
        assert_eq!(
            (parent.owner.as_str(), parent.name.as_str()),
            ("octo", "hello")
        );
        assert!(parent.is_github());
        for body in [
            r#"{"parent":{"name":"..","owner":{"login":"octo"}}}"#,
            r#"{"parent":{"name":"x","owner":{"login":"-bad"}}}"#,
            "not json",
        ] {
            assert_eq!(
                parse_info(body.as_bytes()).unwrap_err().kind(),
                HostingErrorKind::BadResponse,
                "{body}"
            );
        }
    }

    #[test]
    fn protected_flag() {
        assert_eq!(
            parse_protected(br#"{"name":"main","protected":true}"#),
            Some(true)
        );
        assert_eq!(
            parse_protected(br#"{"name":"main","protected":false}"#),
            Some(false)
        );
        assert_eq!(parse_protected(br#"{"name":"main"}"#), None);
        assert_eq!(parse_protected(b"nope"), None);
    }

    #[test]
    fn list_skips_odd_items_and_reads_times() {
        let body = br#"[
            {"name":"a","owner":{"login":"me"},"private":false,"pushed_at":"2026-10-02T00:00:00Z","description":null},
            {"name":"b","owner":{"login":"org-1","type":"Organization"},"private":true,"pushed_at":null},
            {"name":"../c","owner":{"login":"me"},"private":false,"pushed_at":null}
        ]"#;
        let list = parse_list(body).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(
            list[0].pushed_at,
            crate::pr::parse_time("2026-10-02T00:00:00Z")
        );
        assert!(list[1].private && list[1].pushed_at.is_none());
        assert!(!list[0].owner_is_org && list[1].owner_is_org);
    }
}
