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
    /// 원격 저장소를 부모 폴더(실행 폴더) 아래 새 폴더로 복제한다(D-115).
    Clone(CloneInput),
    /// 올라가 있는 지금 브랜치로 새 검토 요청을 만든다(D-113).
    PrCreate(PrCreateInput),
    /// 전역 이름·이메일을 저장한다(D-125). 저장소가 필요 없다.
    IdentitySet(IdentityInput),
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
    /// 고친 파일을 모두 임시 보관한다(D-076).
    Stash,
    /// 보관해 둔 것을 꺼낸다.
    TakeOut(TakeOutInput),
    /// 고른 파일의 고친 내용을 버린다(확인 필요, D-074).
    Discard(PathsInput),
    /// 작업 내역의 버전으로 지금 브랜치를 옮긴다(D-085).
    Restore(RestoreInput),
    /// 버려진 잠금 파일을 지운다(확인 필요, D-081).
    ClearLock,
    /// stash 목록에 남은 맡긴 항목을 맡긴 ref로 옮긴다(D-082).
    RecoverParked,
    Switch(SwitchInput),
    BranchCreate(BranchCreateInput),
    Fetch(FetchInput),
    Pull,
    Push,
    Sync(SyncInput),
    /// 지금 위치의 버전을 메시지·넣은 파일로 고친다(D-097).
    Amend(AmendInput),
    /// 고른 버전(지금 위치의 조상)의 변경을 취소하는 새 버전을 만든다.
    Revert(OidInput),
    /// 다른 브랜치의 버전 하나를 지금 브랜치로 복사한다.
    CherryPick(OidInput),
    /// 지금 브랜치를 고른 옛 버전(지금 위치의 조상)으로 옮긴다(안 올린 버전만 빠짐).
    Reset(OidInput),
    /// 기록 정리: 고른 구간의 순서 바꾸기·묶기·빼기·메시지 고치기(D-100, D-106).
    RebaseI(RewriteInput),
    /// 고른 버전에 가벼운 태그를 단다(D-104).
    Tag(TagInput),
    /// 태그를 지운다(가벼운 태그만).
    TagDelete(TagDeleteInput),
    /// 로컬 브랜치를 지운다(D-103).
    BranchDelete(BranchNameInput),
    /// 로컬 브랜치 이름을 바꾼다(D-103).
    BranchRename(BranchRenameInput),
    /// 덮어쓰며 올리기(D-128). 되돌릴 수 없어 확인 토큰이 이 입력 전체에 묶인다.
    ForcePush(ForcePushInput),
    /// 원격 브랜치 지우기(D-130). 되돌릴 수 없어 확인 토큰이 입력에 묶인다.
    RemoteBranchDelete(RemoteBranchInput),
    /// 원격 브랜치 이름 바꾸기(D-130).
    RemoteBranchRename(RemoteBranchRenameInput),
    /// 태그 하나를 원격에 올린다(D-131). 덧붙이기만 해 확인이 없다.
    TagPush(TagPushInput),
    /// GitHub에 새 저장소를 만들고 `origin`을 잇는다(D-132). 올리기는 화면이 이어서 `push`.
    Publish(PublishInput),
}

