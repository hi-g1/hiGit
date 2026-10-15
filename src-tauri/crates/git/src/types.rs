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
    fn orders_versions() {
        assert!(GitVersion::new(2, 29, 9) < GitVersion::new(2, 30, 0));
        assert!(GitVersion::new(2, 30, 0) <= GitVersion::new(2, 30, 0));
    }
}
