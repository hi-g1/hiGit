//! 원격 브랜치 지우기·이름 바꾸기 확인 창의 값(D-130). 네트워크 없이 로컬 추적 ref로 계산한다.

use serde::Serialize;
use specta::Type;

/// 지우거나 이름을 바꿀 수 없는 까닭.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RemoteBranchBlock {
    /// 원격 기본 브랜치(`main`·`master`·`init.defaultBranch`·원격 HEAD).
    Protected,
    /// 로컬에 그 원격 브랜치의 추적 ref가 없다(원격 확인 뒤 다시).
    NoTrackingRef,
    /// 그런 원격이 없다.
    NoRemote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchPlanView {
    pub remote: String,
    pub branch: String,
    /// 추적 ref가 가리키는 버전(실행 입력의 기대 값).
    pub tip: Option<String>,
    /// 이 브랜치에만 있는 버전 수(원격 기본 브랜치·다른 추적 브랜치·로컬 브랜치에서 닿지 않음, 200개까지 셈).
    pub only_here: u32,
    /// 그 가운데 다른 사람이 만든 것(작성자 이메일이 내 이메일과 다름, 모르면 모두).
    pub by_others: u32,
    pub blocked: Option<RemoteBranchBlock>,
    /// 지금 브랜치의 upstream이다(지우면 upstream이 사라진다).
    pub is_upstream_of_current: bool,
}
