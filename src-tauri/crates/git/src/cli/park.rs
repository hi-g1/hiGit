//! 브랜치를 바꿀 때 고친 파일을 맡겨 두고 꺼내기(D-047, D-054).
//!
//! git에는 "목록에 넣지 않는 stash"가 없어 `stash push`로 만든 뒤 core가 ref를 쓰고 목록에서 뺀다.
//! 사용자의 stash 목록은 건드리지 않는다: push 전후 `refs/stash`가 같으면 맡길 것이 없었던 것이다.

use std::ffi::OsStr;
use std::path::Path;

use super::CliBackend;
use super::args::{Cmd, push_revs};
use super::merge_check::Scratch;
use super::runner::nul_separated;
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

    /// 고른 경로를 상태에서 나눈다: (추적 중인 바뀐 경로, 새 파일). 바뀌지 않은 경로는 뺀다.
    /// 이름 바꿈의 옛 이름도 추적 중인 경로로 센다.
    fn split_changed(
        &self,
        repo: &RepoPath,
        paths: &[String],
    ) -> Result<(Vec<String>, Vec<String>)> {
        if let Some(bad) = paths.iter().find(|p| !safe_relative(p)) {
            return Err(GitError::Parse(format!("저장소 밖 경로: {bad}")));
        }
        let status = crate::GitBackend::status(self, repo)?;
        let mut tracked = Vec::new();
        let mut untracked = Vec::new();
        for p in paths {
            let entry = status
                .entries
                .iter()
                .find(|e| &e.path == p || e.orig_path.as_ref() == Some(p));
            match entry {
                Some(e) if e.kind == crate::EntryKind::Untracked => untracked.push(p.clone()),
                Some(_) => tracked.push(p.clone()),
                None => {}
            }
        }
        tracked.sort();
        tracked.dedup();
        untracked.sort();
        untracked.dedup();
        Ok((tracked, untracked))
    }

    /// 고른 경로만 담은 stash 모양 버전(D-196). 첫 부모는 지금 버전, 두 번째는 지금 버전에 그 경로의
    /// index 모습만 얹은 버전, 새 파일이 있으면 세 번째 부모. 임시 index로 만들어 저장소는 그대로다.
    pub(super) fn snapshot_paths(
        &self,
        repo: &RepoPath,
        paths: &[String],
        message: &str,
    ) -> Result<OpOutput> {
        let (tracked, untracked) = self.split_changed(repo, paths)?;
        if tracked.is_empty() && untracked.is_empty() {
            return Ok(OpOutput::default());
        }
        let dir = Some(repo.as_path());
        let head = crate::GitBackend::resolve_ref(self, repo, "HEAD")?
            .ok_or_else(|| GitError::Parse("HEAD 없음".into()))?;
        // index 모습: 지금 index에 있으면 그 항목, 없으면(넣은 삭제·이름 바꿈의 옛 이름) 지운다(모드 0).
        let index = self.ls_index(repo)?;
        let zero = "0".repeat(head.as_str().len());
        let mut info = Vec::new();
        for p in &tracked {
            let line = match index.iter().find(|e| &e.path == p && e.stage == 0) {
                Some(e) => format!("{} {}\t{p}\0", e.mode, e.oid),
                None => format!("0 {zero}\t{p}\0"),
            };
            info.extend_from_slice(line.as_bytes());
        }
        let scratch = Scratch::new()?;
        let index_file = scratch.path("index");
        let env = [("GIT_INDEX_FILE", index_file.as_os_str())];
        self.runner
            .run_env(dir, &["read-tree", head.as_str()], &env)?;
        if !info.is_empty() {
            self.runner.run_with_stdin_env(
                dir,
                &["update-index", "-z", "--index-info"],
                &info,
                &env,
            )?;
        }
        let index_tree = self
            .runner
            .run_env(dir, &["write-tree"], &env)?
            .stdout_utf8()?;
        // 작업 폴더 모습: index 모습 위에 그 경로의 파일(없으면 지운다).
        if !tracked.is_empty() {
            self.runner.run_with_stdin_env(
                dir,
                &["update-index", "--add", "--remove", "-z", "--stdin"],
                &nul_separated(&tracked),
                &env,
            )?;
        }
        let work_tree = self
            .runner
            .run_env(dir, &["write-tree"], &env)?
            .stdout_utf8()?;
        let branch =
            crate::GitBackend::current_branch(self, repo)?.unwrap_or_else(|| "HEAD".into());
        let ident: [(&str, &OsStr); 4] = [
            ("GIT_AUTHOR_NAME", OsStr::new("hiGit")),
            ("GIT_AUTHOR_EMAIL", OsStr::new("higit@localhost")),
            ("GIT_COMMITTER_NAME", OsStr::new("hiGit")),
            ("GIT_COMMITTER_EMAIL", OsStr::new("higit@localhost")),
        ];
        let commit = |tree: &str, parents: &[&str], subject: String| -> Result<String> {
            let mut args = vec!["commit-tree", "--no-gpg-sign", tree];
            for p in parents {
                args.extend(["-p", p]);
            }
            args.extend(["-m", subject.as_str()]);
            Ok(self
                .runner
                .run_env(dir, &args, &ident)?
                .stdout_utf8()?
                .trim()
                .to_string())
        };
        let index_commit = commit(
            index_tree.trim(),
            &[head.as_str()],
            format!("index on {branch}: {message}"),
        )?;
        let untracked_commit = if untracked.is_empty() {
            None
        } else {
            let file = scratch.path("untracked");
            let env = [("GIT_INDEX_FILE", file.as_os_str())];
            self.runner.run_with_stdin_env(
                dir,
                &["update-index", "--add", "-z", "--stdin"],
                &nul_separated(&untracked),
                &env,
            )?;
            let tree = self
                .runner
                .run_env(dir, &["write-tree"], &env)?
                .stdout_utf8()?;
            Some(commit(
                tree.trim(),
                &[],
                format!("untracked files on {branch}: {message}"),
            )?)
        };
        let mut parents = vec![head.as_str(), index_commit.as_str()];
        if let Some(u) = &untracked_commit {
            parents.push(u.as_str());
        }
        let snapshot = commit(
            work_tree.trim(),
            &parents,
            format!("On {branch}: {message}"),
        )?;
        Ok(OpOutput {
            oid: Oid::parse(&snapshot),
            ..OpOutput::default()
        })
    }

    /// 고른 경로를 지금 버전 그대로 돌린다: 추적 중인 경로는 `restore --staged --worktree`(넣은 새 파일·
    /// 이름 바꿈도 지금 버전 모습으로), 새 파일은 지운다. 경로는 stdin과 `--literal-pathspecs`로.
    pub(super) fn discard_paths(&self, repo: &RepoPath, paths: &[String]) -> Result<OpOutput> {
        let (tracked, untracked) = self.split_changed(repo, paths)?;
        if !tracked.is_empty() {
            self.runner.run_with_stdin(
                Some(repo.as_path()),
                &[
                    "--literal-pathspecs",
                    "restore",
                    "--source=HEAD",
                    "--staged",
                    "--worktree",
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                &super::runner::nul_separated(&tracked),
            )?;
        }
        for path in &untracked {
            remove_inside(repo.as_path(), path);
        }
        Ok(OpOutput::default())
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
                    remove_inside(dir, path);
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
            remove_inside(dir, &path);
        }
        Ok(OpOutput::default())
    }

    /// 버린 파일을 되살린다(D-196). 그 경로가 지금 모두 깨끗할 때만, 다른 파일(넣어 둔 것 포함)은
    /// 건드리지 않는다. `stash apply`는 넣어 둔 파일이 있으면 거절하므로 경로마다 직접 쓴다:
    /// index는 두 번째 부모에서(`reset <버전> -- 경로`), 작업 폴더는 stash 버전에서(임시 index와
    /// `checkout-index`, 지운 파일은 지운다), 새 파일은 세 번째 부모에서.
    pub(super) fn restore_discarded(&self, repo: &RepoPath, oid: &Oid) -> Result<OpOutput> {
        let dir = repo.as_path();
        let refuse = |detail: String| GitError::Refused {
            kind: FailureKind::WouldOverwrite,
            detail,
        };
        let head = crate::GitBackend::resolve_ref(self, repo, "HEAD")?;
        let base = crate::GitBackend::resolve_ref(self, repo, &format!("{oid}^1"))?;
        if head.is_none() || head != base {
            return Err(refuse(
                "the version changed since the files were discarded".into(),
            ));
        }
        let worktree = self.name_status(repo, &format!("{oid}^1"), oid.as_str())?;
        let staged = self.name_status(repo, &format!("{oid}^1"), &format!("{oid}^2"))?;
        let untracked = self.stash_untracked_paths(repo, oid)?;
        let mut owned: Vec<String> = worktree
            .iter()
            .chain(&staged)
            .map(|(_, p)| p.clone())
            .collect();
        owned.sort();
        owned.dedup();
        let status = crate::GitBackend::status(self, repo)?;
        let touched = status
            .entries
            .iter()
            .flat_map(|e| std::iter::once(&e.path).chain(e.orig_path.as_ref()))
            .find(|p| owned.contains(p) || untracked.contains(p));
        if let Some(path) = touched {
            return Err(refuse(format!("changed again: {path}")));
        }
        if let Some(clash) = untracked.iter().find(|p| exists(dir, p)) {
            return Err(refuse(format!("already exists: {clash}")));
        }
        let failed = |e: GitError| GitError::Refused {
            kind: FailureKind::UnparkConflict,
            detail: e.detail(),
        };
        // stash 버전에 있는 경로: 작업 폴더에서 바뀐 것은 지우지 않은 것, 넣은 것만 바뀐 것은 지금 버전에 있던 것.
        let present: Vec<String> = owned
            .iter()
            .filter(|p| match worktree.iter().find(|(_, w)| w == *p) {
                Some((status, _)) => *status != 'D',
                None => staged.iter().any(|(s, i)| i == *p && *s != 'A'),
            })
            .cloned()
            .collect();
        if !owned.is_empty() {
            self.runner
                .run_with_stdin(
                    Some(dir),
                    &[
                        "--literal-pathspecs",
                        "reset",
                        "-q",
                        &format!("{oid}^2"),
                        "--pathspec-from-file=-",
                        "--pathspec-file-nul",
                    ],
                    &nul_separated(&owned),
                )
                .map_err(failed)?;
        }
        let scratch = Scratch::new()?;
        if !present.is_empty() {
            let file = scratch.path("work");
            let env = [("GIT_INDEX_FILE", file.as_os_str())];
            self.runner
                .run_env(Some(dir), &["read-tree", oid.as_str()], &env)
                .map_err(failed)?;
            self.runner
                .run_with_stdin_env(
                    Some(dir),
                    &["checkout-index", "-f", "-z", "--stdin"],
                    &nul_separated(&present),
                    &env,
                )
                .map_err(failed)?;
        }
        for path in owned.iter().filter(|p| !present.contains(p)) {
            remove_inside(dir, path);
        }
        if !untracked.is_empty() {
            let file = scratch.path("untracked");
            let env = [("GIT_INDEX_FILE", file.as_os_str())];
            self.runner
                .run_env(Some(dir), &["read-tree", &format!("{oid}^3")], &env)
                .map_err(failed)?;
            self.runner
                .run_env(Some(dir), &["checkout-index", "-f", "-a"], &env)
                .map_err(failed)?;
        }
        Ok(OpOutput {
            oid: Some(oid.clone()),
            ..OpOutput::default()
        })
    }

    /// 두 버전 사이에 바뀐 경로와 그 종류(`A`·`M`·`D`·`T`, 이름 바꿈은 두 경로로).
    fn name_status(&self, repo: &RepoPath, from: &str, to: &str) -> Result<Vec<(char, String)>> {
        let mut args = vec!["diff", "--name-status", "-z", "--no-renames"];
        push_revs(&mut args, Cmd::Diff, self.git_version(), &[from, to]);
        let out = self.runner.run(Some(repo.as_path()), &args)?;
        let text = out.stdout_lossy();
        let mut fields = text.split('\0').filter(|f| !f.is_empty());
        let mut changes = Vec::new();
        while let (Some(kind), Some(path)) = (fields.next(), fields.next()) {
            if !safe_relative(path) {
                return Err(GitError::Parse(format!("stash 경로: {path}")));
            }
            changes.push((kind.chars().next().unwrap_or('M'), path.to_string()));
        }
        Ok(changes)
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
                    remove_inside(dir, path);
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

/// 저장소 안 상대 경로인지: 절대 경로, 드라이브, `\`, `..`, 빈 조각, `.git`을 가리키는 조각을
/// 거부한다. 받은 저장소의 경로라 OS와 상관없이 같은 규칙을 쓴다(D-034).
pub fn safe_relative(path: &str) -> bool {
    let b = path.as_bytes();
    let drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    !(path.starts_with('/')
        || path.contains('\\')
        || path.contains('\0')
        || drive
        || path
            .split('/')
            .any(|c| c.is_empty() || c == ".." || names_git_dir(c)))
}

/// Windows에서 `.git` 폴더로 풀리는 이름: `.git`, 끝의 점·공백(`.git.`, `.git `), 대체 데이터 흐름
/// (`.git:$INDEX_ALLOCATION`), 8.3 짧은 이름(`GIT~1`). 대소문자는 가리지 않는다.
fn names_git_dir(component: &str) -> bool {
    let stem = component.split(':').next().unwrap_or(component);
    let stem = stem.trim_end_matches(['.', ' ']);
    if stem.eq_ignore_ascii_case(".git") {
        return true;
    }
    let lower = stem.to_ascii_lowercase();
    lower
        .strip_prefix("git~")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
}

fn exists(dir: &Path, rel: &str) -> bool {
    dir.join(rel).symlink_metadata().is_ok()
}

/// 작업 폴더 안의 파일 하나를 지운다. 경로 중간 조각이 링크(Windows는 재해석 지점 포함)이거나
/// 폴더가 아니면 따라가지 않고 그만둔다. 마지막 조각이 링크면 링크만 지운다(D-037).
fn remove_inside(dir: &Path, rel: &str) {
    if !safe_relative(rel) {
        return;
    }
    let mut at = dir.to_path_buf();
    let mut parts = rel.split('/').peekable();
    while let Some(part) = parts.next() {
        at.push(part);
        let Ok(meta) = at.symlink_metadata() else {
            return;
        };
        if parts.peek().is_some() {
            // `FileType::is_dir`은 링크(정션 포함)이면 거짓이다.
            if !meta.file_type().is_dir() {
                return;
            }
        } else if !meta.file_type().is_dir() {
            let _ = std::fs::remove_file(&at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{remove_inside, safe_relative};
    use crate::testutil::{self, git};

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
            ".git./config",
            ".GIT /config",
            ".git:$INDEX_ALLOCATION/config",
            "GIT~1/config",
            "a/git~12",
        ] {
            assert!(!safe_relative(bad), "{bad}");
        }
        for good in [
            "a.txt",
            "폴더/새 파일.txt",
            ".gitignore",
            "a..b",
            "git~",
            "git~a",
            ".github/x",
        ] {
            assert!(safe_relative(good), "{good}");
        }
    }

    #[test]
    fn removing_does_not_follow_a_linked_folder() {
        let repo = testutil::init_repo();
        let p = repo.path();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("victim.txt"), "남아야 함").unwrap();
        // git이 추적하는 링크를 꺼내 작업 폴더에 `lnk -> 바깥 폴더`를 만든다(Linux). Windows의
        // core.symlinks=false에서는 보통 파일이 생기고, 그래도 지우면 안 된다.
        let target = p.join("target.txt");
        std::fs::write(&target, outside.path().to_string_lossy().as_bytes()).unwrap();
        let blob = git(p, &["hash-object", "-w", "target.txt"]);
        let blob = blob.trim();
        git(
            p,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("120000,{blob},lnk"),
            ],
        );
        git(p, &["checkout", "--", "lnk"]);
        remove_inside(p, "lnk/victim.txt");
        assert!(outside.path().join("victim.txt").exists());
        // 보통 파일은 지운다.
        std::fs::create_dir_all(p.join("폴더")).unwrap();
        std::fs::write(p.join("폴더/새 파일.txt"), "x").unwrap();
        remove_inside(p, "폴더/새 파일.txt");
        assert!(!p.join("폴더/새 파일.txt").exists());
    }
}
