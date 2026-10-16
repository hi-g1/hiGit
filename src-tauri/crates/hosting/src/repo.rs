//! 저장소 만들기·목록(D-132, D-133). 본문은 `serde_json`으로만 만들고 서버가 준 값은 모양을 확인한
//! 뒤에만 쓴다. 서버가 준 URL(`clone_url` 등)은 읽지 않는다.

use serde::{Deserialize, Serialize};

use crate::error::{HostingError, HostingErrorKind, ValidationKind};
use crate::remote::{valid_name, valid_owner};

/// 저장소 설명의 바이트 상한(GitHub 화면의 한도 안).
pub const REPO_DESCRIPTION_MAX_BYTES: usize = 350;
/// 저장소 목록 한 쪽의 최대 수.
pub const REPO_PAGE_MAX: u32 = 50;

/// 새 저장소(빈 저장소, 초기 파일 없음).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRepo {
    pub name: String,
    pub description: String,
    pub private: bool,
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
}

#[derive(Deserialize)]
struct RepoJson {
    name: String,
    owner: OwnerJson,
    #[serde(default)]
    private: bool,
    pushed_at: Option<String>,
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

/// 목록. 소유자·이름 모양이 틀린 항목은 뺀다.
pub(crate) fn parse_list(body: &[u8]) -> Result<Vec<RepoSummary>, HostingError> {
    let parsed: Vec<RepoJson> =
        serde_json::from_slice(body).map_err(|_| bad("repositories are not JSON"))?;
    Ok(parsed
        .into_iter()
        .filter(|r| valid_owner(&r.owner.login) && valid_name(&r.name))
        .map(|r| RepoSummary {
            pushed_at: r.pushed_at.as_deref().and_then(crate::pr::parse_time),
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
    fn list_skips_odd_items_and_reads_times() {
        let body = br#"[
            {"name":"a","owner":{"login":"me"},"private":false,"pushed_at":"2026-10-02T00:00:00Z","description":null},
            {"name":"b","owner":{"login":"org-1"},"private":true,"pushed_at":null},
            {"name":"../c","owner":{"login":"me"},"private":false,"pushed_at":null}
        ]"#;
        let list = parse_list(body).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(
            list[0].pushed_at,
            crate::pr::parse_time("2026-10-02T00:00:00Z")
        );
        assert!(list[1].private && list[1].pushed_at.is_none());
    }
}
