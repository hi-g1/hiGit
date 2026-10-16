//! GitHub 연결 화면 타입(D-109). 토큰은 어떤 타입에도 필드가 없다.

use serde::{Deserialize, Serialize};
use specta::Type;

/// 설정의 GitHub 구역이 그리는 상태. 키체인을 건드리지 않고 만든다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GitHubStatusView {
    /// client id가 있어 연결할 수 있다. 아니면 "준비 안 됨".
    pub configured: bool,
    pub connected: bool,
    pub login: Option<String>,
    /// 키체인에 저장한 연결. 연결됐는데 거짓이면 "이번 실행에만 저장됨".
    pub remembered: bool,
    /// 워크플로 권한을 받은 연결(D-135).
    pub workflow: bool,
    /// 이메일 확인 권한(`user:email`)을 받은 연결(D-186).
    pub email: bool,
}

/// 다시 연결하며 더할 권한(D-135, D-186). 지금 가진 권한은 그대로 두고 더한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ScopeAdd {
    Workflow,
    Email,
}

/// 전역 이메일이 GitHub 계정의 것인지(D-186). 계정의 이메일 목록은 담지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum EmailCheckState {
    /// GitHub에 연결돼 있지 않다.
    NotConnected,
    /// 전역 이메일이 없다.
    NoIdentity,
    /// 이 계정의 가림 주소다.
    Noreply,
    /// 이메일 확인 권한이 없다.
    NoPermission,
    /// 계정에 있고 확인된 이메일이다.
    OnAccount,
    /// 계정에 있지만 확인되지 않았다.
    Unverified,
    /// 계정에 없다.
    NotOnAccount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmailCheckView {
    pub state: EmailCheckState,
}

/// 내 GitHub 저장소 목록 한 쪽(복제 대화상자, D-133). 이름·공개 여부·날짜뿐이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoListView {
    pub items: Vec<GitHubRepoRowView>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoRowView {
    pub owner: String,
    pub name: String,
    pub private: bool,
    /// 마지막으로 올린 시각(unix 초).
    #[specta(type = Option<u32>)]
    pub pushed_at: Option<i64>,
    /// 소유자가 조직이다(D-184 "만들 곳" 후보).
    pub owner_is_org: bool,
}

/// 로그인 시작: 사람이 브라우저에 넣을 코드.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LoginChallengeView {
    pub user_code: String,
    pub expires_in_secs: u32,
    pub interval_secs: u32,
}

/// 한 번 물어본 결과.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LoginPollView {
    #[serde(rename_all = "camelCase")]
    Pending {
        interval_secs: u32,
    },
    Connected {
        login: String,
    },
    Expired,
    Denied,
}

/// 앱이 기본 브라우저로 여는 GitHub 페이지. 주소는 core가 상수와 검사한 값으로만 만든다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HostingPage {
    /// 코드를 넣는 페이지.
    DeviceLogin,
    /// 앱 권한 페이지(GitHub 쪽 연결 취소).
    AppSettings,
    /// 열린 저장소의 검토 요청 하나.
    PullRequest { number: u32 },
    /// 열린 저장소의 GitHub 페이지(GitHub에 올리기 뒤, D-132).
    Repository,
    /// 열린 저장소의 브랜치 보호 규칙 설정(서버가 올리기를 거절했을 때, D-183).
    BranchSettings,
}

/// 복제 대화상자의 기본값(D-115). 부모 폴더는 홈 폴더.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CloneDefaultsView {
    /// 복제할 폴더를 만들 곳. 홈 폴더를 모르면 `None`(화면이 고르게 한다).
    pub parent: Option<String>,
}
