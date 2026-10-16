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

/// 원격 하나에서 git이 실제로 쓸 주소들(`insteadOf`·`pushInsteadOf`·`pushurl`을 적용한 값).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemoteUrls {
    pub fetch: Vec<String>,
    pub push: Vec<String>,
}

/// 원격 호출의 인증 방식(D-111). core가 정하고 화면 값이 아니다.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AuthMode {
    /// 사용자 설정 그대로(GCM, 사용자 helper, SSH 키).
    #[default]
    UserDefault,
    /// 이 호출에서만 사용자 helper를 비우고 hiGit helper(앱 실행 파일 `--git-credential`)를 쓴다.
    /// helper 경로가 없으면 `UserDefault`처럼 돈다.
    Helper,
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

/// 세 쪽을 합친 결과의 한 구간. 줄 끝(`\r\n`)까지 그대로 담아 이어 붙이면 원본과 같다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeSegment {
    /// 겹치지 않아 저절로 합쳐진 줄들.
    Same(Vec<u8>),
    /// 두 쪽이 같은 곳을 다르게 고친 곳.
    Conflict {
        ours: Vec<u8>,
        base: Vec<u8>,
        theirs: Vec<u8>,
    },
}

/// HEAD reflog 한 줄(작업 내역, D-085).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflogEntry {
    pub oid: Oid,
    /// 그 일이 있은 시각(unix 초).
    pub at: i64,
    /// git이 적은 일(`commit: 이름`, `checkout: moving from a to b`). 원문이라 화면은 접어 둔다.
    pub subject: String,
    /// 그 버전의 메시지 첫 줄.
    pub summary: String,
}

/// stash 목록 한 줄(`refs/stash`의 reflog).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StashEntry {
    pub oid: Oid,
    /// reflog 제목(`On main: 메시지`, `WIP on main: abc 이름`).
    pub subject: String,
    /// 만든 시각(unix 초).
    pub at: i64,
}

/// ref 하나(이름과 가리키는 해시).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefEntry {
    pub name: String,
    pub oid: String,
    /// 가리키는 버전의 시각(unix 초).
    pub time: i64,
    /// 가리키는 버전의 메시지 첫 줄.
    pub summary: String,
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

/// 여러 끝점에서 위상 순서로 읽는 기록 조건(노선도, D-088). 끝점과 제외는 해시만 받아
/// 이름이 명령줄에 닿지 않는다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphLogQuery {
    /// 여기서 시작한다(지금 위치, 브랜치 끝). 비면 빈 목록.
    pub tips: Vec<Oid>,
    /// 이 버전들과 그 조상은 뺀다(예: upstream으로 안 올린 버전만).
    pub exclude: Vec<Oid>,
    pub skip: u32,
    pub limit: u32,
}

/// 버전 하나에서 바뀐 파일 수와 줄 수(노선도 선 굵기, D-089). 이진 파일은 줄 0.
/// 합치기 버전은 바뀐 줄을 재지 않아 모두 0이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitStat {
    pub oid: Oid,
    pub files: u32,
    pub added: u32,
    pub removed: u32,
}

/// 기록 정리의 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteStep {
    pub oid: Oid,
    pub kind: RewriteKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteKind {
    Pick,
    /// 바로 위 줄의 버전에 합친다(메시지는 위 줄 것).
    Fixup,
    /// 놓은 뒤 멈춘다(core가 메시지를 고치고 이어간다).
    Edit,
    Drop,
}

/// 멈춘 기록 정리의 상태(`<git 폴더>/rebase-merge/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseState {
    /// 정리하는 브랜치(`refs/heads/…`).
    pub head_name: String,
    pub onto: Option<Oid>,
    pub orig_head: Option<Oid>,
    /// `edit` 줄에서 멈췄다(충돌이 아님).
    pub stopped_at_edit: bool,
    /// 남은 할 일 줄(주석·빈 줄 제외).
    pub todo: Vec<String>,
    /// 한 줄.
    pub done: Vec<String>,
}

