//! 작업 내역(D-045, D-085). 앱 작업·reflog·백업·맡긴 파일·임시 보관을 시간순으로.

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RecoverySource {
    /// 앱 작업 목록(journal).
    Journal,
    /// git의 HEAD 기록(앱 밖 작업 포함).
    Reflog,
    /// 앱이 남긴 백업 ref.
    Backup,
    /// 브랜치에 맡겨 둔 파일.
    Parked,
    /// 임시 보관.
    Stash,
}

/// 앱 작업 줄에 Ctrl+Z가 닿는지(D-204).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum UndoHint {
    /// 다음 Ctrl+Z가 되돌린다.
    Next,
    /// 위의 더 새 작업을 먼저 되돌려야 한다.
    Earlier,
    /// 되돌렸다.
    Done,
    /// Ctrl+Z로 되돌릴 수 없다(경계 앞, 올리기, 앱 밖에서 바뀜).
    Never,
    /// 앱 작업 줄이 아니다.
    None,
    /// 다음 Ctrl+Z가 되돌리는 새 버전을 만든다(올린 버전, D-073).
    RevertCommit,
}

/// 되살리는 방법. `Version`은 "이 시점으로 되돌리기", `Changes`는 "꺼내기".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RestoreKind {
    None,
    Version,
    Changes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryItem {
    /// 목록 안에서만 쓰는 구분 값.
    pub id: String,
    /// 그 일이 있은 시각(unix 초).
    #[specta(type = u32)]
    pub at: i64,
    pub source: RecoverySource,
    /// 앱 작업이면 액션 id(화면이 용어로 보여준다).
    pub action: Option<String>,
    /// 원천별 이름: 백업 이름표, 브랜치 이름, 보관 메시지, reflog 원문.
    pub label: String,
    /// 그 버전의 메시지 첫 줄.
    pub summary: String,
    /// 되살리기·꺼내기에 그대로 돌려보내는 값(버전 해시).
    pub oid: Option<String>,
    /// 꺼내기에 그대로 돌려보내는 ref 이름(백업·맡긴 파일).
    pub refname: Option<String>,
    pub restore: RestoreKind,
    /// 앱 작업 중 되돌린 것.
    pub undone: bool,
    /// Ctrl+Z가 닿는지(D-204).
    pub undo: UndoHint,
    /// 앱 작업 줄의 실행 전 버전. "이 작업 전으로 되돌리기"가 `restore`로 보낸다. 지금 브랜치에서 HEAD를 옮긴
    /// 줄이고 그 버전이 Git 기록에 있을 때만 값이 있다.
    pub before_oid: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryView {
    pub items: Vec<RecoveryItem>,
}