impl ActionInput {
    /// 이 입력이 가리키는 액션 id. serde 태그와 같아야 한다(테스트로 확인).
    pub fn action_id(&self) -> &'static str {
        match self {
            ActionInput::Init => "init",
            ActionInput::Clone(_) => "clone",
            ActionInput::PrCreate(_) => "pr-create",
            ActionInput::IdentitySet(_) => "identity-set",
            ActionInput::Stage(_) => "stage",
            ActionInput::Unstage(_) => "unstage",
            ActionInput::Commit(_) => "commit",
            ActionInput::Ignore(_) => "ignore",
            ActionInput::Merge(_) => "merge",
            ActionInput::MergeInto(_) => "merge-into",
            ActionInput::ConflictFinish(_) => "conflict-finish",
            ActionInput::ConflictAbort => "conflict-abort",
            ActionInput::Stash => "stash",
            ActionInput::TakeOut(_) => "take-out",
            ActionInput::Discard(_) => "discard",
            ActionInput::Restore(_) => "restore",
            ActionInput::ClearLock => "clear-lock",
            ActionInput::RecoverParked => "recover-parked",
            ActionInput::Switch(_) => "switch",
            ActionInput::BranchCreate(_) => "branch-create",
            ActionInput::Fetch(_) => "fetch",
            ActionInput::Pull => "pull",
            ActionInput::Push => "push",
            ActionInput::Sync(_) => "sync",
            ActionInput::Amend(_) => "amend",
            ActionInput::Revert(_) => "revert",
            ActionInput::CherryPick(_) => "cherry-pick",
            ActionInput::Reset(_) => "reset",
            ActionInput::RebaseI(_) => "rebase-i",
            ActionInput::Tag(_) => "tag",
            ActionInput::TagDelete(_) => "tag-delete",
            ActionInput::BranchDelete(_) => "branch-delete",
            ActionInput::BranchRename(_) => "branch-rename",
            ActionInput::ForcePush(_) => "force-push",
            ActionInput::RemoteBranchDelete(_) => "remote-branch-delete",
            ActionInput::RemoteBranchRename(_) => "remote-branch-rename",
            ActionInput::TagPush(_) => "tag-push",
            ActionInput::Publish(_) => "publish",
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
            // 이름·이메일은 어디에도 남기지 않는다(D-125).
            ActionInput::IdentitySet(_) => ActionInput::IdentitySet(IdentityInput::default()),
            ActionInput::Stage(p) => ActionInput::Stage(cut(p)),
            ActionInput::Unstage(p) => ActionInput::Unstage(cut(p)),
            ActionInput::Discard(p) => ActionInput::Discard(cut(p)),
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
                locale: journal_locale(&i.locale),
            }),
            // 화면이 준 언어 글자는 길이 제한이 없다. 줄에는 쓰이는 두 값만 남긴다.
            ActionInput::Merge(i) => ActionInput::Merge(MergeInput {
                branch: i.branch.clone(),
                locale: journal_locale(&i.locale),
            }),
            ActionInput::Sync(i) => ActionInput::Sync(SyncInput {
                locale: journal_locale(&i.locale),
                merge_anyway: i.merge_anyway,
            }),
            ActionInput::RebaseI(i) => ActionInput::RebaseI(RewriteInput {
                steps: i
                    .steps
                    .iter()
                    .take(MAX_PATHS)
                    .map(|s| RewriteStepInput {
                        oid: s.oid.clone(),
                        kind: s.kind,
                        message: s.message.as_ref().map(|_| String::new()),
                    })
                    .collect(),
            }),
            // 메시지(최대 1MB)는 줄에 남기지 않는다. 되돌리기는 `before`/`after`로 한다.
            ActionInput::Amend(i) => ActionInput::Amend(AmendInput {
                expect_head: i.expect_head.clone(),
                message: i.message.as_ref().map(|_| String::new()),
            }),
            other => other.clone(),
        }
    }
}

/// 합친 버전 이름의 언어는 `ko`로 시작하는지만 본다(그 밖은 영어).
fn journal_locale(locale: &str) -> String {
    if locale.starts_with("ko") { "ko" } else { "en" }.to_string()
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
    /// 시작할 버전(노선도의 역, 작업 내역). 없으면 지금 버전에서 만들고 고친 파일이 따라간다.
    #[serde(default)]
    pub from: Option<String>,
}

/// 새 검토 요청 입력. head는 core가 지금 브랜치의 upstream으로 정한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrCreateInput {
    pub title: String,
    pub body: String,
    /// 합칠 대상 원격 브랜치 이름.
    pub base: String,
    pub draft: bool,
}

/// 전역 이름·이메일(D-125). core가 앞뒤 공백을 빼고 다시 검사한다.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IdentityInput {
    pub name: String,
    pub email: String,
}

/// 복제 입력. 주소와 폴더 이름은 core가 검사한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CloneInput {
    pub url: String,
    pub name: String,
    /// git이 보안 수정 기준보다 낮은 것을 알고 진행한다.
    pub accept_old_git: bool,
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
pub struct TakeOutInput {
    pub source: TakeOutSource,
}

/// 꺼낼 곳. 화면은 조회가 준 값만 보낸다(core가 다시 확인한다).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TakeOutSource {
    /// 임시 보관 목록의 항목.
    Stash { oid: String },
    /// 지금 브랜치에 맡겨 둔 파일. 겹치면 충돌 해결 화면으로 간다(D-083).
    Parked { branch: String },
    /// 작업 내역의 백업 ref(stash 모양)나 맡긴 ref(D-085).
    Ref { refname: String },
}

