//! `HostingProvider` 트레이트와 GitHub 구현(반복 4).
//!
//! 다른 내부 크레이트에 의존하지 않는다. 네트워크는 [`HttpTransport`] 뒤에 두어
//! 시험은 가짜 전송으로 돈다. 토큰은 `SecretString`으로만 다룬다(D-040).

#![forbid(unsafe_code)]

mod api;
mod config;
mod error;
mod github;
mod http;
mod login;
mod pr;
mod remote;
mod repo;
#[doc(hidden)]
pub mod testing;
mod ureq_transport;

pub use config::{GITHUB_API_BASE, GITHUB_WEB_BASE, GitHubConfig, valid_client_id};
pub use error::{DETAIL_MAX_CHARS, HostingError, HostingErrorKind, ValidationKind};
pub use github::GitHub;
pub use http::{HttpRequest, HttpResponse, HttpTransport, Method, TransportError};
pub use login::{HostUser, LoginChallenge, LoginPoll, LoginScope};
pub use pr::{
    BRANCH_MAX_BYTES, ChecksSummary, Mergeable, NewPr, PR_BODY_MAX_BYTES, PR_PAGE_MAX,
    PR_TITLE_MAX_BYTES, PrDetail, PrPage, PrQuery, PrState, PrStateFilter, PullRequest,
    ReviewsSummary,
};
pub use remote::{GITHUB_HOST, RemoteRepo, parse_remote, valid_name, valid_owner};
pub use repo::{
    CreatedRepo, NewRepo, REPO_DESCRIPTION_MAX_BYTES, REPO_PAGE_MAX, RepoPage, RepoSummary,
};
pub use secrecy::{ExposeSecret, SecretString};
pub use ureq_transport::{BODY_MAX_BYTES, CONNECT_TIMEOUT, TOTAL_TIMEOUT, UreqTransport};

/// 호스팅 서비스 하나(지금은 github.com만). 모든 호출은 블로킹이고 core가 작업 스레드에서 부른다.
pub trait HostingProvider: Send + Sync {
    /// 서비스 이름(`"github"`).
    fn id(&self) -> &'static str;
    /// 원격 주소가 이 서비스의 저장소면 저장소.
    fn detect(&self, url: &str) -> Option<RemoteRepo>;
    /// device flow 시작: 사람이 브라우저에 넣을 코드를 받는다.
    fn begin_login(&self, scope: LoginScope) -> Result<LoginChallenge, HostingError>;
    /// 승인됐는지 한 번 묻는다. 간격은 부른 쪽이 지킨다.
    fn poll_login(&self, challenge: &LoginChallenge) -> Result<LoginPoll, HostingError>;
    /// 토큰 주인을 확인한다.
    fn current_user(&self, token: &SecretString) -> Result<HostUser, HostingError>;
    /// 검토 요청 목록 한 쪽(최근에 바뀐 것부터).
    fn list_prs(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        query: &PrQuery,
    ) -> Result<PrPage, HostingError>;
    /// 검토 요청 하나와 합칠 수 있는지, 시험 결과 요약.
    fn pr_detail(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        number: u64,
    ) -> Result<PrDetail, HostingError>;
    /// 새 검토 요청. 입력이 규칙에 맞지 않으면 보내지 않는다.
    fn create_pr(
        &self,
        token: &SecretString,
        repo: &RemoteRepo,
        new: &NewPr,
    ) -> Result<PullRequest, HostingError>;
    /// 토큰 주인의 계정에 빈 저장소를 만든다(D-132). 입력이 규칙에 맞지 않으면 보내지 않는다.
    fn create_repo(&self, token: &SecretString, new: &NewRepo)
    -> Result<CreatedRepo, HostingError>;
    /// 토큰 주인이 볼 수 있는 저장소 한 쪽(최근에 올린 것부터, D-133).
    fn list_repos(
        &self,
        token: &SecretString,
        page: u32,
        per_page: u32,
    ) -> Result<RepoPage, HostingError>;
}
