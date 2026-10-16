//! 전역 이름·이메일 조회(D-125).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IdentityView {
    pub name: Option<String>,
    pub email: Option<String>,
    /// 이름과 이메일이 둘 다 있다.
    pub complete: bool,
    /// GitHub에 기억한 연결이 있으면 가림 이메일. 화면은 칸에 채우기만 한다(D-126).
    pub noreply_email: Option<String>,
}
