//! Git 보안 수정 기준 판정(D-036, D-050). 거부 기준(`MIN_GIT_VERSION`)과 따로 둔다.

use git::GitVersion;

/// 계열별로 보안 수정이 들어간 가장 낮은 버전. Git 보안 공지가 나오면 여기만 고친다.
/// 2025년 7월 공지(CVE-2025-48384 등) 기준. 2026년 10월 확인 때 그 뒤 git 공지는 없었다.
pub const GIT_SECURITY_BASELINE: &[GitVersion] = &[
    GitVersion::new(2, 43, 7),
    GitVersion::new(2, 44, 4),
    GitVersion::new(2, 45, 4),
    GitVersion::new(2, 46, 4),
    GitVersion::new(2, 47, 3),
    GitVersion::new(2, 48, 2),
    GitVersion::new(2, 49, 1),
    GitVersion::new(2, 50, 1),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityJudgement {
    Ok,
    Outdated,
    /// 배포판 패키지라 OS 업데이트가 보안 수정을 맡는다. 경고하지 않는다.
    DistroManaged,
}

pub fn judge(version: GitVersion, distro_package: bool) -> SecurityJudgement {
    if distro_package {
        return SecurityJudgement::DistroManaged;
    }
    let same_series = GIT_SECURITY_BASELINE
        .iter()
        .find(|b| b.major == version.major && b.minor == version.minor);
    let ok = match same_series {
        Some(base) => version >= *base,
        None => GIT_SECURITY_BASELINE
            .last()
            .is_some_and(|newest| version > *newest),
    };
    if ok {
        SecurityJudgement::Ok
    } else {
        SecurityJudgement::Outdated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(a: u32, b: u32, c: u32) -> GitVersion {
        GitVersion::new(a, b, c)
    }

    #[test]
    fn table() {
        use SecurityJudgement::*;
        let cases = [
            (v(2, 34, 1), Outdated),
            (v(2, 20, 0), Outdated),
            (v(2, 43, 6), Outdated),
            (v(2, 43, 7), Ok),
            (v(2, 50, 0), Outdated),
            (v(2, 50, 1), Ok),
            (v(2, 51, 0), Ok),
            (v(2, 60, 0), Ok),
            (v(3, 0, 0), Ok),
        ];
        for (version, want) in cases {
            assert_eq!(judge(version, false), want, "{version}");
        }
        assert_eq!(judge(v(2, 34, 1), true), DistroManaged);
    }
}
