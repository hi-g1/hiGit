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
