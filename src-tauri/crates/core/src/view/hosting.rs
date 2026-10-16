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
