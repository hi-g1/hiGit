//! GitHub에 올리기 대화상자의 처음 값(D-132). 네트워크 없이 읽는다.

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PublishDraftView {
    /// 폴더 이름에서 만든 저장소 이름 제안(`[A-Za-z0-9._-]`만, 못 만들면 빈 값).
    pub suggested_name: String,
    /// 지금 브랜치(없으면 특정 버전을 보는 중).
    pub branch: Option<String>,
    /// 저장한 버전 수(0이면 올릴 것이 없다, 10000에서 멈춰 셈).
    pub versions: u32,
    /// 원격 수(하나라도 있으면 GitHub에 올리기 대신 동기화).
    pub remotes: u32,
    pub configured: bool,
    pub connected: bool,
    pub login: Option<String>,
}
