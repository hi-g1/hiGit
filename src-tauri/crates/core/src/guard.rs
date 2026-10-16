//! 기록을 바꾸는 액션이 같이 쓰는 검사(D-097, D-099). 화면이 준 해시·이름은 낡거나 조작될 수 있어
//! core가 다시 확인한다. 해시는 `Oid`로만 명령줄에 닿는다.
//!
//! 쓰는 순서: `require_branch` → `refuse_during_operation` → 입력 확인(`verified_commit`) → 상태
//! 검사(`refuse_staged`·`refuse_dirty`·`refuse_published`). 앞의 검사일수록 git을 덜 부르고, 진행
//! 중인 작업이 있으면 뒤의 상태 검사는 뜻이 없다.

use git::{Change, GitBackend, Oid, RepoPath};

use crate::view::RestoreKind;
use crate::{CoreError, Result};

/// 해시가 어디에 있어야 하는지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// 지금 위치이거나 그 조상(되돌리기·돌아가기).
    HeadAncestor,
    /// 로컬 브랜치 끝에서 닿음(복사).
    BranchReachable,
    /// 로컬 브랜치 끝에서 닿거나 작업 내역에 버전으로 있음(태그·새 브랜치의 시작, 이때 파일 보기).
    #[allow(dead_code)] // 계획 008 E16부터 쓴다.
    BranchOrActivity,
}

const NO_VERSION: CoreError = CoreError::InvalidInput("없는 버전이에요");

/// 화면이 준 해시가 버전(커밋)이고 `scope` 안에 있으면 그 `Oid`.
pub(crate) fn verified_commit(
    git: &dyn GitBackend,
    repo: &RepoPath,
    oid: &str,
    scope: Scope,
) -> Result<Oid> {
    let oid = Oid::parse(oid).ok_or(NO_VERSION)?;
    if git
        .resolve_ref(repo, &format!("{oid}^{{commit}}"))?
        .as_ref()
        != Some(&oid)
    {
        return Err(NO_VERSION);
    }
    let ok = match scope {
        Scope::HeadAncestor => match git.resolve_ref(repo, "HEAD")? {
            Some(head) => git.is_ancestor(repo, oid.as_str(), head.as_str())?,
            None => false,
        },
        Scope::BranchReachable => git.reachable_from_branches(repo, &oid)?,
        Scope::BranchOrActivity => {
            git.reachable_from_branches(repo, &oid)? || in_activity(git, repo, &oid)?
        }
    };
    if !ok {
        return Err(NO_VERSION);
    }
    Ok(oid)
}

