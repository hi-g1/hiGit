//! 검토 요청 화면 타입(D-112, D-113). 주소·토큰은 없다(페이지는 번호로 core가 연다).

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum HostingProviderView {
    Github,
}

/// 열린 저장소의 호스팅 정보. 네트워크 없이 로컬 정보만으로 만든다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HostingContextView {
    /// 지금 브랜치가 올리는 원격이 github.com이면 `github`.
    pub provider: Option<HostingProviderView>,
    /// `소유자/이름`.
    pub repo: Option<String>,
    pub configured: bool,
    pub connected: bool,
    /// 지금 로컬 브랜치.
    pub branch: Option<String>,
    /// 검토 요청의 head가 될 원격 브랜치 이름(upstream). 아직 올린 적 없으면 `None`.
    pub head: Option<String>,
    /// 기본 브랜치.
    pub base: Option<String>,
    pub is_default_branch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PrStateFilterView {
    Open,
    Closed,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrListRequest {
    pub state: PrStateFilterView,
    /// 이 원격 브랜치에서 낸 것만.
    pub branch: Option<String>,
    /// 1부터.
    pub page: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PrStateView {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrRowView {
    pub number: u32,
    pub title: String,
    pub state: PrStateView,
    pub draft: bool,
    pub head: String,
    /// 다른 저장소(포크)에서 냈으면 그 소유자.
    pub head_owner: Option<String>,
    pub base: String,
    pub author: Option<String>,
    /// unix 초.
    #[specta(type = u32)]
    pub updated_at: i64,
    /// 이 저장소의 지금 브랜치(원격 브랜치 이름이 같음)에서 낸 것.
    pub is_current_branch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrListView {
    pub items: Vec<PrRowView>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MergeableView {
    Clean,
    Conflicts,
    Behind,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChecksView {
    pub success: u32,
    pub failed: u32,
    pub pending: u32,
    /// 100개를 넘어 일부만 셌다.
    pub partial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrDetailView {
    pub row: PrRowView,
    pub mergeable: MergeableView,
    /// 시험 결과를 볼 수 없으면 `None`.
    pub checks: Option<ChecksView>,
    /// 리뷰 요약(사람마다 마지막 승인·변경 요청). 볼 수 없거나 100개를 넘으면 `None`(D-134).
    pub reviews: Option<ReviewsView>,
    /// 댓글 수(대화 + 코드).
    pub comments: u32,
    /// 대화 댓글 수. 댓글 목록 조회가 마지막 쪽을 계산하는 데 쓴다(D-178).
    pub issue_comments: u32,
    /// head 버전. 합치기가 이 값일 때만 합친다(D-179).
    pub head_oid: String,
}

/// 합치는 방법(D-179). "다시 쌓아 합치기"는 없다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MergeMethodView {
    Merge,
    Squash,
}

/// 검토 요청 화면이 단추를 정하는 저장소 정보(D-177).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrRepoView {
    /// 연결된 계정이 이 저장소에 올릴 수 있다.
    pub can_push: bool,
    /// 합칠 수 있다(올릴 권한과 같음, 서버가 마지막에 정한다).
    pub can_merge: bool,
    /// 저장소가 허용하는 방법(앞이 기본). 칸이 없으면 `[merge]`.
    pub merge_methods: Vec<MergeMethodView>,
    /// "다시 쌓아 합치기"만 허용한다(화면은 "GitHub에서 합쳐 주세요").
    pub rebase_only: bool,
    pub archived: bool,
    /// 연결된 로그인.
    pub viewer: Option<String>,
}

/// 검토 요청의 최근 댓글(D-178). 글자는 화면이 글자로만 그린다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrCommentView {
    /// 탈퇴한 사용자면 `None`.
    pub author: Option<String>,
    #[specta(type = u32)]
    pub created_at: i64,
    /// 앞 4000자.
    pub body: String,
    /// 연결된 로그인이 쓴 댓글.
    pub mine: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrCommentsView {
    /// 오래된 것부터, 최근 20개까지.
    pub items: Vec<PrCommentView>,
    /// 대화 댓글 수(화면이 넘긴 값).
    pub total: u32,
    /// 이보다 앞의 댓글이 더 있다.
    pub partial: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReviewsView {
    pub approved: u32,
    pub changes_requested: u32,
}

/// 새 검토 요청 대화상자의 기본값. 로컬 정보만으로 만든다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrDraftView {
    pub branch: String,
    /// 올라가 있는 원격 브랜치 이름. 아직 올린 적 없으면 `None`(화면이 먼저 올린다).
    pub head: Option<String>,
    pub base: String,
    /// 그 원격의 브랜치들(기본 브랜치 먼저, 100개까지). 지금 브랜치는 뺀다.
    pub base_options: Vec<String>,
    pub suggestion_title: String,
    /// `base`보다 앞선 버전 수.
    pub ahead: u32,
    /// 아직 올리지 않은 버전 수(upstream이 없으면 앞선 버전 전부).
    pub unpushed: u32,
}
