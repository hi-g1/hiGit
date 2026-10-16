//! 브랜치 목록 조회의 화면 타입.

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchListView {
    pub local: Vec<BranchRow>,
    /// 원격에만 있는 것도 포함한 원격 브랜치(`origin/main`).
    pub remote: Vec<BranchRow>,
    /// 기본 브랜치 이름(원격 HEAD, 없으면 main·master 순, D-048).
    pub default_branch: Option<String>,
    /// 앱 밖에서 브랜치를 지워 맡긴 파일만 남은 브랜치 이름(D-054).
    pub orphan_parked: Vec<String>,
    /// 원격 이름. 비어 있으면 동기화 버튼을 숨긴다(D-045).
    pub remotes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchRow {
    /// 로컬은 `main`, 원격은 `origin/main`.
    pub name: String,
    /// 원격 브랜치면 원격 이름.
    pub remote: Option<String>,
    pub is_head: bool,
    /// 기본 브랜치(로컬은 기본 브랜치, 원격 줄은 그 원격의 HEAD와 `main`·`master`·`init.defaultBranch`). 지우기를 흐린다.
    pub is_default: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub gone: bool,
    pub last_author: String,
    /// 마지막 버전 시각(unix 초). TS에서는 number(CommitRow.author_time과 같은 이유).
    #[specta(type = u32)]
    pub last_at: i64,
    /// 이 브랜치에 맡겨 둔 파일 수. 없으면 0.
    pub parked_count: u32,
    /// 맡겨 둔 파일이 있는데 지금 이 브랜치에 있다(겹쳐서 꺼내지 못함, D-054).
    pub park_blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchNameCheck {
    pub ok: bool,
    pub problem: Option<NameProblem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NameProblem {
    Empty,
    /// git 규칙에 맞지 않거나 `-`로 시작한다.
    Invalid,
    /// 같은 이름(대소문자 무시)의 로컬 브랜치가 있다.
    Exists,
}
