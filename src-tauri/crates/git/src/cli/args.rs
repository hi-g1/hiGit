//! 리비전 인자 앞의 `--end-of-options`(D-035).
//!
//! `-`로 시작하는 브랜치 이름이 git 옵션(`--output=…`, `--upload-pack=…`)으로 읽히지 않게 한다.
//! 이 표시는 명령과 git 버전에 따라 동작이 달라, 직접 확인한 조합에서만 넣는다.
//! 넣지 못하는 조합은 core의 이름 검사가 막는다.

use crate::GitVersion;

/// 리비전을 받는 git 명령.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // 브랜치·원격 단계에서 하나씩 쓴다.
pub(crate) enum Cmd {
    RevParse,
    Log,
    Diff,
    Switch,
    Branch,
    Push,
    Merge,
    MergeBase,
    RevList,
    UpdateRef,
    CatFile,
    ShowRef,
    ForEachRef,
    SymbolicRef,
    Fetch,
    LsRemote,
    Stash,
    ReadTree,
    MergeTree,
    LsTree,
    CommitTree,
    Show,
    DiffTree,
    CherryPick,
    Revert,
    Tag,
    RemoteGetUrl,
    RemoteAdd,
    Clone,
    /// 2.34.1에서 `--end-of-options`를 경로로 읽는다.
    Checkout,
    /// 2.34.1에서 옵션 위치 오류를 낸다.
    Reset,
    /// 2.34.1에서 usage 오류를 낸다.
    CheckRefFormat,
}

/// `--end-of-options`가 제대로 동작함을 확인한 가장 낮은 git 버전(이 개발 PC의 Ubuntu 22.04 기본 git).
/// 그보다 낮은 버전(2.30~2.33)은 확인하지 못했으므로 넣지 않는다.
const VERIFIED_SINCE: GitVersion = GitVersion::new(2, 34, 0);

/// 이 명령과 버전에서 `--end-of-options`를 넣어도 되는지.
pub(crate) fn supports_end_of_options(cmd: Cmd, git: GitVersion) -> bool {
    !matches!(cmd, Cmd::Checkout | Cmd::Reset | Cmd::CheckRefFormat) && git >= VERIFIED_SINCE
}

/// 리비전 인자를 붙인다. 되는 조합이면 앞에 `--end-of-options`를 넣는다.
/// git 크레이트에서 `--end-of-options` 글자는 이 함수에만 있다.
pub(crate) fn push_revs<'a>(args: &mut Vec<&'a str>, cmd: Cmd, git: GitVersion, revs: &[&'a str]) {
    if supports_end_of_options(cmd, git) {
        args.push("--end-of-options");
    }
    args.extend_from_slice(revs);
}

#[cfg(test)]
mod tests {
    use super::*;

    const UBUNTU_2204: GitVersion = GitVersion::new(2, 34, 1);
    const NEW: GitVersion = GitVersion::new(2, 45, 0);
    const OLD: GitVersion = GitVersion::new(2, 30, 0);

    #[test]
    fn verified_commands_get_the_marker() {
        for cmd in [Cmd::Log, Cmd::RevParse, Cmd::Switch, Cmd::Merge, Cmd::Push] {
            assert!(supports_end_of_options(cmd, UBUNTU_2204), "{cmd:?}");
            assert!(supports_end_of_options(cmd, NEW), "{cmd:?}");
            assert!(!supports_end_of_options(cmd, OLD), "{cmd:?}");
        }
    }

    #[test]
    fn checkout_reset_and_check_ref_format_never_get_it() {
        for cmd in [Cmd::Checkout, Cmd::Reset, Cmd::CheckRefFormat] {
            for git in [OLD, UBUNTU_2204, NEW] {
                assert!(!supports_end_of_options(cmd, git), "{cmd:?} {git}");
            }
        }
    }

    #[test]
    fn push_revs_puts_marker_before_revisions() {
        let mut args = vec!["log"];
        push_revs(&mut args, Cmd::Log, UBUNTU_2204, &["-x"]);
        assert_eq!(args, ["log", "--end-of-options", "-x"]);
        let mut args = vec!["log"];
        push_revs(&mut args, Cmd::Log, OLD, &["HEAD"]);
        assert_eq!(args, ["log", "HEAD"]);
    }
}