/// 작업 내역에 "버전"으로 나오는 해시인지(reflog 또는 버전 모양 백업 ref, D-085).
pub(crate) fn in_activity(git: &dyn GitBackend, repo: &RepoPath, oid: &Oid) -> Result<bool> {
    if git.reflog(repo, 200)?.iter().any(|r| &r.oid == oid) {
        return Ok(true);
    }
    for r in git.list_refs(repo, crate::undo::backup::BACKUP_PREFIX)? {
        if r.oid == oid.as_str()
            && crate::recovery::kind_of(git, repo, &r.oid) == RestoreKind::Version
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `base`에서 `tip`으로 가며 기록에서 바뀌거나 빠지는 버전(`tip`에만 있고 `base`에는 없는 것)이
/// 하나라도 원격 추적 브랜치에 닿으면 거절한다. 판정하지 못해도 거절한다(D-097).
pub(crate) fn refuse_published(
    git: &dyn GitBackend,
    repo: &RepoPath,
    tip: &Oid,
    base: &Oid,
) -> Result<()> {
    let (_, total) = git.count_between(repo, base.as_str(), tip.as_str())?;
    if total == 0 {
        return Ok(());
    }
    match git.count_unpublished(repo, tip, Some(base))? {
        Some(n) if n == total => Ok(()),
        _ => Err(CoreError::PublishedHistory),
    }
}

/// 버전 하나가 원격 추적 브랜치에 닿는지. 판정하지 못하면 올린 것으로 본다(D-097).
pub(crate) fn is_published(git: &dyn GitBackend, repo: &RepoPath, oid: &Oid) -> Result<bool> {
    // 첫 부모 아래는 세지 않는다: 이 버전이 닿으면 0, 안 닿으면 적어도 1(자기 자신).
    let parent = git.resolve_ref(repo, &format!("{oid}^1"))?;
    Ok(!matches!(
        git.count_unpublished(repo, oid, parent.as_ref())?,
        Some(n) if n > 0
    ))
}

/// 이번 버전에 넣은 파일이 있으면 거절한다(합친 버전에 섞이지 않게, D-069).
pub(crate) fn refuse_staged(git: &dyn GitBackend, repo: &RepoPath) -> Result<()> {
    let staged = git
        .status(repo)?
        .entries
        .iter()
        .any(|e| !matches!(e.index, Change::Unmodified | Change::Untracked));
    if staged {
        return Err(CoreError::StagedChanges);
    }
    Ok(())
}

/// 추적 중인 파일을 고쳤거나 넣은 파일이 있으면 거절한다. 새 파일(추적 안 함)은 둔다.
pub(crate) fn refuse_dirty(git: &dyn GitBackend, repo: &RepoPath) -> Result<()> {
    let clean = |c: Change| matches!(c, Change::Unmodified | Change::Untracked);
    let dirty = git
        .status(repo)?
        .entries
        .iter()
        .any(|e| !clean(e.index) || !clean(e.worktree));
    if dirty {
        return Err(CoreError::HasChanges);
    }
    Ok(())
}

/// 지금 브랜치 이름. 특정 버전을 보는 중이면 거절한다.
pub(crate) fn require_branch(git: &dyn GitBackend, repo: &RepoPath) -> Result<String> {
    git.current_branch(repo)?.ok_or(CoreError::InvalidInput(
        "특정 버전을 보는 중에는 할 수 없어요",
    ))
}

/// 기록 정리에서 한 번에 고르는 버전 수 상한(D-100).
pub(crate) const MAX_REWRITE: usize = 50;
/// 고른 구간 맨 아래의 부모부터 지금 위치까지 버전 수 상한(위쪽 버전도 다시 놓인다).
const MAX_CHAIN: usize = 1000;

/// 기록 정리할 구간. 모든 목록은 새 것부터.
#[derive(Debug, Clone)]
pub(crate) struct RewriteChain {
    pub branch: String,
    pub head: Oid,
    /// 고른 구간 맨 아래 버전의 부모(정리의 바탕). 첫 버전이 들어 있으면 없다.
    pub base: Option<Oid>,
    pub selected: Vec<git::Commit>,
    /// 고른 구간 위(새 쪽)부터 지금 위치까지. 순서 그대로 다시 놓인다.
    pub above: Vec<git::Commit>,
}

/// 기록 정리 검사 결과. 구간을 찾았으면 막혔어도 `chain`이 있다(화면이 카드를 보여준다).
#[derive(Debug, Clone)]
pub(crate) struct RewriteCheck {
    pub chain: Option<RewriteChain>,
    pub blocked: Option<crate::view::RewriteBlocked>,
}

/// 고른 버전이 기록 정리할 수 있는 구간인지(D-100, D-106). 조회는 막힌 까닭을 화면에 알리고
/// 액션은 같은 검사를 실행 때 다시 해 오류로 낸다. 지금 브랜치의 첫 부모 선을 지금 위치부터
/// 따라가 고른 버전이 그 선 위의 연속 구간이고, 구간부터 지금 위치까지 합친 버전·첫 버전이 없고,
/// 바뀔 버전이 하나도 올라가지 않았고, 고친 파일과 진행 중인 작업이 없어야 한다.
pub(crate) fn rewrite_chain(
    git: &dyn GitBackend,
    repo: &RepoPath,
    oids: &[String],
) -> Result<RewriteCheck> {
    use crate::view::RewriteBlocked as B;
    let blocked = |b: B| {
        Ok(RewriteCheck {
            chain: None,
            blocked: Some(b),
        })
    };
    // 리베이스 중에는 브랜치를 떠나 있으므로 진행 중인 작업을 먼저 본다.
    if crate::actions::switch::refuse_during_operation(git, repo).is_err()
        || git.index_entries(repo)?.iter().any(|e| e.stage > 0)
    {
        return blocked(B::OtherOperation);
    }
    let Some(branch) = git.current_branch(repo)? else {
        return blocked(B::Detached);
    };
    if oids.len() > MAX_REWRITE {
        return blocked(B::TooMany);
    }
    let Some(wanted) = oids
        .iter()
        .map(|o| Oid::parse(o))
        .collect::<Option<Vec<_>>>()
    else {
        return blocked(B::NotOnBranchLine);
    };
    let mut set: std::collections::HashSet<&str> = wanted.iter().map(Oid::as_str).collect();
    if set.is_empty() || set.len() != wanted.len() {
        return blocked(B::NotOnBranchLine);
    }
    let Some(head) = git.resolve_ref(repo, "HEAD")? else {
        return blocked(B::NotOnBranchLine);
    };
    let line = git.log(
        repo,
        &git::LogQuery {
            skip: 0,
            limit: (MAX_CHAIN + 1) as u32,
            rev: Some(head.to_string()),
        },
    )?;
    let (mut above, mut selected) = (Vec::new(), Vec::new());
    for commit in line {
        if commit.parents.len() > 1 {
            return blocked(B::HasMerge);
        }
        if set.remove(commit.oid.as_str()) {
            selected.push(commit);
            if set.is_empty() {
                break;
            }
        } else if !selected.is_empty() {
            // 고른 구간 사이에 안 고른 버전이 끼어 있다.
            return blocked(B::NotOnBranchLine);
        } else {
            above.push(commit);
        }
    }
    if !set.is_empty() {
        // 선 위에서 다 찾지 못했다(다른 브랜치의 버전, 너무 멂).
        let far = above.len() + selected.len() > MAX_CHAIN;
        return blocked(if far { B::TooMany } else { B::NotOnBranchLine });
    }
    let base = selected
        .last()
        .and_then(|c| c.parents.first())
        .and_then(|p| Oid::parse(p));
    let mut check = RewriteCheck {
        chain: Some(RewriteChain {
            branch,
            head: head.clone(),
            base: base.clone(),
            selected,
            above,
        }),
        blocked: None,
    };
    let Some(base) = base else {
        check.blocked = Some(B::IncludesFirstVersion);
        return Ok(check);
    };
    match refuse_published(git, repo, &head, &base) {
        Err(CoreError::PublishedHistory) => check.blocked = Some(B::Published),
        other => other?,
    }
    if check.blocked.is_none() {
        match refuse_dirty(git, repo) {
            Err(CoreError::HasChanges) => check.blocked = Some(B::HasChanges),
            other => other?,
        }
    }
    Ok(check)
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

    /// 버전 셋이 일렬인 저장소. (폴더, [첫째, 둘째, 셋째])
    fn line_of_three() -> (tempfile::TempDir, [Oid; 3]) {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        git(dir, &["config", "core.autocrlf", "false"]);
        let mut oids = Vec::new();
        for n in ["1", "2", "3"] {
            std::fs::write(dir.join("a.txt"), n).unwrap();
            git(dir, &["add", "-A"]);
            git(dir, &["commit", "-q", "-m", n]);
            oids.push(Oid::parse(&git(dir, &["rev-parse", "HEAD"])).unwrap());
        }
        (repo, oids.try_into().unwrap())
    }

    fn backend() -> git::CliBackend {
        git::CliBackend::new("git", Arc::new(|_| {}))
    }

    #[test]
    fn published_versions_are_refused() {
        let (tmp, [a, b, c]) = line_of_three();
        let dir = tmp.path();
        let (g, repo) = (backend(), RepoPath::new(dir));
        // 원격이 없으면 모두 안 올림.
        refuse_published(&g, &repo, &c, &a).unwrap();
        assert!(!is_published(&g, &repo, &c).unwrap());
        // 원격이 base에 있으면 그 위는 안 올림.
        git(dir, &["update-ref", "refs/remotes/origin/main", a.as_str()]);
        refuse_published(&g, &repo, &c, &a).unwrap();
        // 원격이 구간 안에 있으면 거절. 다른 이름의 원격 브랜치도 같다.
        git(dir, &["update-ref", "-d", "refs/remotes/origin/main"]);
        git(dir, &["update-ref", "refs/remotes/up/기능", b.as_str()]);
        assert!(matches!(
            refuse_published(&g, &repo, &c, &a),
            Err(CoreError::PublishedHistory)
        ));
        assert!(is_published(&g, &repo, &b).unwrap());
        assert!(is_published(&g, &repo, &a).unwrap());
        assert!(!is_published(&g, &repo, &c).unwrap());
        // 빠지는 버전이 없으면(앞으로 옮김) 통과.
        refuse_published(&g, &repo, &a, &c).unwrap();
        // 첫 버전(부모 없음)도 판정한다.
        git(dir, &["update-ref", "-d", "refs/remotes/up/기능"]);
        assert!(!is_published(&g, &repo, &a).unwrap());
    }

    #[test]
    fn dirty_and_staged_files() {
        let (tmp, _) = line_of_three();
        let dir = tmp.path();
        let (g, repo) = (backend(), RepoPath::new(dir));
        std::fs::write(dir.join("새.txt"), "새").unwrap();
        refuse_dirty(&g, &repo).unwrap();
        refuse_staged(&g, &repo).unwrap();
        std::fs::write(dir.join("a.txt"), "고침").unwrap();
        assert!(matches!(
            refuse_dirty(&g, &repo),
            Err(CoreError::HasChanges)
        ));
        refuse_staged(&g, &repo).unwrap();
        git(dir, &["add", "a.txt"]);
        assert!(matches!(
            refuse_dirty(&g, &repo),
            Err(CoreError::HasChanges)
        ));
        assert!(matches!(
            refuse_staged(&g, &repo),
            Err(CoreError::StagedChanges)
        ));
    }

    #[test]
    fn commits_are_checked_against_the_scope() {
        let (tmp, [a, _, c]) = line_of_three();
        let dir = tmp.path();
        let (g, repo) = (backend(), RepoPath::new(dir));
        assert_eq!(
            verified_commit(&g, &repo, a.as_str(), Scope::HeadAncestor).unwrap(),
            a
        );
        verified_commit(&g, &repo, c.as_str(), Scope::HeadAncestor).unwrap();
        // 짧은 해시·이름·트리 해시는 받지 않는다.
        let tree = git(dir, &["rev-parse", "HEAD^{tree}"]);
        for bad in [&a.as_str()[..7], "main", "HEAD", tree.as_str(), "-x"] {
            assert!(
                verified_commit(&g, &repo, bad, Scope::BranchOrActivity).is_err(),
                "{bad}"
            );
        }
        // 다른 브랜치의 버전은 지금 위치의 조상이 아니다.
        git(dir, &["switch", "-q", "-c", "옆", a.as_str()]);
        std::fs::write(dir.join("b.txt"), "옆").unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "옆"]);
        let side = Oid::parse(&git(dir, &["rev-parse", "HEAD"])).unwrap();
        git(dir, &["switch", "-q", "main"]);
        assert!(verified_commit(&g, &repo, side.as_str(), Scope::HeadAncestor).is_err());
        verified_commit(&g, &repo, side.as_str(), Scope::BranchReachable).unwrap();
        // 브랜치를 지우면 닿지 않지만 작업 내역(reflog)에는 남는다.
        git(dir, &["branch", "-q", "-D", "옆"]);
        assert!(verified_commit(&g, &repo, side.as_str(), Scope::BranchReachable).is_err());
        verified_commit(&g, &repo, side.as_str(), Scope::BranchOrActivity).unwrap();
    }

    #[test]
    fn detached_head_has_no_branch() {
        let (tmp, [a, ..]) = line_of_three();
        let dir = tmp.path();
        let (g, repo) = (backend(), RepoPath::new(dir));
        assert_eq!(require_branch(&g, &repo).unwrap(), "main");
        git(dir, &["switch", "-q", "--detach", a.as_str()]);
        assert!(matches!(
            require_branch(&g, &repo),
            Err(CoreError::InvalidInput(_))
        ));
    }
}
