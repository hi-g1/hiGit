//! 원격과 통신하는 git 호출(D-023, D-041, D-045, D-066).
//!
//! 시간 제한이 지나면 프로세스 트리째 끝낸다. 백그라운드 원격 확인은 질문 창을 띄우지 않는다.

use std::ffi::OsStr;
use std::time::Duration;

use super::CliBackend;
use super::runner::{EnvChange, GitOutput};
use crate::{GitError, RepoPath, Result};

/// 사용자가 누른 원격 작업의 시간 제한.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// 저절로 하는 원격 확인의 시간 제한.
pub const BACKGROUND_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    /// 사용자가 눌렀다. 자격 증명 도우미 창(Windows GCM 등)은 막지 않는다(D-023).
    Foreground,
    /// 저절로 한다. 어떤 질문 창도 띄우지 않는다(D-045).
    Background,
}

impl CliBackend {
    /// 원격 작업용 실행. `args`는 git 하위 명령부터.
    pub(crate) fn run_network(
        &self,
        repo: &RepoPath,
        args: &[&str],
        mode: NetMode,
    ) -> Result<GitOutput> {
        let dir = Some(repo.as_path());
        let mut full: Vec<&str> = Vec::new();
        let mut env: Vec<(&str, EnvChange)> = Vec::new();
        // 사용자가 SSH 명령을 정하지 않았으면 질문 대신 실패하게 한다(D-023).
        if self.user_ssh_command(repo)?.is_none() {
            env.push((
                "GIT_SSH_COMMAND",
                EnvChange::Set(OsStr::new("ssh -o BatchMode=yes")),
            ));
        }
        let timeout = match mode {
            NetMode::Foreground => self.timeouts.0,
            NetMode::Background => {
                full.extend([
                    "-c",
                    "credential.interactive=never",
                    "-c",
                    "gc.auto=0",
                    "-c",
                    "maintenance.auto=false",
                ]);
                env.push(("GCM_INTERACTIVE", EnvChange::Set(OsStr::new("never"))));
                env.push(("SSH_ASKPASS_REQUIRE", EnvChange::Set(OsStr::new("never"))));
                env.push(("GIT_ASKPASS", EnvChange::Remove));
                env.push(("SSH_ASKPASS", EnvChange::Remove));
                self.timeouts.1
            }
        };
        full.extend_from_slice(args);
        self.runner.run_limited(dir, &full, &env, timeout)
    }

    /// 사용자가 정한 SSH 명령(환경 변수 `GIT_SSH_COMMAND`나 설정 `core.sshCommand`).
    fn user_ssh_command(&self, repo: &RepoPath) -> Result<Option<String>> {
        if let Some(v) = std::env::var_os("GIT_SSH_COMMAND")
            && !v.is_empty()
        {
            return Ok(Some(v.to_string_lossy().into_owned()));
        }
        match self.runner.run(
            Some(repo.as_path()),
            &["config", "--get", "core.sshCommand"],
        ) {
            Ok(out) => Ok(Some(out.stdout_lossy().trim().to_string())),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