/// 멈출 수 있는 작업의 종류(D-101).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    CherryPick,
    Revert,
    Rebase,
}

impl OperationKind {
    pub(crate) fn command(self) -> &'static str {
        match self {
            OperationKind::CherryPick => "cherry-pick",
            OperationKind::Revert => "revert",
            OperationKind::Rebase => "rebase",
        }
    }
}

/// 버전 하나의 전체 메시지(마지막 버전 고치기·기록 정리).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMessage {
    pub oid: Oid,
    pub message: String,
}

/// 버전을 가리키는 태그. 주석 태그는 가리키는 버전으로 한 단계 벗긴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRef {
    /// `refs/tags/` 뒤의 이름.
    pub name: String,
    pub commit: Oid,
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
    /// 고른 파일의 고친 내용을 stash로 만들며 작업 폴더·index에서 버린다(고친 것 버리기, D-074).
    /// 새 파일은 지워진다. 만든 해시는 [`OpOutput::oid`], 바뀐 것이 없으면 `None`.
    DiscardPaths { paths: Vec<String>, message: String },
    /// stash 목록 맨 위가 이 해시일 때만 뺀다.
    StashDropIf { oid: Oid },
    /// stash 목록에서 이 해시인 항목을 뺀다(위치를 찾고 빼기 직전에 다시 확인한다).
    StashDropOid { oid: Oid },
    /// 맡긴 파일을 꺼낸다. 작업 폴더가 깨끗해야 하고, 겹치면 깨끗하게 되돌린 뒤 `UnparkConflict`.
    Unpark { oid: Oid },
    /// 맡긴 파일을 꺼내되 겹친 곳은 충돌로 남긴다(`stash apply`, 충돌 해결 화면용, D-083).
    /// 작업 폴더가 깨끗해야 하고, 맡길 때 새 파일이던 것이 이미 있으면 `UnparkConflict`.
    /// 겹쳤으면 [`OpOutput::stdout`]이 아니라 index의 충돌 항목으로 알 수 있다.
    UnparkKeep { oid: Oid },
    /// `UnparkKeep`으로 남긴 상태를 꺼내기 전으로 되돌린다. 바뀐 파일이 모두 그 stash의 경로일 때만.
    UnparkAbort { oid: Oid },
    /// 원격의 새 버전을 받아 온다(내 파일은 그대로). `remote`가 없으면 모든 원격. prune은 하지 않는다.
    Fetch {
        remote: Option<String>,
        background: bool,
        auth: AuthMode,
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
    /// 충돌한 파일 하나를 고른 내용(`bytes`, index 형식)으로 해결한다. 블롭을 만들어 index에
    /// 넣고(stage 1·2·3을 지움) 작업 폴더 파일을 사용자의 줄바꿈 설정대로 다시 쓴다.
    /// `mode`는 `100644`·`100755`·`120000`만 받는다.
    ResolveFile {
        path: String,
        mode: String,
        bytes: Vec<u8>,
    },
    /// 충돌한 파일을 지우는 쪽으로 해결한다(`rm -f`).
    ResolveDelete { path: String },
    /// 충돌한 파일을 한쪽 그대로 해결한다(`checkout --ours`/`--theirs` 뒤 `add`). 이진·링크·큰 파일용.
    ResolveSide { path: String, ours: bool },
    /// 작업 폴더의 `paths` 파일 내용을 stash 모양 커밋(부모: HEAD, index 커밋)으로 만든다.
    /// index·작업 폴더·stash 목록은 그대로다. 만든 해시는 [`OpOutput::oid`]. 덮어쓰기 전 백업용.
    SnapshotFiles { paths: Vec<String>, message: String },
    /// `oid`의 변경을 취소하는 새 버전을 만든다(`revert --no-edit`, 합치기 버전은 `-m`).
    /// 실패하면 반쯤 된 되돌리기를 그만두고(`revert --abort`, 안 되면 `reset --merge`) 오류를 돌려준다.
    /// 정리가 넣은 파일을 되돌리므로 core는 넣은 파일이 없을 때만 부른다.
    Revert { oid: Oid, mainline: Option<u32> },
    /// 마지막 버전을 고친다(`commit --amend`, 넣은 파일도 함께). 메시지가 있으면 stdin으로
    /// (`--cleanup=whitespace`라 `#` 줄도 남는다), 없으면 메시지는 그대로(`--no-edit`).
    Amend { message: Option<String> },
    /// `oid` 하나를 지금 브랜치에 새 버전으로 복사한다(`cherry-pick`). 충돌·hook으로 멈추면 정리하지
    /// 않는다(core가 상태를 보고 충돌 화면으로 넘기거나 그만둔다).
    CherryPickStart { oid: Oid },
    /// `oid`의 변경을 취소하는 새 버전(`revert --no-edit`, 합치기 버전은 `-m`). 멈추면 정리하지 않는다.
    /// (올린 뒤 Ctrl+Z가 쓰는 `Revert`는 실패하면 정리한다.)
    RevertStart { oid: Oid, mainline: Option<u32> },
    /// 기록 정리를 시작한다(`rebase -i`, D-100). `base` 위로 `steps`를 차례로 놓는다. 끝까지 가거나
    /// `edit`·충돌·hook으로 멈춘다(멈춤은 `rebase_state`로 본다).
    RewriteHistory { base: Oid, steps: Vec<RewriteStep> },
    /// 가벼운 태그를 만든다(`tag <이름> <해시>`, 서명 설정을 끈다, D-104).
    TagCreate { name: String, oid: Oid },
    /// 태그를 지운다(`tag -d`).
    TagDelete { name: String },
    /// 로컬 브랜치를 지운다(`branch -D`, 설정 구역도 지운다, D-103).
    BranchDelete { name: String },
    /// 로컬 브랜치 이름을 바꾼다(`branch -m`, 대소문자만 다르면 `force`로 `-M`).
    BranchRename {
        from: String,
        to: String,
        force: bool,
    },
    /// 연결된 원격 브랜치를 끊는다. 이미 없으면 성공으로 본다.
    UnsetUpstream { branch: String },
    /// 원격 브랜치를 잇는다(`branch --set-upstream-to`).
    SetUpstream { branch: String, upstream: String },
    /// `oid`에서 새 브랜치를 만들고 갈아탄다(`switch --no-guess -c <이름> <해시>`).
    CreateAndSwitchAt { name: String, oid: Oid },
    /// 멈춘 복사·되돌리기·기록 정리를 이어간다(`--continue`). core가 표식을 확인한 뒤에만 부른다.
    OperationContinue { kind: OperationKind },
    /// 지금 멈춘 버전을 건너뛴다(`--skip`, 해결 결과가 비었을 때).
    OperationSkip { kind: OperationKind },
    /// 시작 전으로 돌린다(`--abort`).
    OperationAbort { kind: OperationKind },
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
        auth: AuthMode,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
    /// 추적 중인 파일: 마지막 저장과 지금(넣은 것 + 안 넣은 것)을 비교한다.
    Tracked,
    /// 아직 추적하지 않는 새 파일: 빈 파일과 비교한다.
    Untracked,
    /// 그 버전에서 바뀐 것: 첫 부모(첫 버전이면 빈 트리)와 그 버전을 비교한다(D-105).
    Commit { oid: Oid },
}

/// 버전 하나에서 바뀐 파일(D-105).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitFile {
    pub path: String,
    /// 이름 바꿈·복사의 원래 경로.
    pub orig_path: Option<String>,
    pub change: Change,
}

/// 버전의 바뀐 파일 목록. 500개까지 담고 `total`은 전체 수.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitFiles {
    pub files: Vec<CommitFile>,
    pub total: u32,
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
