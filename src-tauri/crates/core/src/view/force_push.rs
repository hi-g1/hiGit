//! 덮어쓰며 올리기 확인 창의 값(D-128). 네트워크 없이 로컬 정보로만 계산한다.

use serde::Serialize;
use specta::Type;

/// 덮어쓰며 올리기를 할 수 없는 까닭.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ForcePushBlock {
    /// 설정 "올린 버전도 바꾸기"가 꺼져 있다(D-127).
    Disabled,
    /// 특정 버전을 보는 중이다.
    Detached,
    /// 합치는 중이거나 다른 도구의 작업이 진행 중이다.
    OtherOperation,
    /// 올린 적이 없다(upstream 없음).
    NoUpstream,
    /// upstream이 원격 기본 브랜치다(`main`·`master`·`init.defaultBranch`·원격 HEAD).
    DefaultBranch,
    /// 원격에 내 기록에 없는 버전이 없다. 덮어쓸 것 없이 그냥 올리면 된다.
    NothingToOverwrite,
}

/// 원격에서 사라질 버전 하나(앞 5개만 보인다).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DroppedVersion {
    pub short_oid: String,
    pub summary: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ForcePushPlanView {
    /// 설정이 켜져 있다(꺼져 있으면 화면은 진입점을 숨긴다).
    pub allowed: bool,
    pub blocked: Option<ForcePushBlock>,
    /// 지금 브랜치(로컬).
    pub branch: Option<String>,
    /// upstream 원격 이름과 원격 브랜치 이름. 실행 입력에 그대로 담는다.
    pub remote: Option<String>,
    pub remote_branch: Option<String>,
    /// 확인 창이 보여 줄 원격 끝과 지금 위치. 실행 입력의 기대 값이 된다.
    pub expect_remote: Option<String>,
    pub expect_head: Option<String>,
    /// 올라갈 내 버전 수(원격에 없는 것).
    pub ahead: u32,
    /// 원격에서 사라질 버전 수(내 기록에 없는 것).
    pub dropped: u32,
    /// 사라질 버전 가운데 다른 사람이 만든 것(작성자 이메일이 내 이메일과 다름, 모르면 모두).
    pub dropped_by_others: u32,
    pub sample: Vec<DroppedVersion>,
    /// 이 브랜치에 "올린 기록을 바꿨다" 표식이 있다(D-129).
    pub rewrote_published: bool,
}
