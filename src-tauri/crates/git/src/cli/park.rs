//! 브랜치를 바꿀 때 고친 파일을 맡겨 두고 꺼내기(D-047, D-054).
//!
//! git에는 "목록에 넣지 않는 stash"가 없어 `stash push`로 만든 뒤 core가 ref를 쓰고 목록에서 뺀다.
//! 사용자의 stash 목록은 건드리지 않는다: push 전후 `refs/stash`가 같으면 맡길 것이 없었던 것이다.

use std::path::Path;

use super::CliBackend;
use super::args::{Cmd, push_revs};
use crate::{FailureKind, GitError, Oid, OpOutput, RepoPath, Result};

impl CliBackend {
    /// 고친 파일(새 파일 포함, 무시된 파일 제외)을 stash로 만든다. 새로 만든 stash 해시를 돌려주고,
    /// 맡길 것이 없었으면 `None`. 목록에서 빼는 일은 core가 ref를 쓴 뒤 `StashDropIf`로 한다.
    pub(super) fn park_push(&self, repo: &RepoPath, message: &str) -> Result<OpOutput> {
        let before = self.stash_top(repo)?;
        let out = self.runner.run(
            Some(repo.as_path()),
            &["stash", "push", "--include-untracked", "-m", message],
        )?;
        let after = self.stash_top(repo)?;
        let created = match (&before, &after) {
            (_, None) => None,
            (Some(b), Some(a)) if a == b => None,
            (_, Some(a)) => Some(a.clone()),
        };
        Ok(OpOutput {
            stdout: out.stdout_lossy(),
            stderr: out.stderr_lossy(),
            oid: created,
        })
    }

    /// 꺼내되 겹친 곳은 충돌로 남긴다. 겹치지 않으면 `unpark`와 같다.
    pub(super) fn unpark_keep(&self, repo: &RepoPath, oid: &Oid) -> Result<OpOutput> {
        let dir = repo.as_path();
        let status = crate::GitBackend::status(self, repo)?;
        if !status.entries.is_empty() {
            return Err(GitError::Refused {
                kind: FailureKind::WouldOverwrite,
                detail: "고친 파일이 있어 맡긴 파일을 꺼내지 않았어요".into(),
            });
        }
        // 새 파일끼리 겹치면 `stash apply`가 충돌을 남기지 못하고 멈춘다.
        let untracked = self.stash_untracked_paths(repo, oid)?;
        if let Some(clash) = untracked.iter().find(|p| exists(dir, p)) {
            return Err(GitError::Refused {
                kind: FailureKind::UnparkConflict,
                detail: format!("already exists: {clash}"),
            });
        }
        let mut args = vec!["stash", "apply"];
        push_revs(&mut args, Cmd::Stash, self.git_version(), &[oid.as_str()]);
        match self.runner.run(Some(dir), &args) {
            Ok(out) => Ok(OpOutput {
                stdout: out.stdout_lossy(),
                stderr: out.stderr_lossy(),
                oid: Some(oid.clone()),
            }),
            // 겹쳐서 충돌 항목을 남겼으면 성공으로 본다. 그 밖의 실패는 꺼내기 전으로 돌린다.
            Err(e) => {
                let conflicted = self.ls_index(repo)?.iter().any(|e| e.stage > 0);
                if conflicted {
                    return Ok(OpOutput {
                        oid: Some(oid.clone()),
                        ..OpOutput::default()
                    });
                }
                self.runner.run(Some(dir), &["reset", "--hard", "-q"])?;
                for path in &untracked {
                    let _ = std::fs::remove_file(dir.join(path));
                }
                Err(e)
            }
        }
    }

    /// `unpark_keep`이 남긴 상태를 꺼내기 전으로. 그 stash에 없는 파일이 바뀌었으면 하지 않는다.
    pub(super) fn unpark_abort(&self, repo: &RepoPath, oid: &Oid) -> Result<OpOutput> {
        let dir = repo.as_path();
        let mut owned = self.stash_paths(repo, oid)?;
        owned.extend(self.stash_untracked_paths(repo, oid)?);
        let status = crate::GitBackend::status(self, repo)?;
        if let Some(other) = status.entries.iter().find(|e| !owned.contains(&e.path)) {
            return Err(GitError::Refused {
                kind: FailureKind::WouldOverwrite,
                detail: format!("not from the parked files: {}", other.path),
            });
        }
        self.runner.run(Some(dir), &["reset", "--hard", "-q"])?;
        for path in self.stash_untracked_paths(repo, oid)? {
            let _ = std::fs::remove_file(dir.join(path));
        }
        Ok(OpOutput::default())
    }

