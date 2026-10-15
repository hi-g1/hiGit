use std::fmt;
use std::path::{Path, PathBuf};

/// 저장소 작업 폴더의 최상위 경로.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoPath(PathBuf);

impl RepoPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// `git --version`에서 읽은 버전. 뒤에 붙는 `.windows.1` 같은 꼬리는 버린다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GitVersion {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// `git version 2.45.1.windows.1` 같은 한 줄을 읽는다.
    pub fn parse(line: &str) -> Option<Self> {
        let number = line.trim().strip_prefix("git version ")?;
        let mut parts = number
            .split(|c: char| c == '.' || c.is_whitespace())
            .map(|p| p.parse::<u32>().ok());
        let major = parts.next()??;
        let minor = parts.next()??;
        let patch = parts.next().flatten().unwrap_or(0);
        Some(Self::new(major, minor, patch))
    }
}

impl fmt::Display for GitVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// `git status` 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub branch: BranchInfo,
    pub entries: Vec<StatusEntry>,
}

/// 현재 갈래 정보(`# branch.*` 헤더).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BranchInfo {
    /// 현재 저장의 해시. 저장이 하나도 없으면 `None`.
    pub oid: Option<String>,
    /// 현재 갈래 이름. 특정 저장을 직접 보고 있으면(detached) `None`.
    pub head: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

/// 바뀐 파일 하나. 경로 구분자는 OS와 상관없이 `/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub path: String,
    /// 이름 바꿈·복사의 원래 경로.
    pub orig_path: Option<String>,
    /// 저장에 넣은(stage) 쪽 변화.
    pub index: Change,
    /// 작업 폴더 쪽 변화.
    pub worktree: Change,
    pub kind: EntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Unmodified,
    Modified,
    TypeChanged,
    Added,
    Deleted,
    Renamed,
    Copied,
    Unmerged,
    Untracked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Ordinary,
    Renamed,
    Unmerged,
    Untracked,
}

/// 버전(객체) 해시. SHA-1(40자)과 SHA-256(64자) 저장소 모두 받는다.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Oid(String);

impl Oid {
    /// 16진수 40자나 64자만 받는다. 저장소나 journal에서 온 값을 그대로 믿지 않는다.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let ok = (text.len() == 40 || text.len() == 64)
            && text
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
        ok.then(|| Self(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 브랜치 하나(`for-each-ref`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    /// 로컬은 `main`, 원격은 `origin/main`.
    pub name: String,
    /// `refs/heads/main` 같은 전체 이름.
    pub refname: String,
    pub kind: BranchKind,
    pub oid: String,
    /// 지금 브랜치.
    pub is_head: bool,
    /// 연결된 원격 브랜치(`origin/main`). 로컬 브랜치만.
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    /// upstream이 원격에서 사라졌다.
    pub gone: bool,
    /// 마지막 버전의 작성자 이름.
    pub last_author: String,
    /// 마지막 버전의 시각(unix 초, committer 기준).
    pub last_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKind {
    Local,
    Remote { remote: String, branch: String },
}

/// 원격 저장소 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

/// index 항목 하나(`ls-files -s`). 충돌 중이면 같은 경로가 stage 1(원래)·2(내 쪽)·3(합칠 쪽)으로 나온다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub path: String,
    /// `100644`·`100755`·`120000`(링크)·`160000`(서브모듈).
    pub mode: String,
    pub oid: Oid,
    pub stage: u8,
}

/// 블롭의 크기와 앞부분. 너무 크면 앞부분을 읽지 않는다(`head`가 빈 값).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobHead {
    pub oid: Oid,
    pub size: u64,
    pub head: Vec<u8>,
}

/// ref 하나(이름과 가리키는 해시).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefEntry {
    pub name: String,
    pub oid: String,
}

/// 두 버전을 합칠 때 겹치는지(작업 폴더를 건드리지 않고 미리 본 결과).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeCheck {
    Clean,
    /// 두 쪽이 같은 곳을 다르게 바꾼 파일들.
    Overlap {
        paths: Vec<String>,
    },
}

