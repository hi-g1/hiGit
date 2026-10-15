//! 홈 두 줄 노선도 조회의 화면 타입(D-046).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MapCommit {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
}

/// main과 지금 브랜치 두 줄(지금 브랜치가 main이면 한 줄)을 그리는 데 필요한 것만.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HomeMapView {
    /// 지금 브랜치. 특정 버전을 보는 중이면 null.
    pub current: Option<String>,
    /// 기본 브랜치(D-048). 없으면 null(한 줄로 그린다).
    pub default_branch: Option<String>,
    /// 지금 브랜치가 기본 브랜치에서 갈라져 있어 두 줄로 그린다.
    pub two_lines: bool,
    /// 지금 줄에 그릴 버전(오래된 것부터, 최대 3개). 두 줄이면 갈라진 뒤의 버전만.
    pub current_commits: Vec<MapCommit>,
    /// 그리지 않은 앞쪽 버전 수("…"로 접는다).
    pub current_more: u32,
    /// 갈라진 뒤 기본 브랜치에 생긴 버전 수.
    pub main_since: u32,
    /// 기본 브랜치의 마지막 버전(두 줄일 때).
    pub main_last: Option<MapCommit>,
    /// 지금 브랜치 upstream이 가리키는 버전과 그 표시("GitHub" 또는 원격 이름).
    pub upstream_oid: Option<String>,
    pub upstream_label: Option<String>,
    pub changed_files: u32,
}