    /// stash가 고친 파일 경로(첫 부모와 stash 커밋 사이).
    fn stash_paths(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>> {
        let first = format!("{oid}^1");
        let mut args = vec!["diff", "--name-only", "-z", "--no-renames"];
        push_revs(
            &mut args,
            Cmd::Diff,
            self.git_version(),
            &[&first, oid.as_str()],
        );
        let out = self.runner.run(Some(repo.as_path()), &args)?;
        let paths: Vec<String> = out
            .stdout_lossy()
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
        if let Some(bad) = paths.iter().find(|p| !safe_relative(p)) {
            return Err(GitError::Parse(format!("stash 경로: {bad}")));
        }
        Ok(paths)
    }

    /// stash 목록(`log -g refs/stash`). 레코드는 `해시\0시각\0제목\0`.
    pub(super) fn stashes(&self, repo: &RepoPath) -> Result<Vec<crate::StashEntry>> {
        if self.stash_top(repo)?.is_none() {
            return Ok(Vec::new());
        }
        let out = self.runner.run(
            Some(repo.as_path()),
            &["log", "-g", "-z", "--format=%H%x00%ct%x00%gs", "refs/stash"],
        )?;
        let text = out.stdout_lossy();
        let fields: Vec<&str> = text.split('\0').collect();
        let (records, _) = fields.as_chunks::<3>();
        Ok(records
            .iter()
            .filter_map(|[oid, at, subject]| {
                Some(crate::StashEntry {
                    oid: Oid::parse(oid.trim_start_matches('\n'))?,
                    at: at.parse().ok()?,
                    subject: subject.to_string(),
                })
            })
            .collect())
    }

    /// stash 목록에서 `oid` 항목을 뺀다. 위치를 찾은 뒤 빼기 직전에 그 자리가 같은지 다시 본다.
    pub(super) fn stash_drop_oid(&self, repo: &RepoPath, oid: &Oid) -> Result<OpOutput> {
        let list = self.stashes(repo)?;
        let Some(index) = list.iter().position(|e| &e.oid == oid) else {
            return Err(GitError::Refused {
                kind: FailureKind::Other,
                detail: format!("stash {oid} not found"),
            });
        };
        let place = format!("stash@{{{index}}}");
        let now = crate::GitBackend::resolve_ref(self, repo, &place)?;
        if now.as_ref() != Some(oid) {
            return Err(GitError::Refused {
                kind: FailureKind::Other,
                detail: "stash list changed".into(),
            });
        }
        let out = self
            .runner
            .run(Some(repo.as_path()), &["stash", "drop", "-q", &place])?;
        Ok(OpOutput {
            stdout: out.stdout_lossy(),
            stderr: out.stderr_lossy(),
            oid: Some(oid.clone()),
        })
    }

    /// stash 목록 맨 위가 `oid`일 때만 뺀다. 그사이 다른 stash가 생겼으면 그대로 둔다.
    pub(super) fn stash_drop_if(&self, repo: &RepoPath, oid: &Oid) -> Result<OpOutput> {
        if self.stash_top(repo)?.as_ref() != Some(oid) {
            return Ok(OpOutput::default());
        }
        let out = self
            .runner
            .run(Some(repo.as_path()), &["stash", "drop", "-q", "stash@{0}"])?;
        Ok(OpOutput {
            stdout: out.stdout_lossy(),
            stderr: out.stderr_lossy(),
            oid: Some(oid.clone()),
        })
    }

    /// 맡긴 파일을 꺼낸다. 작업 폴더가 깨끗할 때만 하고, 겹쳐서 실패하면 깨끗한 상태로 되돌린 뒤
    /// `UnparkConflict`로 알린다. 작업 폴더에 충돌 표시를 남기지 않는다.
    pub(super) fn unpark(&self, repo: &RepoPath, oid: &Oid) -> Result<OpOutput> {
        let dir = repo.as_path();
        let status = crate::GitBackend::status(self, repo)?;
        if !status.entries.is_empty() {
            return Err(GitError::Refused {
                kind: FailureKind::WouldOverwrite,
                detail: "고친 파일이 있어 맡긴 파일을 꺼내지 않았어요".into(),
            });
        }
        // 맡길 때 새 파일이었던 것(stash의 세 번째 부모)이 지금 작업 폴더에 있으면 꺼내지 않는다.
        let untracked = self.stash_untracked_paths(repo, oid)?;
        if let Some(clash) = untracked.iter().find(|p| exists(dir, p)) {
            return Err(GitError::Refused {
                kind: FailureKind::UnparkConflict,
                detail: format!("already exists: {clash}"),
            });
        }
        let mut args = vec!["stash", "apply", "--index"];
        push_revs(&mut args, Cmd::Stash, self.git_version(), &[oid.as_str()]);
        match self.runner.run(Some(dir), &args) {
            Ok(out) => Ok(OpOutput {
                stdout: out.stdout_lossy(),
                stderr: out.stderr_lossy(),
                oid: Some(oid.clone()),
            }),
            Err(e) => {
                // 반쯤 적용됐을 수 있다. 시작할 때 깨끗했으므로 HEAD로 되돌리고 꺼낸 새 파일을 지운다.
                self.runner.run(Some(dir), &["reset", "--hard", "-q"])?;
                for path in &untracked {
                    let _ = std::fs::remove_file(dir.join(path));
                }
                Err(GitError::Refused {
                    kind: FailureKind::UnparkConflict,
                    detail: e.detail(),
                })
            }
        }
    }

    /// stash에 든 파일: 맡길 때 버전과 달라진 추적 파일 + 새 파일. 이름 순, 겹침 없이.
    pub(super) fn parked_paths_impl(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>> {
        let base = format!("{oid}^1");
        let mut args = vec!["diff", "--name-only", "-z", "--no-renames", "--no-ext-diff"];
        push_revs(
            &mut args,
            Cmd::Diff,
            self.git_version(),
            &[&base, oid.as_str()],
        );
        args.push("--");
        let out = self.runner.run(Some(repo.as_path()), &args)?;
        let mut paths: Vec<String> = out
            .stdout_lossy()
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
        paths.extend(self.stash_untracked_paths(repo, oid)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn stash_top(&self, repo: &RepoPath) -> Result<Option<Oid>> {
        crate::GitBackend::resolve_ref(self, repo, "refs/stash")
    }

    /// stash에 새 파일이 들어 있으면 그 경로들(`/` 구분).
    fn stash_untracked_paths(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>> {
        let third = format!("{oid}^3");
        if crate::GitBackend::resolve_ref(self, repo, &third)?.is_none() {
            return Ok(Vec::new());
        }
        let mut args = vec!["ls-tree", "-r", "-z", "--name-only"];
        push_revs(&mut args, Cmd::LsTree, self.git_version(), &[&third]);
        let out = self.runner.run(Some(repo.as_path()), &args)?;
        let paths: Vec<String> = out
            .stdout_lossy()
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
        // 맡긴 stash도 받은 저장소에 들어 있을 수 있는 입력이다(D-034). 이상한 경로가 있으면 꺼내지 않는다.
        if let Some(bad) = paths.iter().find(|p| !safe_relative(p)) {
            return Err(GitError::Refused {
                kind: FailureKind::UnparkConflict,
                detail: format!("unsafe path in parked files: {bad}"),
            });
        }
        Ok(paths)
    }
}

/// 저장소 안 상대 경로인지: 절대 경로, 드라이브, `\`, `..`, 빈 조각, `.git` 조각을 거부한다.
pub(super) fn safe_relative(path: &str) -> bool {
    let b = path.as_bytes();
    let drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    !(path.starts_with('/')
        || path.contains('\\')
        || path.contains('\0')
        || drive
        || path
            .split('/')
            .any(|c| c.is_empty() || c == ".." || c.eq_ignore_ascii_case(".git")))
}

fn exists(dir: &Path, rel: &str) -> bool {
    dir.join(rel).symlink_metadata().is_ok()
}

#[cfg(test)]
mod tests {
    use super::safe_relative;

    #[test]
    fn rejects_paths_that_leave_the_repo() {
        for bad in [
            "/etc/x",
            "../x",
            "a/../b",
            "C:/x",
            "a\\b",
            ".git/config",
            "a/.GIT/x",
            "a//b",
        ] {
            assert!(!safe_relative(bad), "{bad}");
        }
        for good in ["a.txt", "폴더/새 파일.txt", ".gitignore", "a..b"] {
            assert!(safe_relative(good), "{good}");
        }
    }
}
