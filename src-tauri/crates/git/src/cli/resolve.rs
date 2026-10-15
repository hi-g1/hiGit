//! 충돌 해결 쓰기와 덮어쓰기 전 백업(충돌 해결 화면, D-069, D-070).

use std::ffi::OsStr;

use super::CliBackend;
use super::merge_check::Scratch;
use super::park::safe_relative;
use super::runner::{GitOutput, nul_separated};
use crate::{GitError, Oid, OpOutput, RepoPath, Result};

fn refuse_path(path: &str) -> GitError {
    GitError::Parse(format!("저장소 밖 경로: {path}"))
}

impl CliBackend {
    /// 블롭을 만들어 index에 넣고(같은 경로의 stage 1·2·3이 지워진다) 작업 폴더 파일을 다시 쓴다.
    pub(super) fn resolve_file(
        &self,
        repo: &RepoPath,
        path: &str,
        mode: &str,
        bytes: &[u8],
    ) -> Result<GitOutput> {
        if !safe_relative(path) {
            return Err(refuse_path(path));
        }
        if !matches!(mode, "100644" | "100755" | "120000") {
            return Err(GitError::Parse(format!("모드: {mode}")));
        }
        let dir = Some(repo.as_path());
        let oid = self
            .runner
            .run_with_stdin(
                dir,
                &["hash-object", "-w", "--no-filters", "--stdin"],
                bytes,
            )?
            .stdout_utf8()?;
        let oid = Oid::parse(oid.trim()).ok_or_else(|| GitError::Parse("hash-object".into()))?;
        let mut info = format!("{mode} {oid}\t{path}").into_bytes();
        info.push(0);
        self.runner
            .run_with_stdin(dir, &["update-index", "-z", "--index-info"], &info)?;
        // 작업 폴더 파일은 git이 사용자의 줄바꿈·필터 설정으로 만든다. 링크를 건너 쓰지 않는다.
        self.runner.run_with_stdin(
            dir,
            &["checkout-index", "-f", "-z", "--stdin"],
            &nul_separated(&[path.to_string()]),
        )
    }

    /// 지우는 쪽으로 해결한다.
    pub(super) fn resolve_delete(&self, repo: &RepoPath, path: &str) -> Result<GitOutput> {
        if !safe_relative(path) {
            return Err(refuse_path(path));
        }
        self.runner.run_with_stdin(
            Some(repo.as_path()),
            &[
                "--literal-pathspecs",
                "rm",
                "-f",
                "-q",
                "--pathspec-from-file=-",
                "--pathspec-file-nul",
            ],
            &nul_separated(&[path.to_string()]),
        )
    }

    /// 한쪽 그대로 해결한다. 그쪽 파일을 작업 폴더에 쓰고 index에 넣는다.
    pub(super) fn resolve_side(
        &self,
        repo: &RepoPath,
        path: &str,
        ours: bool,
    ) -> Result<GitOutput> {
        if !safe_relative(path) {
            return Err(refuse_path(path));
        }
        let dir = Some(repo.as_path());
        let list = nul_separated(&[path.to_string()]);
        let side = if ours { "--ours" } else { "--theirs" };
        self.runner.run_with_stdin(
            dir,
            &[
                "--literal-pathspecs",
                "checkout",
                side,
                "--pathspec-from-file=-",
                "--pathspec-file-nul",
            ],
            &list,
        )?;
        self.runner.run_with_stdin(
            dir,
            &[
                "--literal-pathspecs",
                "add",
                "--pathspec-from-file=-",
                "--pathspec-file-nul",
            ],
            &list,
        )
    }

    /// 작업 폴더의 `paths` 내용을 stash 모양 커밋으로 만든다. 임시 index를 써서 사용자의 index는
    /// 건드리지 않는다. 백업 커밋은 원격에 올라가지 않으므로 사용자 이름이 없어도 만들 수 있게
    /// 앱 이름으로 적고 서명하지 않는다.
    pub(super) fn snapshot_files(
        &self,
        repo: &RepoPath,
        paths: &[String],
        message: &str,
    ) -> Result<OpOutput> {
        if let Some(bad) = paths.iter().find(|p| !safe_relative(p)) {
            return Err(refuse_path(bad));
        }
        let dir = Some(repo.as_path());
        let head = crate::GitBackend::resolve_ref(self, repo, "HEAD")?
            .ok_or_else(|| GitError::Parse("HEAD 없음".into()))?;
        let scratch = Scratch::new()?;
        let index = scratch.path("index");
        let ident: [(&str, &OsStr); 4] = [
            ("GIT_AUTHOR_NAME", OsStr::new("hiGit")),
            ("GIT_AUTHOR_EMAIL", OsStr::new("higit@localhost")),
            ("GIT_COMMITTER_NAME", OsStr::new("hiGit")),
            ("GIT_COMMITTER_EMAIL", OsStr::new("higit@localhost")),
        ];
        let temp_index = [("GIT_INDEX_FILE", index.as_os_str())];
        self.runner
            .run_env(dir, &["read-tree", head.as_str()], &temp_index)?;
        self.runner.run_with_stdin_env(
            dir,
            &["update-index", "--add", "--remove", "-z", "--stdin"],
            &nul_separated(paths),
            &temp_index,
        )?;
        let tree = self
            .runner
            .run_env(dir, &["write-tree"], &temp_index)?
            .stdout_utf8()?;
        let head_tree = format!("{head}^{{tree}}");
        let branch =
            crate::GitBackend::current_branch(self, repo)?.unwrap_or_else(|| "HEAD".into());
        let index_commit = self
            .runner
            .run_env(
                dir,
                &[
                    "commit-tree",
                    "--no-gpg-sign",
                    &head_tree,
                    "-p",
                    head.as_str(),
                    "-m",
                    &format!("index on {branch}: {message}"),
                ],
                &ident,
            )?
            .stdout_utf8()?;
        let snapshot = self
            .runner
            .run_env(
                dir,
                &[
                    "commit-tree",
                    "--no-gpg-sign",
                    tree.trim(),
                    "-p",
                    head.as_str(),
                    "-p",
                    index_commit.trim(),
                    "-m",
                    &format!("On {branch}: {message}"),
                ],
                &ident,
            )?
            .stdout_utf8()?;
        Ok(OpOutput {
            oid: Oid::parse(snapshot.trim()),
            ..OpOutput::default()
        })
    }
}