/// 저장(커밋) 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub oid: String,
    pub short_oid: String,
    /// 부모 해시. 첫 저장은 비어 있고, 합치기 저장은 2개 이상.
    pub parents: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    /// 작성 시각(unix 초). 표시 형식은 화면이 정한다.
    pub author_time: i64,
    /// 메시지 첫 줄.
    pub summary: String,
}

/// 저장 기록 조회 조건. 나중에 확대·축소 단계 같은 필드를 더할 수 있게 구조체로 둔다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogQuery {
    pub skip: u32,
    pub limit: u32,
    /// 시작점. `None`이면 HEAD.
    pub rev: Option<String>,
}

impl Default for LogQuery {
    fn default() -> Self {
        Self {
            skip: 0,
            limit: 200,
            rev: None,
        }
    }
}

/// 상태를 바꾸는 git 작업. 새 작업은 변형 하나와 `CliBackend` 매핑 하나를 더한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitOp {
    /// 아직 저장소가 아닌 폴더를 저장소로 만든다. 기본 갈래 이름은 사용자 git 설정을 따른다.
    Init,
    /// 파일을 다음 저장에 넣는다. 지운 파일도 넣는다.
    Stage { paths: Vec<String> },
    /// 넣었던 파일을 다음 저장에서 뺀다. 작업 폴더 파일은 그대로다.
    Unstage { paths: Vec<String> },
    /// 넣은 내용을 메시지와 함께 저장한다. 사용자의 commit hook과 서명 설정을 따른다.
    Commit { message: String },
    /// 지금 버전에서 새 브랜치를 만들고 갈아탄다(`switch -c`). 작업 폴더는 그대로라 고친 파일이 따라간다.
    /// 이름은 core가 `check_branch_name`으로 먼저 검사한다.
    CreateAndSwitch { name: String },
    /// 있는 로컬 브랜치로 갈아탄다. 이름 추측(`--guess`)은 끈다.
    Switch { branch: String },
    /// 원격 브랜치(`origin/x`)에서 같은 이름의 로컬 브랜치를 만들어 연결하고 갈아탄다.
    SwitchTrack {
        local: String,
        remote_branch: String,
    },
    /// 고친 파일을 stash로 만든다. 만든 해시는 [`OpOutput::oid`], 맡길 것이 없으면 `None`.
    /// 사용자의 stash 목록에서는 core가 ref를 쓴 뒤 [`GitOp::StashDropIf`]로 뺀다.
    Park { message: String },
    /// stash 목록 맨 위가 이 해시일 때만 뺀다.
    StashDropIf { oid: Oid },
    /// 맡긴 파일을 꺼낸다. 작업 폴더가 깨끗해야 하고, 겹치면 깨끗하게 되돌린 뒤 `UnparkConflict`.
    Unpark { oid: Oid },
    /// 원격의 새 버전을 받아 온다(내 파일은 그대로). `remote`가 없으면 모든 원격. prune은 하지 않는다.
    Fetch {
        remote: Option<String>,
        background: bool,
    },
    /// `rev`로 앞으로만 감기(`merge --ff-only`). 갈라졌으면 `NotFastForward`.
    MergeFf { rev: String },
    /// `rev`를 합친 버전을 만든다(`--no-ff`, 메시지는 `-m`). 겹치면 `MergeConflict`.
    Merge { rev: String, message: String },
    /// 진행 중인 합치기를 그만둔다. 합치기 중이 아니면 아무것도 하지 않는다.
    MergeAbort,
    /// 고친 파일을 모두 다음 버전에 넣는다(`add -A`, 무시된 파일 제외).
    StageAll,
    /// 브랜치를 `oid`로 되돌린다. index와 작업 폴더는 그대로(`reset --soft`, 버전 저장 되돌리기).
    ResetSoft { oid: Oid },
    /// `oid`의 변경을 취소하는 새 버전을 만든다(`revert --no-edit`, 합치기 버전은 `-m`).
    /// 실패하면 반쯤 된 되돌리기를 그만두고(`revert --abort`, 안 되면 `reset --merge`) 오류를 돌려준다.
    /// 정리가 넣은 파일을 되돌리므로 core는 넣은 파일이 없을 때만 부른다.
    Revert { oid: Oid, mainline: Option<u32> },
    /// 브랜치와 파일을 `oid`로 옮기되 고친 파일은 둔다(`reset --keep`, 합치기·되살리기 되돌리기).
    /// 고친 파일이 옮길 파일과 겹치면 아무것도 바꾸지 않고 `WouldOverwrite`.
    ResetKeep { oid: Oid },
    /// index를 `tree`로 바꾼다. 작업 폴더는 그대로(`read-tree`).
    ReadTree { tree: Oid },
    /// 사용자 전역 설정의 `safe.directory`에 이 폴더 하나를 더한다(사용자가 고를 때만, D-039).
    /// `*`는 쓰지 않는다. `path`는 git이 받는 형식(Windows는 `C:/…`).
    AddSafeDirectory { path: String },
    /// 올린다. `target`이 있으면 `<원격> HEAD:refs/heads/<브랜치>`, `set_upstream`이면 `-u`.
    Push {
        target: Option<(String, String)>,
        set_upstream: bool,
    },
}

