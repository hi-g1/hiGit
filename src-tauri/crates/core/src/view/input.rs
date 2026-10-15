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
    /// 제외할 파일에 추가(새 파일만, 200개까지).
    Ignore(PathsInput),
    /// 다른 브랜치를 지금 브랜치로 가져와 합친다(D-069).
    Merge(MergeInput),
    /// 지금 버전으로 기본 브랜치를 앞으로 감는다("main에 합치기", D-071).
    MergeInto(MergeIntoInput),
    /// 충돌 해결 화면에서 고른 것으로 합치기를 마친다(D-069, D-070).
    ConflictFinish(ConflictFinishInput),
    /// 합치기를 그만둔다(D-070).
    ConflictAbort,
    Switch(SwitchInput),
    BranchCreate(BranchCreateInput),
    Fetch(FetchInput),
    Pull,
    Push,
    Sync(SyncInput),
}

impl ActionInput {
    /// 이 입력이 가리키는 액션 id. serde 태그와 같아야 한다(테스트로 확인).
    pub fn action_id(&self) -> &'static str {
        match self {
            ActionInput::Init => "init",
            ActionInput::Stage(_) => "stage",
            ActionInput::Unstage(_) => "unstage",
            ActionInput::Commit(_) => "commit",
            ActionInput::Ignore(_) => "ignore",
            ActionInput::Merge(_) => "merge",
            ActionInput::MergeInto(_) => "merge-into",
            ActionInput::ConflictFinish(_) => "conflict-finish",
            ActionInput::ConflictAbort => "conflict-abort",
            ActionInput::Switch(_) => "switch",
            ActionInput::BranchCreate(_) => "branch-create",
            ActionInput::Fetch(_) => "fetch",
            ActionInput::Pull => "pull",
            ActionInput::Push => "push",
            ActionInput::Sync(_) => "sync",
        }
    }

    /// 작업 목록(journal)에 남길 사본. 줄 하나가 커지지 않게 긴 경로 목록은 앞부분만 남긴다.
    /// 되돌리기·다시 하기는 입력이 아니라 전후 상태로 하므로 잘라도 된다(D-072).
    pub(crate) fn journal_copy(&self) -> ActionInput {
        const MAX_PATHS: usize = 200;
        let cut = |p: &PathsInput| PathsInput {
            paths: p.paths.iter().take(MAX_PATHS).cloned().collect(),
        };
        match self {
            ActionInput::Stage(p) => ActionInput::Stage(cut(p)),
            ActionInput::Unstage(p) => ActionInput::Unstage(cut(p)),
            // 직접 고친 글은 남기지 않고 선택 종류만 남긴다.
            ActionInput::ConflictFinish(i) => ActionInput::ConflictFinish(ConflictFinishInput {
                resolutions: i
                    .resolutions
                    .iter()
                    .take(MAX_PATHS)
                    .map(|r| FileResolution {
                        path: r.path.clone(),
                        choice: match &r.choice {
                            FileChoice::Hunks { choices } => FileChoice::Hunks {
                                choices: choices
                                    .iter()
                                    .map(|c| match c {
                                        HunkChoice::Custom { .. } => HunkChoice::Custom {
                                            text: String::new(),
                                        },
                                        other => other.clone(),
                                    })
                                    .collect(),
                            },
                            other => other.clone(),
                        },
                    })
                    .collect(),
                locale: i.locale.clone(),
            }),
            other => other.clone(),
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FetchInput {
    /// 저절로 하는 원격 확인이면 true. 질문 창을 띄우지 않고 60초 안에 끝낸다(D-045, D-055).
    pub background: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MergeInput {
    /// 가져올 로컬 브랜치 이름.
    pub branch: String,
    /// 합친 버전 메시지에 쓸 앱 언어("ko", "en").
    pub locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MergeIntoInput {
    /// 앞으로 감을 브랜치(기본 브랜치).
    pub branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFinishInput {
    /// 남은 충돌 파일마다 하나씩.
    pub resolutions: Vec<FileResolution>,
    /// 합친 버전 메시지에 쓸 앱 언어("ko", "en").
    pub locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileResolution {
    pub path: String,
    pub choice: FileChoice,
}

/// 파일 하나의 선택. 텍스트 파일은 곳마다(`hunks`), 그 밖은 한쪽 그대로.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FileChoice {
    Hunks { choices: Vec<HunkChoice> },
    Ours,
    Theirs,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HunkChoice {
    Ours,
    Theirs,
    /// 내 쪽 다음 합칠 쪽.
    Both,
    /// 직접 고친 글. 표시 문자 없는 글상자의 내용.
    Custom {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncInput {
    /// 합친 버전 메시지에 쓸 앱 언어("ko", "en").
    pub locale: String,
}
