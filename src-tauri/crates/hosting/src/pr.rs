//! 검토 요청(pull request) 목록·상세·만들기의 타입과 응답 읽기.
//!
//! 사용자 글자는 serde로만 JSON을 만들고, URL 조각은 검사한 값을 퍼센트 인코딩해 붙인다.

use serde::{Deserialize, Serialize};

use crate::error::{HostingError, HostingErrorKind, ValidationKind};

/// 한 쪽에 받는 최대 개수.
pub const PR_PAGE_MAX: u32 = 50;
/// 제목 최대 바이트.
pub const PR_TITLE_MAX_BYTES: usize = 256;
/// 본문 최대 바이트.
pub const PR_BODY_MAX_BYTES: usize = 64 * 1024;
/// 브랜치 이름 최대 바이트.
pub const BRANCH_MAX_BYTES: usize = 255;
/// 화면에 넘기는 제목 최대 글자 수(서버가 준 값).
const TITLE_SHOWN_MAX_CHARS: usize = 1024;
/// check-runs 한 번에 받는 개수.
pub(crate) const CHECK_RUNS_PAGE: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrStateFilter {
    Open,
    Closed,
    All,
}

impl PrStateFilter {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrQuery {
    pub state: PrStateFilter,
    /// 이 브랜치에서 낸 것만.
    pub head_branch: Option<String>,
    /// `head_branch`의 소유자(포크에서 낸 것, D-181). 없으면 이 저장소의 소유자.
    pub head_owner: Option<String>,
    /// 1부터.
    pub page: u32,
    /// 1~50.
    pub per_page: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: PrState,
    pub draft: bool,
    pub head_branch: String,
    /// 다른 저장소(포크)에서 낸 것이면 그 소유자. 지워진 포크면 `None`이고 `head_deleted`.
    pub head_owner: Option<String>,
    pub head_deleted: bool,
    pub base_branch: String,
    /// 탈퇴한 사용자면 `None`.
    pub author: Option<String>,
    /// unix 초.
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrPage {
    pub items: Vec<PullRequest>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mergeable {
    Clean,
    Conflicts,
    Behind,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChecksSummary {
    pub success: u32,
    pub failed: u32,
    pub pending: u32,
    /// 100개를 넘어 일부만 셌다.
    pub partial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrDetail {
    pub pr: PullRequest,
    pub mergeable: Mergeable,
    /// 시험 결과를 볼 수 없으면(권한·없음) `None`.
    pub checks: Option<ChecksSummary>,
    /// 리뷰 요약. 볼 수 없거나(권한·없음) 100개를 넘으면 `None`(D-134).
    pub reviews: Option<ReviewsSummary>,
    /// 댓글 수(대화 댓글 + 코드 댓글).
    pub comments: u32,
    /// 대화 댓글 수(댓글 쪽 계산에 쓴다, D-178).
    pub issue_comments: u32,
    /// 코드 줄 댓글 수.
    pub review_comments: u32,
    /// head 버전(16진수 40·64자를 확인한 것). 합치기의 `sha`(D-179).
    pub head_sha: String,
}

/// 사람마다 마지막 리뷰 상태로 센 승인·변경 요청 수(D-134). 취소된 리뷰(`DISMISSED`)는 빼고
/// 댓글만 남긴 리뷰(`COMMENTED`)와 아직 내지 않은 리뷰(`PENDING`)는 상태를 바꾸지 않는다.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReviewsSummary {
    pub approved: u32,
    pub changes_requested: u32,
}

/// 리뷰를 읽는 한 쪽의 수. 이보다 많으면 요약을 보이지 않는다.
pub(crate) const REVIEWS_PAGE: u32 = 100;

/// 새 검토 요청.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPr {
    pub title: String,
    pub body: String,
    /// 올라가 있는 원격 브랜치 이름.
    pub head: String,
    /// 포크에서 내면 그 소유자(`head`가 `<소유자>:<브랜치>`로 나간다, D-181).
    pub head_owner: Option<String>,
    pub base: String,
    pub draft: bool,
}

// ---- 응답 모양 ----

#[derive(Deserialize)]
struct UserJson {
    login: String,
}

#[derive(Deserialize)]
struct RepoJson {
    owner: UserJson,
}

#[derive(Deserialize)]
struct RefJson {
    #[serde(rename = "ref")]
    name: String,
    sha: Option<String>,
    repo: Option<RepoJson>,
}

#[derive(Deserialize)]
struct PullJson {
    number: u64,
    title: String,
    state: String,
    #[serde(default)]
    draft: bool,
    merged_at: Option<String>,
    head: RefJson,
    base: RefJson,
    user: Option<UserJson>,
    created_at: String,
    updated_at: String,
    mergeable_state: Option<String>,
    #[serde(default)]
    comments: u64,
    #[serde(default)]
    review_comments: u64,
}

#[derive(Deserialize)]
struct ReviewJson {
    user: Option<UserJson>,
    state: String,
}

#[derive(Deserialize)]
struct CheckRunJson {
    conclusion: Option<String>,
}

#[derive(Deserialize)]
struct CheckRunsJson {
    total_count: u64,
    check_runs: Vec<CheckRunJson>,
}

#[derive(Deserialize)]
struct ValidationJson {
    message: Option<String>,
    #[serde(default)]
    errors: Vec<ValidationItem>,
}

#[derive(Deserialize)]
struct ValidationItem {
    message: Option<String>,
    code: Option<String>,
}

#[derive(Serialize)]
struct NewPrJson<'a> {
    title: &'a str,
    body: &'a str,
    head: &'a str,
    base: &'a str,
    draft: bool,
}

fn bad(detail: &str) -> HostingError {
    HostingError::with_detail(HostingErrorKind::BadResponse, detail)
}

fn invalid(detail: &str) -> HostingError {
    HostingError::with_detail(
        HostingErrorKind::Validation(ValidationKind::Invalid),
        detail,
    )
}

pub(crate) fn parse_list(body: &[u8], base_owner: &str) -> Result<Vec<PullRequest>, HostingError> {
    let items: Vec<PullJson> =
        serde_json::from_slice(body).map_err(|_| bad("pull request list is not JSON"))?;
    items
        .into_iter()
        .map(|item| convert(item, base_owner).map(|(pr, _, _)| pr))
        .collect()
}

pub(crate) fn parse_one(body: &[u8], base_owner: &str) -> Result<PullRequest, HostingError> {
    let item: PullJson =
        serde_json::from_slice(body).map_err(|_| bad("pull request is not JSON"))?;
    convert(item, base_owner).map(|(pr, _, _)| pr)
}

/// 상세: 검토 요청과 합칠 수 있는지, 시험 결과를 볼 head sha.
pub(crate) fn parse_detail(body: &[u8], base_owner: &str) -> Result<PrDetail, HostingError> {
    let item: PullJson =
        serde_json::from_slice(body).map_err(|_| bad("pull request is not JSON"))?;
    let issue_comments = u32::try_from(item.comments).unwrap_or(u32::MAX);
    let review_comments = u32::try_from(item.review_comments).unwrap_or(u32::MAX);
    let comments = issue_comments.saturating_add(review_comments);
    let (pr, sha, mergeable_state) = convert(item, base_owner)?;
    let head_sha = sha
        .filter(|sha| valid_sha(sha))
        .ok_or_else(|| bad("unexpected head sha"))?;
    let mergeable = match mergeable_state.as_deref() {
        Some("clean" | "has_hooks" | "unstable") => Mergeable::Clean,
        Some("dirty") => Mergeable::Conflicts,
        Some("behind") => Mergeable::Behind,
        Some("blocked") => Mergeable::Blocked,
        _ => Mergeable::Unknown,
    };
    Ok(PrDetail {
        pr,
        mergeable,
        checks: None,
        reviews: None,
        comments,
        issue_comments,
        review_comments,
        head_sha,
    })
}

/// `GET pulls/{n}/reviews`(오래된 것부터). 사람마다 마지막으로 정한 상태만 센다.
pub(crate) fn parse_reviews(body: &[u8]) -> Result<ReviewsSummary, HostingError> {
    let parsed: Vec<ReviewJson> =
        serde_json::from_slice(body).map_err(|_| bad("reviews are not JSON"))?;
    let mut last: Vec<(String, &str)> = Vec::new();
    for review in &parsed {
        let Some(user) = &review.user else { continue };
        let state = match review.state.as_str() {
            s @ ("APPROVED" | "CHANGES_REQUESTED" | "DISMISSED") => s,
            _ => continue,
        };
        match last.iter_mut().find(|(login, _)| *login == user.login) {
            Some(entry) => entry.1 = state,
            None => last.push((user.login.clone(), state)),
        }
    }
    let count = |want: &str| {
        u32::try_from(last.iter().filter(|(_, s)| *s == want).count()).unwrap_or(u32::MAX)
    };
    Ok(ReviewsSummary {
        approved: count("APPROVED"),
        changes_requested: count("CHANGES_REQUESTED"),
    })
}

pub(crate) fn parse_check_runs(body: &[u8]) -> Result<ChecksSummary, HostingError> {
    let parsed: CheckRunsJson =
        serde_json::from_slice(body).map_err(|_| bad("check runs are not JSON"))?;
    let mut summary = ChecksSummary {
        partial: parsed.total_count > u64::from(CHECK_RUNS_PAGE),
        ..ChecksSummary::default()
    };
    for run in parsed.check_runs {
        match run.conclusion.as_deref() {
            None => summary.pending += 1,
            Some("success" | "neutral" | "skipped") => summary.success += 1,
            Some(_) => summary.failed += 1,
        }
    }
    Ok(summary)
}

/// 422 본문을 나눈다.
pub(crate) fn validation_error(body: &[u8]) -> HostingError {
    let Ok(parsed) = serde_json::from_slice::<ValidationJson>(body) else {
        return invalid("HTTP 422");
    };
    let messages: Vec<&str> = parsed
        .errors
        .iter()
        .filter_map(|e| e.message.as_deref().or(e.code.as_deref()))
        .collect();
    let joined = messages.join("; ");
    let kind = if messages
        .iter()
        .any(|m| m.contains("A pull request already exists") || m.contains("name already exists"))
    {
        ValidationKind::AlreadyExists
    } else if messages.iter().any(|m| m.contains("No commits between")) {
        ValidationKind::NoDifference
    } else {
        ValidationKind::Invalid
    };
    let detail = if joined.is_empty() {
        parsed.message.unwrap_or_default()
    } else {
        joined
    };
    HostingError::with_detail(HostingErrorKind::Validation(kind), &detail)
}

/// 만들기 전 입력 검사. 틀리면 보내지 않는다.
pub(crate) fn check_new_pr(pr: &NewPr) -> Result<(), HostingError> {
    let title = pr.title.trim();
    if title.is_empty() || title.len() > PR_TITLE_MAX_BYTES {
        return Err(invalid("title length"));
    }
    if pr
        .title
        .chars()
        .any(|c| c.is_control() && c != '\t' && c != '\n')
    {
        return Err(invalid("title has control characters"));
    }
    if pr.body.len() > PR_BODY_MAX_BYTES {
        return Err(invalid("body too long"));
    }
    if !valid_branch(&pr.head) || !valid_branch(&pr.base) || pr.head.contains(':') {
        return Err(invalid("branch name"));
    }
    if pr
        .head_owner
        .as_deref()
        .is_some_and(|owner| !crate::remote::valid_owner(owner))
    {
        return Err(invalid("head owner"));
    }
    Ok(())
}

pub(crate) fn new_pr_json(pr: &NewPr) -> Result<Vec<u8>, HostingError> {
    let head = match &pr.head_owner {
        Some(owner) => format!("{owner}:{}", pr.head),
        None => pr.head.clone(),
    };
    serde_json::to_vec(&NewPrJson {
        title: pr.title.trim(),
        body: &pr.body,
        head: &head,
        base: &pr.base,
        draft: pr.draft,
    })
    .map_err(|_| invalid("could not encode request"))
}

pub(crate) fn valid_branch(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= BRANCH_MAX_BYTES
        && !name.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// 40자(SHA-1)나 64자(SHA-256) 16진수.
pub(crate) fn valid_sha(sha: &str) -> bool {
    (sha.len() == 40 || sha.len() == 64) && sha.bytes().all(|b| b.is_ascii_hexdigit())
}

fn convert(
    item: PullJson,
    base_owner: &str,
) -> Result<(PullRequest, Option<String>, Option<String>), HostingError> {
    let state = match (item.state.as_str(), item.merged_at.is_some()) {
        ("open", _) => PrState::Open,
        ("closed", true) => PrState::Merged,
        ("closed", false) => PrState::Closed,
        _ => return Err(bad("unexpected pull request state")),
    };
    let head_deleted = item.head.repo.is_none();
    let head_owner = item
        .head
        .repo
        .map(|repo| repo.owner.login)
        .filter(|owner| !owner.eq_ignore_ascii_case(base_owner));
    let pr = PullRequest {
        number: item.number,
        title: clean_text(&item.title, TITLE_SHOWN_MAX_CHARS),
        state,
        draft: item.draft,
        head_branch: item.head.name,
        head_owner,
        head_deleted,
        base_branch: item.base.name,
        author: item.user.map(|u| u.login),
        created_at: parse_time(&item.created_at).ok_or_else(|| bad("unexpected time"))?,
        updated_at: parse_time(&item.updated_at).ok_or_else(|| bad("unexpected time"))?,
    };
    Ok((pr, item.head.sha, item.mergeable_state))
}

fn clean_text(text: &str, max_chars: usize) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(max_chars)
        .collect()
}

/// `YYYY-MM-DDTHH:MM:SSZ`를 unix 초로. 다른 모양은 `None`.
pub(crate) fn parse_time(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let num = |range: std::ops::Range<usize>| -> Option<i64> {
        let part = text.get(range)?;
        if !part.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        part.parse().ok()
    };
    let (year, month, day) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hour, minute, second) = (num(11..13)?, num(14..16)?, num(17..19)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

/// 그레고리력 날짜 → 1970-01-01부터 지난 날 수(Howard Hinnant의 알고리즘).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `Link` 헤더에 `rel="next"`가 있는지.
pub(crate) fn has_next_link(link: Option<&str>) -> bool {
    link.is_some_and(|link| {
        link.split(',').any(|part| {
            part.split(';')
                .skip(1)
                .any(|param| param.trim().eq_ignore_ascii_case(r#"rel="next""#))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pull(extra: &str) -> String {
        format!(
            r#"{{"number":7,"title":"Add login","state":"open","draft":false,"merged_at":null,
               "head":{{"ref":"feature/login","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
               "base":{{"ref":"main","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
               "user":{{"login":"mona"}},"created_at":"2026-10-01T09:30:00Z",
               "updated_at":"2026-10-02T00:00:00Z"{extra}}}"#,
            sha = "a".repeat(40)
        )
    }

    #[test]
    fn reads_list_samples() {
        let sha = "b".repeat(40);
        let long_title = "가".repeat(1000);
        let json = format!(
            r#"[{open},
            {{"number":8,"title":"Merged one","state":"closed","merged_at":"2026-10-03T00:00:00Z",
              "head":{{"ref":"x","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
              "base":{{"ref":"main","sha":"{sha}","repo":null}},"user":null,
              "created_at":"2026-10-01T00:00:00Z","updated_at":"2026-10-03T00:00:00Z"}},
            {{"number":9,"title":"🚀 {long_title}","state":"closed","merged_at":null,"draft":true,
              "head":{{"ref":"fix","sha":"{sha}","repo":{{"owner":{{"login":"forker"}}}}}},
              "base":{{"ref":"main","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
              "user":{{"login":"forker"}},
              "created_at":"2026-10-01T00:00:00Z","updated_at":"2026-10-03T00:00:00Z"}},
            {{"number":10,"title":"gone fork","state":"open",
              "head":{{"ref":"y","sha":"{sha}","repo":null}},
              "base":{{"ref":"main","sha":"{sha}","repo":{{"owner":{{"login":"octo"}}}}}},
              "user":{{"login":"ghost"}},
              "created_at":"2026-10-01T00:00:00Z","updated_at":"2026-10-03T00:00:00Z"}}]"#,
            open = pull("")
        );
        let items = parse_list(json.as_bytes(), "Octo").unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].state, PrState::Open);
        assert_eq!(items[0].head_branch, "feature/login");
        assert_eq!(items[0].head_owner, None);
        assert_eq!(items[0].author.as_deref(), Some("mona"));
        assert_eq!(items[0].created_at, 1_790_847_000);
        assert_eq!(items[1].state, PrState::Merged);
        assert_eq!(items[1].author, None);
        assert_eq!(items[2].state, PrState::Closed);
        assert!(items[2].draft);
        assert_eq!(items[2].head_owner.as_deref(), Some("forker"));
        assert!(items[2].title.starts_with("🚀 가"));
        assert_eq!(items[3].head_owner, None);
        assert!(items[3].head_deleted);
    }

    #[test]
    fn rejects_bad_list() {
        assert!(parse_list(b"<html>", "o").is_err());
        assert!(parse_list(br#"{"message":"x"}"#, "o").is_err());
        let weird_state = pull("").replace(r#""state":"open""#, r#""state":"weird""#);
        assert!(parse_list(format!("[{weird_state}]").as_bytes(), "o").is_err());
        let bad_time = pull("").replace("2026-10-01T09:30:00Z", "2026-10-01 09:30:00");
        assert!(parse_list(format!("[{bad_time}]").as_bytes(), "o").is_err());
    }

    #[test]
    fn detail_mergeable_table() {
        let cases = [
            ("clean", Mergeable::Clean),
            ("unstable", Mergeable::Clean),
            ("has_hooks", Mergeable::Clean),
            ("dirty", Mergeable::Conflicts),
            ("behind", Mergeable::Behind),
            ("blocked", Mergeable::Blocked),
            ("unknown", Mergeable::Unknown),
            ("draft", Mergeable::Unknown),
        ];
        for (state, want) in cases {
            let json = pull(&format!(r#","mergeable_state":"{state}""#));
            let detail = parse_detail(json.as_bytes(), "octo").unwrap();
            assert_eq!(detail.mergeable, want, "{state}");
        }
        let detail = parse_detail(pull("").as_bytes(), "octo").unwrap();
        assert_eq!(detail.mergeable, Mergeable::Unknown);
        assert_eq!(detail.head_sha, "a".repeat(40));
    }

    #[test]
    fn detail_rejects_odd_sha() {
        for sha in ["abc", "../../x", &"g".repeat(40), &"a".repeat(41)] {
            let json = pull("").replacen(&"a".repeat(40), sha, 1);
            assert!(parse_detail(json.as_bytes(), "octo").is_err(), "{sha}");
        }
        let json = pull("").replacen(&"a".repeat(40), &"c".repeat(64), 1);
        assert!(parse_detail(json.as_bytes(), "octo").is_ok());
    }

    #[test]
    fn check_runs_summary() {
        let json = r#"{"total_count":7,"check_runs":[
            {"status":"completed","conclusion":"success"},
            {"status":"completed","conclusion":"neutral"},
            {"status":"completed","conclusion":"skipped"},
            {"status":"completed","conclusion":"failure"},
            {"status":"completed","conclusion":"timed_out"},
            {"status":"completed","conclusion":"action_required"},
            {"status":"in_progress","conclusion":null}]}"#;
        assert_eq!(
            parse_check_runs(json.as_bytes()).unwrap(),
            ChecksSummary {
                success: 3,
                failed: 3,
                pending: 1,
                partial: false
            }
        );
        let many = r#"{"total_count":150,"check_runs":[]}"#;
        assert!(parse_check_runs(many.as_bytes()).unwrap().partial);
    }

    #[test]
    fn validation_table() {
        let kind = |body: &str| match validation_error(body.as_bytes()).kind() {
            HostingErrorKind::Validation(kind) => kind,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            kind(
                r#"{"message":"Validation Failed","errors":[{"message":"A pull request already exists for octo:x."}]}"#
            ),
            ValidationKind::AlreadyExists
        );
        assert_eq!(
            kind(
                r#"{"message":"Validation Failed","errors":[{"message":"No commits between main and x"}]}"#
            ),
            ValidationKind::NoDifference
        );
        assert_eq!(
            kind(r#"{"message":"Validation Failed","errors":[{"code":"invalid","field":"head"}]}"#),
            ValidationKind::Invalid
        );
        assert_eq!(kind("not json"), ValidationKind::Invalid);
        let err = validation_error(
            br#"{"message":"Draft pull requests are not supported in this repository.","errors":[]}"#,
        );
        assert!(err.detail().unwrap().contains("Draft"));
    }

    #[test]
    fn new_pr_checks_and_json() {
        let ok = NewPr {
            title: "제목 \"quoted\" \\ back".into(),
            body: "line1\nline2 \u{1F600}".into(),
            head: "feature/x".into(),
            head_owner: None,
            base: "main".into(),
            draft: true,
        };
        check_new_pr(&ok).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&new_pr_json(&ok).unwrap()).unwrap();
        assert_eq!(json["title"], "제목 \"quoted\" \\ back");
        assert_eq!(json["body"], "line1\nline2 \u{1F600}");
        assert_eq!(json["head"], "feature/x");
        assert_eq!(json["base"], "main");
        assert_eq!(json["draft"], true);
        let raw = String::from_utf8(new_pr_json(&ok).unwrap()).unwrap();
        assert!(
            raw.contains(r#""title":"제목 \"quoted\" \\ back""#),
            "{raw}"
        );
        assert!(raw.contains(r#"line1\nline2"#), "{raw}");
        let from_fork = NewPr {
            head_owner: Some("mona".into()),
            ..ok.clone()
        };
        check_new_pr(&from_fork).unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&new_pr_json(&from_fork).unwrap()).unwrap();
        assert_eq!(json["head"], "mona:feature/x");

        let bad = [
            NewPr {
                title: "  ".into(),
                ..ok.clone()
            },
            NewPr {
                title: "a".repeat(257),
                ..ok.clone()
            },
            NewPr {
                title: "a\u{7}b".into(),
                ..ok.clone()
            },
            NewPr {
                body: "a".repeat(64 * 1024 + 1),
                ..ok.clone()
            },
            NewPr {
                head: "has space".into(),
                ..ok.clone()
            },
            NewPr {
                head: String::new(),
                ..ok.clone()
            },
            NewPr {
                base: "a".repeat(256),
                ..ok.clone()
            },
            // 소유자는 head_owner로만 붙는다.
            NewPr {
                head: "mona:feature".into(),
                ..ok.clone()
            },
            NewPr {
                head_owner: Some("-mona".into()),
                ..ok.clone()
            },
        ];
        for pr in bad {
            assert_eq!(
                check_new_pr(&pr).unwrap_err().kind(),
                HostingErrorKind::Validation(ValidationKind::Invalid),
                "{pr:?}"
            );
        }
    }

    #[test]
    fn link_header() {
        assert!(has_next_link(Some(
            r#"<https://api.github.com/repositories/1/pulls?page=2>; rel="next", <https://api.github.com/repositories/1/pulls?page=5>; rel="last""#
        )));
        assert!(!has_next_link(Some(
            r#"<https://api.github.com/repositories/1/pulls?page=1>; rel="prev""#
        )));
        assert!(!has_next_link(None));
        assert!(!has_next_link(Some("garbage")));
    }

    #[test]
    fn time_parsing() {
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time("2000-03-01T00:00:00Z"), Some(951_868_800));
        assert_eq!(parse_time("2026-10-01T09:30:00Z"), Some(1_790_847_000));
        for bad in [
            "2026-10-01T09:30:00",
            "2026-13-01T00:00:00Z",
            "2026-10-01T24:00:00Z",
            "+026-10-01T00:00:00Z",
            "2026-10-01T09:30:00.000Z",
            "",
        ] {
            assert_eq!(parse_time(bad), None, "{bad}");
        }
    }

    #[test]
    fn reviews_count_each_person_by_their_last_decision() {
        let body = br#"[
            {"user":{"login":"a"},"state":"APPROVED"},
            {"user":{"login":"a"},"state":"CHANGES_REQUESTED"},
            {"user":{"login":"b"},"state":"APPROVED"},
            {"user":{"login":"b"},"state":"COMMENTED"},
            {"user":{"login":"c"},"state":"APPROVED"},
            {"user":{"login":"c"},"state":"DISMISSED"},
            {"user":{"login":"d"},"state":"PENDING"},
            {"user":null,"state":"APPROVED"}
        ]"#;
        assert_eq!(
            parse_reviews(body).unwrap(),
            ReviewsSummary {
                approved: 1,
                changes_requested: 1
            }
        );
        assert_eq!(parse_reviews(b"[]").unwrap(), ReviewsSummary::default());
        assert!(parse_reviews(b"nope").is_err());
    }
}