/// 쓰기 작업의 출력 원문.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpOutput {
    pub stdout: String,
    pub stderr: String,
    /// 작업이 만든 해시가 있으면(맡겨 두기).
    pub oid: Option<Oid>,
}

/// 비교할 파일. 경로는 저장소 기준, 구분자 `/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffTarget {
    pub path: String,
    /// 이름 바꿈의 원래 경로.
    pub orig_path: Option<String>,
    pub kind: DiffKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    /// 추적 중인 파일: 마지막 저장과 지금(넣은 것 + 안 넣은 것)을 비교한다.
    Tracked,
    /// 아직 추적하지 않는 새 파일: 빈 파일과 비교한다.
    Untracked,
}

/// 파일 하나의 줄 단위 비교.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileDiff {
    pub binary: bool,
    /// 너무 커서 앞부분만 담았다.
    pub truncated: bool,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub old_start: u32,
    pub old_len: u32,
    pub new_start: u32,
    pub new_len: u32,
    /// `@@ … @@` 뒤의 함수 이름 같은 설명. 없으면 빈 문자열.
    pub section: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    /// 줄 내용(앞의 `+`/`-`/공백과 줄 끝의 `\r`은 뺀다).
    pub text: String,
    /// 파일 끝에 줄바꿈이 없는 줄.
    pub no_newline_at_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Context,
    Added,
    Removed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        let cases = [
            ("git version 2.34.1", GitVersion::new(2, 34, 1)),
            ("git version 2.45.1.windows.1\n", GitVersion::new(2, 45, 1)),
            (
                "git version 2.39.3 (Apple Git-145)",
                GitVersion::new(2, 39, 3),
            ),
            ("git version 2.30.0.rc1", GitVersion::new(2, 30, 0)),
            ("git version 3.0", GitVersion::new(3, 0, 0)),
        ];
        for (line, want) in cases {
            assert_eq!(GitVersion::parse(line), Some(want), "{line}");
        }
    }

    #[test]
    fn rejects_other_text() {
        assert_eq!(GitVersion::parse("hello"), None);
        assert_eq!(GitVersion::parse("git version x.y"), None);
    }

    #[test]
    fn oid_accepts_only_lowercase_hex_of_known_lengths() {
        assert!(Oid::parse(&"a".repeat(40)).is_some());
        assert!(Oid::parse(&"0".repeat(64)).is_some());
        assert!(Oid::parse(&format!("{}\n", "b".repeat(40))).is_some());
        for bad in [
            "",
            "abc",
            &"g".repeat(40),
            &"A".repeat(40),
            &"a".repeat(41),
            "-x",
        ] {
            assert!(Oid::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn orders_versions() {
        assert!(GitVersion::new(2, 29, 9) < GitVersion::new(2, 30, 0));
        assert!(GitVersion::new(2, 30, 0) <= GitVersion::new(2, 30, 0));
    }
}
