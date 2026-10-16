//! "올린 기록을 바꿨다"는 표식(D-129, 계획 010 검토 C6). 설정을 켜고(`PublishedRule::Allow`) 올린 버전의
//! 기록을 바꾸기 직전에 남기고, 동기화·pull이 원격의 옛 버전을 다시 들여오지 않게 멈춘다.
//!
//! journal 줄이 아니라 git 폴더의 서명 표식(`core::marker`, journal 키, git 폴더 실제 경로에 묶임)이라
//! journal이 잘리거나 경계 줄 뒤여도 남는다. 브랜치마다 그때의 upstream과 그 끝(`old_tip`)을 적고,
//! HEAD가 그 끝을 품게 되면(합쳐 들임·덮어쓰며 올린 뒤 받음) 저절로 지운다. 덮어쓰며 올리기 성공과
//! 브랜치 지우기도 지운다.

use git::{GitBackend, Oid, RepoPath};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::marker::{self, Name};
use crate::undo::JournalKey;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Marks {
    branches: Vec<Mark>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Mark {
    branch: String,
    /// `origin/기능` 같은 원격 추적 이름.
    upstream: String,
    /// 표식을 남길 때 upstream이 가리키던 버전(원격의 옛 기록 끝).
    old_tip: String,
}

/// 지금 브랜치에 표식을 남긴다. upstream이 없으면(올린 곳 없음) 남기지 않는다. 못 남기면 오류라
/// 기록을 바꾸기 전에 부른다(실패하면 바꾸지 않는다).
pub(crate) fn record(git: &dyn GitBackend, repo: &RepoPath, key: &JournalKey) -> Result<()> {
    let Some(branch) = git.current_branch(repo)? else {
        return Ok(());
    };
    let Some(upstream) = git.status(repo)?.branch.upstream else {
        return Ok(());
    };
    let Some(tip) = git.resolve_ref(repo, &format!("refs/remotes/{upstream}"))? else {
        return Ok(());
    };
    let dir = git.git_dir(repo)?;
    let mut marks: Marks = marker::read(&dir, Name::Rewrote, key).unwrap_or_default();
    marks.branches.retain(|m| m.branch != branch);
    marks.branches.push(Mark {
        branch,
        upstream,
        old_tip: tip.to_string(),
    });
    marker::write(&dir, Name::Rewrote, key, &marks)
}

/// 지금 브랜치의 표식 상태. 표식이 없으면 `None`, 있으면 (브랜치, 이미 끝났는지).
fn state(
    git: &dyn GitBackend,
    repo: &RepoPath,
    key: &JournalKey,
) -> Result<Option<(String, bool)>> {
    let Some(branch) = git.current_branch(repo)? else {
        return Ok(None);
    };
    let dir = git.git_dir(repo)?;
    let Some(marks) = marker::read::<Marks>(&dir, Name::Rewrote, key) else {
        return Ok(None);
    };
    let Some(mark) = marks.branches.iter().find(|m| m.branch == branch) else {
        return Ok(None);
    };
    let upstream = git.status(repo)?.branch.upstream;
    let settled = match Oid::parse(&mark.old_tip) {
        Some(old) if upstream.as_deref() == Some(mark.upstream.as_str()) => {
            // 옛 버전이 사라져 비교하지 못하면 아직 갈라진 것으로 본다(막는 쪽).
            matches!(git.is_ancestor(repo, old.as_str(), "HEAD"), Ok(true))
        }
        _ => true,
    };
    Ok(Some((branch, settled)))
}

/// 지금 브랜치에 아직 유효한 표식이 있는지. upstream이 바뀌었거나 HEAD가 표식의 옛 끝을 품게 됐으면
/// 그 표식을 지우고 거짓.
pub(crate) fn active(git: &dyn GitBackend, repo: &RepoPath, key: &JournalKey) -> Result<bool> {
    match state(git, repo, key)? {
        Some((branch, true)) => {
            forget(git, repo, key, &branch)?;
            Ok(false)
        }
        Some((_, false)) => Ok(true),
        None => Ok(false),
    }
}

/// `active`와 같되 파일을 지우지 않는다(조회용).
pub(crate) fn marked(git: &dyn GitBackend, repo: &RepoPath, key: &JournalKey) -> Result<bool> {
    Ok(matches!(state(git, repo, key)?, Some((_, false))))
}

/// 브랜치의 표식을 지운다(덮어쓰며 올리기 성공, 브랜치 지우기).
pub(crate) fn forget(
    git: &dyn GitBackend,
    repo: &RepoPath,
    key: &JournalKey,
    branch: &str,
) -> Result<()> {
    let dir = git.git_dir(repo)?;
    if !marker::exists(&dir, Name::Rewrote) {
        return Ok(());
    }
    let mut marks: Marks = marker::read(&dir, Name::Rewrote, key).unwrap_or_default();
    marks.branches.retain(|m| m.branch != branch);
    if marks.branches.is_empty() {
        marker::clear(&dir, Name::Rewrote);
        Ok(())
    } else {
        marker::write(&dir, Name::Rewrote, key, &marks)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use super::*;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// `기능` 브랜치, upstream `origin/기능`이 지금 버전(`old`)을 가리킨다.
    fn repo() -> (tempfile::TempDir, String) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        git(dir, &["init", "-q", "-b", "기능"]);
        for n in ["a", "b"] {
            std::fs::write(dir.join(format!("{n}.txt")), n).unwrap();
            git(dir, &["add", "-A"]);
            git(dir, &["commit", "-q", "-m", n]);
        }
        let old = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &["remote", "add", "origin", "https://example.com/r.git"],
        );
        git(dir, &["update-ref", "refs/remotes/origin/기능", &old]);
        git(dir, &["branch", "--set-upstream-to=origin/기능"]);
        (tmp, old)
    }

    fn backend() -> git::CliBackend {
        git::CliBackend::new("git", Arc::new(|_| {}))
    }

    #[test]
    fn mark_stays_until_head_holds_the_old_tip() {
        let (tmp, old) = repo();
        let dir = tmp.path();
        let (g, r, key) = (backend(), RepoPath::new(dir), JournalKey::ephemeral());
        assert!(!active(&g, &r, &key).unwrap());
        record(&g, &r, &key).unwrap();
        // 아직 HEAD가 옛 끝 그대로면 이미 품은 것이라 지운다.
        assert!(!active(&g, &r, &key).unwrap());
        record(&g, &r, &key).unwrap();
        git(dir, &["commit", "-q", "--amend", "-m", "고침"]);
        assert!(active(&g, &r, &key).unwrap());
        // 다른 키로는 읽지 않는다(서명).
        assert!(!active(&g, &r, &JournalKey::ephemeral()).unwrap());
        // 옛 끝을 합쳐 들이면 지워진다.
        git(dir, &["merge", "-q", "--no-edit", "-s", "ours", &old]);
        assert!(!active(&g, &r, &key).unwrap());
        assert!(!dir.join(".git/higit/rewrote.json").exists());
    }

    #[test]
    fn a_new_upstream_or_another_branch_drops_or_keeps_the_mark() {
        let (tmp, old) = repo();
        let dir = tmp.path();
        let (g, r, key) = (backend(), RepoPath::new(dir), JournalKey::ephemeral());
        git(dir, &["commit", "-q", "--amend", "-m", "고침"]);
        record(&g, &r, &key).unwrap();
        // 다른 브랜치에서는 그 브랜치의 표식만 본다.
        git(dir, &["switch", "-q", "-c", "옆"]);
        assert!(!active(&g, &r, &key).unwrap());
        git(dir, &["switch", "-q", "기능"]);
        assert!(active(&g, &r, &key).unwrap());
        // upstream을 다른 원격 브랜치로 바꾸면 그 표식은 뜻이 없다.
        git(dir, &["update-ref", "refs/remotes/origin/새것", &old]);
        git(dir, &["branch", "--set-upstream-to=origin/새것"]);
        assert!(!active(&g, &r, &key).unwrap());
        // 표식 파일이 하나도 없으면 forget은 아무 일도 하지 않는다.
        forget(&g, &r, &key, "기능").unwrap();
    }
}