/// 기록 정리 입력. `steps`는 적용 순서(아래 = 옛 것부터)이고, 해시 집합은 고른 구간과 같아야 한다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RewriteInput {
    pub steps: Vec<RewriteStepInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RewriteStepInput {
    pub oid: String,
    pub kind: RewriteKindInput,
    /// 고친 메시지. 묶음의 맨 아래(`pick`) 카드에만 둔다.
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RewriteKindInput {
    Pick,
    /// 바로 아래 단계의 버전에 합친다.
    Squash,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchNameInput {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchRenameInput {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagInput {
    pub oid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagDeleteInput {
    pub name: String,
    /// 화면이 본, 태그가 가리키는 버전.
    pub oid: String,
}

/// 노선도에서 고른 버전 해시. core가 다시 확인한다(D-099).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OidInput {
    pub oid: String,
}

/// 새 저장소의 공개 여부(D-132). 기본값이 없어 화면이 꼭 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Private,
    Public,
}

/// GitHub에 올리기 입력(D-132). 공개면 `acknowledge_public`이 있어야 받는다(대화상자 안의 한 번 더 확인).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PublishInput {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Visibility,
    #[serde(default)]
    pub acknowledge_public: bool,
}

/// 태그 올리기 입력(D-131).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagPushInput {
    pub name: String,
}

/// 원격 브랜치 지우기 입력(D-130). 확인 창이 본 추적 ref 값(`expect_tip`)일 때만 지운다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchInput {
    pub remote: String,
    pub branch: String,
    pub expect_tip: String,
}

/// 원격 브랜치 이름 바꾸기 입력(D-130).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchRenameInput {
    pub remote: String,
    pub from: String,
    pub to: String,
    pub expect_tip: String,
}

/// 덮어쓰며 올리기 입력(D-128, 계획 010 R4). 이름은 화면이 본 upstream이고 core가 upstream에서 다시 구해
/// 같은지 비교한다(토큰이 이름까지 묶는다). 해시는 확인 창이 보여 준 값이다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ForcePushInput {
    /// upstream 원격 이름(`origin`).
    pub remote: String,
    /// upstream의 원격 브랜치 이름(`기능`).
    pub branch: String,
    /// 확인 창이 보여 준 원격 끝(lease 기대 값). 그사이 원격이 바뀌면 서버가 거절한다.
    pub expect_remote: String,
    /// 확인 창이 본 지금 위치. 이 버전만 올라간다.
    pub expect_head: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AmendInput {
    /// 화면이 본 지금 위치의 해시. 그사이 바뀌었으면 거절한다.
    pub expect_head: String,
    /// 새 메시지. 없으면 메시지는 그대로 두고 넣은 파일만 더한다.
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RestoreInput {
    /// 작업 내역이 준 버전 해시.
    pub oid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncInput {
    /// 합친 버전 메시지에 쓸 앱 언어("ko", "en").
    pub locale: String,
    /// "그래도 합치기"(D-129): 올린 기록을 바꾼 표식이 있어도 원격 버전을 받아 합친다. 거짓이면
    /// 직렬화하지 않아 옛 journal 줄의 서명이 그대로 맞는다(계획 010 R6).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[specta(optional)]
    pub merge_anyway: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_input_written_before_merge_anyway_is_the_same_text() {
        // 옛 journal 줄의 서명은 입력 글자에 걸려 있다. 거짓이면 새 칸을 쓰지 않아 글자가 같다(계획 010 R6).
        let old = r#"{"id":"sync","input":{"locale":"ko"}}"#;
        let parsed: ActionInput = serde_json::from_str(old).unwrap();
        assert_eq!(
            parsed,
            ActionInput::Sync(SyncInput {
                locale: "ko".into(),
                merge_anyway: false,
            })
        );
        assert_eq!(serde_json::to_string(&parsed).unwrap(), old);
        let anyway = ActionInput::Sync(SyncInput {
            locale: "ko".into(),
            merge_anyway: true,
        });
        assert_eq!(
            serde_json::to_string(&anyway).unwrap(),
            r#"{"id":"sync","input":{"locale":"ko","mergeAnyway":true}}"#
        );
    }

    #[test]
    fn journal_copy_keeps_only_the_two_locales() {
        let long = "ko-KR".to_string() + &"x".repeat(10_000);
        let copy = ActionInput::Sync(SyncInput {
            locale: long,
            merge_anyway: true,
        })
        .journal_copy();
        assert_eq!(
            copy,
            ActionInput::Sync(SyncInput {
                locale: "ko".into(),
                merge_anyway: true,
            })
        );
        let copy = ActionInput::Merge(MergeInput {
            branch: "main".into(),
            locale: "아무거나".into(),
        })
        .journal_copy();
        assert_eq!(
            copy,
            ActionInput::Merge(MergeInput {
                branch: "main".into(),
                locale: "en".into()
            })
        );
    }
}
