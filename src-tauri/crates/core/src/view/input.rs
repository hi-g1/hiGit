//! 액션 입력(D-020). 액션마다 입력 구조체를 두고, 화면은 태그 붙은 enum으로 보낸다.
//! TS에서는 `{ id: "init" } | { id: "commit"; input: CommitInput }` 같은 판별 유니온이 된다.
//!
//! 새 액션을 추가하면 변형 하나와 [`ActionInput::action_id`]의 줄 하나를 더한다.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(tag = "id", content = "input", rename_all = "kebab-case")]
pub enum ActionInput {
    /// 관리 시작. 입력 없음.
    Init,
    Stage(PathsInput),
    Unstage(PathsInput),
    Commit(CommitInput),
    Switch(SwitchInput),
    BranchCreate(BranchCreateInput),
}

impl ActionInput {
    /// 이 입력이 가리키는 액션 id. serde 태그와 같아야 한다(테스트로 확인).
    pub fn action_id(&self) -> &'static str {
        match self {
            ActionInput::Init => "init",
            ActionInput::Stage(_) => "stage",
            ActionInput::Unstage(_) => "unstage",
            ActionInput::Commit(_) => "commit",
            ActionInput::Switch(_) => "switch",
            ActionInput::BranchCreate(_) => "branch-create",
        }
    }
}

/// 파일 경로 목록. 경로는 바뀐 내용 목록이 준 저장소 기준 경로(구분자 `/`)다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PathsInput {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommitInput {
    pub message: String,
    /// 넣은 파일이 없으면 고친 파일을 모두 넣고 저장한다(D-062).
    #[serde(default)]
    pub stage_all: bool,
}

/// 브랜치 갈아타기. 원격에만 있는 브랜치면 `remote`에 원격 이름을 주고, 같은 이름의 로컬 브랜치를 만들어 연결한다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SwitchInput {
    pub branch: String,
    pub remote: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchCreateInput {
    pub name: String,
}
