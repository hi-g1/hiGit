//! 원격 ref 쓰기(덮어쓰며 올리기·원격 브랜치 지우기·만들기·태그 올리기)와 `ls-remote`(D-128, D-130, D-131).
//!
//! 덮어쓰기는 값 있는 lease(`--force-with-lease=<ref>:<기대 값>`)로만 한다. `--force` 단독이나 `+` refspec을
//! 쓰지 않아 그사이 남이 올린 것을 지우지 않는다. 해시는 `Oid`로만, 이름은 core가 검사한 값만 받는다.

use super::CliBackend;
use super::args::{Cmd, push_revs};
use super::network::NetMode;
use super::runner::GitOutput;
use crate::{AuthMode, Oid, RepoPath, Result};

/// 올리기 공통 앞부분. 태그·서브모듈·서명은 저장소 설정(push.followTags 등)을 따르지 않는다.
pub(super) fn push_prefix() -> Vec<&'static str> {
    vec![
        "push",
        "--progress",
        "--no-follow-tags",
        "--no-recurse-submodules",
        "--no-signed",
    ]
}

impl CliBackend {
    /// 원격 `branch`가 `expect`일 때만 `src`로 바꾼다.
    pub(super) fn push_force(
        &self,
        repo: &RepoPath,
        remote: &str,
        branch: &str,
        src: &Oid,
        expect: &Oid,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let lease = format!("--force-with-lease=refs/heads/{branch}:{expect}");
        let refspec = format!("{src}:refs/heads/{branch}");
        self.push_ref(repo, Some(&lease), remote, &refspec, auth)
    }

    /// 원격 `branch`가 `expect`일 때만 지운다.
    pub(super) fn push_delete(
        &self,
        repo: &RepoPath,
        remote: &str,
        branch: &str,
        expect: &Oid,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let lease = format!("--force-with-lease=refs/heads/{branch}:{expect}");
        let refspec = format!(":refs/heads/{branch}");
        self.push_ref(repo, Some(&lease), remote, &refspec, auth)
    }

    /// 원격에 `branch`가 없을 때만 `oid`로 만든다(빈 기대 값 = 없어야 함).
    pub(super) fn push_new_ref(
        &self,
        repo: &RepoPath,
        remote: &str,
        branch: &str,
        oid: &Oid,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let lease = format!("--force-with-lease=refs/heads/{branch}:");
        let refspec = format!("{oid}:refs/heads/{branch}");
        self.push_ref(repo, Some(&lease), remote, &refspec, auth)
    }

    /// 원격 태그가 `expect`일 때만 지운다(D-182).
    pub(super) fn push_delete_tag(
        &self,
        repo: &RepoPath,
        remote: &str,
        name: &str,
        expect: &Oid,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let lease = format!("--force-with-lease=refs/tags/{name}:{expect}");
        let refspec = format!(":refs/tags/{name}");
        self.push_ref(repo, Some(&lease), remote, &refspec, auth)
    }

    /// 태그 하나를 올린다. 강제하지 않는다.
    pub(super) fn push_tag(
        &self,
        repo: &RepoPath,
        remote: &str,
        name: &str,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let refspec = format!("refs/tags/{name}:refs/tags/{name}");
        self.push_ref(repo, None, remote, &refspec, auth)
    }

    fn push_ref(
        &self,
        repo: &RepoPath,
        lease: Option<&str>,
        remote: &str,
        refspec: &str,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let mut args = push_prefix();
        args.extend(lease);
        push_revs(&mut args, Cmd::Push, self.git_version(), &[remote, refspec]);
        self.run_network(repo, &args, NetMode::Foreground, auth)
    }

    pub(super) fn ls_remote_impl(
        &self,
        repo: &RepoPath,
        remote: &str,
        refname: &str,
        auth: AuthMode,
    ) -> Result<Option<Oid>> {
        let mut args = vec!["ls-remote"];
        push_revs(
            &mut args,
            Cmd::LsRemote,
            self.git_version(),
            &[remote, refname],
        );
        let out = self.run_network(repo, &args, NetMode::Foreground, auth)?;
        Ok(parse_ls_remote(&out.stdout_lossy(), refname))
    }

    pub(super) fn ls_remote_head_impl(
        &self,
        repo: &RepoPath,
        remote: &str,
        auth: AuthMode,
    ) -> Result<Option<String>> {
        let mut args = vec!["ls-remote", "--symref"];
        push_revs(
            &mut args,
            Cmd::LsRemote,
            self.git_version(),
            &[remote, "HEAD"],
        );
        let out = self.run_network(repo, &args, NetMode::Foreground, auth)?;
        Ok(parse_symref_head(&out.stdout_lossy()))
    }
}

/// `<oid>\t<ref>` 줄 가운데 ref 전체가 `refname`과 정확히 같은 것(짧은 이름은 꼬리 일치로 다른 ref도 걸린다).
pub(super) fn parse_ls_remote(text: &str, refname: &str) -> Option<Oid> {
    text.lines().find_map(|line| {
        let (oid, name) = line.split_once('\t')?;
        (name == refname).then(|| Oid::parse(oid)).flatten()
    })
}

/// `ref: refs/heads/main\tHEAD` 줄의 브랜치 이름.
pub(super) fn parse_symref_head(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let target = line.strip_prefix("ref: ")?.strip_suffix("\tHEAD")?;
        target.strip_prefix("refs/heads/").map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ls_remote_takes_only_the_exact_ref() {
        let a = "a".repeat(40);
        let b = "b".repeat(40);
        let text = format!("{a}\trefs/heads/feature/main\n{b}\trefs/heads/main\n");
        assert_eq!(
            parse_ls_remote(&text, "refs/heads/main").map(|o| o.to_string()),
            Some(b)
        );
        assert_eq!(parse_ls_remote(&text, "refs/heads/없음"), None);
        assert_eq!(
            parse_ls_remote("not-an-oid\trefs/heads/main\n", "refs/heads/main"),
            None
        );
    }

    #[test]
    fn symref_head_reads_the_branch() {
        let a = "a".repeat(40);
        let text = format!("ref: refs/heads/기본\tHEAD\n{a}\tHEAD\n");
        assert_eq!(parse_symref_head(&text).as_deref(), Some("기본"));
        assert_eq!(parse_symref_head(&format!("{a}\tHEAD\n")), None);
    }
}
