//! 검토 요청 쓰기와 포크(D-176, D-178~D-181). 본문은 `serde_json`으로만 만들고, 서버가 준 글은
//! 잘라서 글자로만 넘긴다. 서버가 준 URL은 읽지 않는다.

use serde::{Deserialize, Serialize};

use crate::error::{HostingError, HostingErrorKind, ValidationKind};
use crate::remote::{GITHUB_HOST, RemoteRepo, valid_name, valid_owner};

/// 댓글 하나의 바이트 상한(검토 요청 본문과 같음).
pub const COMMENT_MAX_BYTES: usize = 64 * 1024;
/// 화면에 넘기는 댓글 본문의 글자 수.
pub const COMMENT_SHOWN_MAX_CHARS: usize = 4000;
/// 댓글을 읽는 한 쪽의 수(D-178).
pub const COMMENTS_PAGE: u32 = 20;

/// 검토 요청의 대화 댓글 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// 탈퇴한 사용자면 `None`.
    pub author: Option<String>,
    /// unix 초.
    pub created_at: i64,
    /// 앞 [`COMMENT_SHOWN_MAX_CHARS`]자. 줄바꿈·탭 말고 제어 문자는 공백.
    pub body: String,
}

/// 합치는 방법(D-179). "다시 쌓아 합치기"(rebase)는 다루지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMethod {
    Merge,
    Squash,
}

impl MergeMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Squash => "squash",
        }
    }
}

/// 합치기 요청. `sha`가 서버의 head와 다르면 서버가 409로 거절한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeRequest {
    pub method: MergeMethod,
    pub sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResult {
    pub merged: bool,
}

#[derive(Deserialize)]
struct UserJson {
    login: String,
}

#[derive(Deserialize)]
struct CommentJson {
    user: Option<UserJson>,
    created_at: String,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Deserialize)]
struct CreatedJson {
    id: u64,
}

#[derive(Deserialize)]
struct MergedJson {
    #[serde(default)]
    merged: bool,
}

#[derive(Deserialize)]
struct ForkJson {
    name: String,
    owner: UserJson,
}

#[derive(Serialize)]
struct CommentBody<'a> {
    body: &'a str,
}

#[derive(Serialize)]
struct MergeBody<'a> {
    merge_method: &'a str,
    sha: &'a str,
}

#[derive(Serialize)]
struct StateBody {
    state: &'static str,
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

/// 검토 요청 번호는 1 이상.
pub(crate) fn check_number(number: u64) -> Result<(), HostingError> {
    if number == 0 {
        return Err(invalid("pull request number"));
    }
    Ok(())
}

/// 보내기 전 댓글 검사: 앞뒤 공백을 뗀 뒤 비지 않고 64KB 안, NUL 없음.
pub(crate) fn check_comment(body: &str) -> Result<(), HostingError> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.len() > COMMENT_MAX_BYTES || trimmed.contains('\0') {
        return Err(invalid("comment"));
    }
    Ok(())
}

pub(crate) fn comment_json(body: &str) -> Result<Vec<u8>, HostingError> {
    serde_json::to_vec(&CommentBody { body: body.trim() }).map_err(|_| invalid("comment"))
}

pub(crate) fn parse_created_id(body: &[u8]) -> Result<u64, HostingError> {
    serde_json::from_slice::<CreatedJson>(body)
        .map(|c| c.id)
        .map_err(|_| bad("comment is not JSON"))
}

pub(crate) fn parse_comments(body: &[u8]) -> Result<Vec<Comment>, HostingError> {
    let items: Vec<CommentJson> =
        serde_json::from_slice(body).map_err(|_| bad("comments are not JSON"))?;
    items
        .into_iter()
        .map(|c| {
            Ok(Comment {
                author: c.user.map(|u| u.login).filter(|l| valid_owner(l)),
                created_at: crate::pr::parse_time(&c.created_at)
                    .ok_or_else(|| bad("unexpected time"))?,
                body: shown_text(c.body.as_deref().unwrap_or_default()),
            })
        })
        .collect()
}

fn shown_text(text: &str) -> String {
    text.chars()
        .take(COMMENT_SHOWN_MAX_CHARS)
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                ' '
            } else {
                c
            }
        })
        .collect()
}

pub(crate) fn merge_json(req: &MergeRequest) -> Result<Vec<u8>, HostingError> {
    if !crate::pr::valid_sha(&req.sha) {
        return Err(invalid("head sha"));
    }
    serde_json::to_vec(&MergeBody {
        merge_method: req.method.as_str(),
        sha: &req.sha,
    })
    .map_err(|_| invalid("merge"))
}

pub(crate) fn parse_merged(body: &[u8]) -> Result<MergeResult, HostingError> {
    serde_json::from_slice::<MergedJson>(body)
        .map(|m| MergeResult { merged: m.merged })
        .map_err(|_| bad("merge result is not JSON"))
}

