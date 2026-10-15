//! 끝나지 않은 작업(D-082). 프로젝트를 열 때 알림 줄로 보인다.

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PendingWorkView {
    /// 오래된 잠금 파일(git 폴더 기준 이름, D-081). 비면 정리할 것이 없다.
    pub stale_locks: Vec<String>,
    /// stash 목록에 남은 맡긴 항목 수. 브랜치를 갈아타다 끊기면 남는다.
    pub leftover_parked: u32,
    /// 다른 도구가 시작해 앱이 끝낼 수 없는 작업.
    pub other_operation: Option<OtherOperation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum OtherOperation {
    Rebase,
    CherryPick,
    Revert,
}