pub(crate) fn state_json(open: bool) -> Vec<u8> {
    // 고정 값 두 개라 실패하지 않지만, 실패하면 빈 객체(서버가 422로 거절).
    serde_json::to_vec(&StateBody {
        state: if open { "open" } else { "closed" },
    })
    .unwrap_or_else(|_| b"{}".to_vec())
}

/// 포크 응답(202). 소유자·이름 모양만 검사하고 주소 칸은 읽지 않는다.
pub(crate) fn parse_fork(body: &[u8]) -> Result<RemoteRepo, HostingError> {
    let parsed: ForkJson = serde_json::from_slice(body).map_err(|_| bad("fork is not JSON"))?;
    if !valid_owner(&parsed.owner.login) || !valid_name(&parsed.name) {
        return Err(bad("unexpected fork"));
    }
    Ok(RemoteRepo {
        host: GITHUB_HOST.to_owned(),
        owner: parsed.owner.login,
        name: parsed.name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_is_checked_before_sending() {
        check_comment("  좋아요 \u{1F600}\n").unwrap();
        check_comment(&"가".repeat(COMMENT_MAX_BYTES / 3)).unwrap();
        for bad in ["", " \n\t ", "a\0b"] {
            assert!(check_comment(bad).is_err(), "{bad:?}");
        }
        assert!(check_comment(&"a".repeat(COMMENT_MAX_BYTES + 1)).is_err());
        let json: serde_json::Value =
            serde_json::from_slice(&comment_json("  \"따옴표\" <b>\n둘째 ").unwrap()).unwrap();
        assert_eq!(json["body"], "\"따옴표\" <b>\n둘째");
        assert_eq!(json.as_object().unwrap().len(), 1);
    }

    #[test]
    fn comments_are_cut_and_cleaned() {
        let long = "가".repeat(COMMENT_SHOWN_MAX_CHARS + 10);
        let body = format!(
            r#"[
            {{"id":1,"user":{{"login":"mona"}},"created_at":"2026-10-01T00:00:00Z","body":"첫 줄\n둘째\u0007끝 <script>"}},
            {{"id":2,"user":null,"created_at":"2026-10-02T00:00:00Z","body":"{long}"}},
            {{"id":3,"user":{{"login":"../x"}},"created_at":"2026-10-03T00:00:00Z","body":null}}
        ]"#
        );
        let list = parse_comments(body.as_bytes()).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].author.as_deref(), Some("mona"));
        assert_eq!(list[0].body, "첫 줄\n둘째 끝 <script>");
        assert_eq!(list[1].author, None);
        assert_eq!(list[1].body.chars().count(), COMMENT_SHOWN_MAX_CHARS);
        assert_eq!(list[2].author, None);
        assert_eq!(list[2].body, "");
        assert!(parse_comments(br#"[{"user":null,"created_at":"x","body":""}]"#).is_err());
        assert!(parse_comments(b"nope").is_err());
    }

    #[test]
    fn merge_body_needs_a_full_sha() {
        let sha = "a".repeat(40);
        let json: serde_json::Value = serde_json::from_slice(
            &merge_json(&MergeRequest {
                method: MergeMethod::Squash,
                sha: sha.clone(),
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(json["merge_method"], "squash");
        assert_eq!(json["sha"], sha.as_str());
        for bad in ["abc", &"g".repeat(40), &"a".repeat(41), ""] {
            assert!(
                merge_json(&MergeRequest {
                    method: MergeMethod::Merge,
                    sha: bad.to_string(),
                })
                .is_err()
            );
        }
        assert!(
            parse_merged(br#"{"sha":"x","merged":true,"message":"ok"}"#)
                .unwrap()
                .merged
        );
        assert!(!parse_merged(br#"{"message":"x"}"#).unwrap().merged);
    }

    #[test]
    fn state_and_fork() {
        assert_eq!(state_json(false), br#"{"state":"closed"}"#.to_vec());
        assert_eq!(state_json(true), br#"{"state":"open"}"#.to_vec());
        let fork = parse_fork(
            br#"{"name":"hello","owner":{"login":"mona"},"clone_url":"https://evil.example/x.git"}"#,
        )
        .unwrap();
        assert_eq!((fork.owner.as_str(), fork.name.as_str()), ("mona", "hello"));
        assert!(fork.is_github());
        for body in [
            r#"{"name":"..","owner":{"login":"mona"}}"#,
            r#"{"name":"x","owner":{"login":"a b"}}"#,
            "nope",
        ] {
            assert_eq!(
                parse_fork(body.as_bytes()).unwrap_err().kind(),
                HostingErrorKind::BadResponse
            );
        }
        assert!(check_number(0).is_err());
        check_number(1).unwrap();
    }
}
