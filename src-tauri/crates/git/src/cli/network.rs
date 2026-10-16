//! 원격과 통신하는 git 호출(D-023, D-041, D-045, D-066).
//!
//! 시간 제한이 지나면 프로세스 트리째 끝낸다. 백그라운드 원격 확인은 질문 창을 띄우지 않는다.

use std::ffi::OsStr;
use std::time::Duration;

use super::CliBackend;
use super::runner::{EnvChange, GitOutput};
use crate::{AuthMode, GitError, RepoPath, Result};

/// hiGit credential helper의 실행 파일 경로를 넘기는 환경 변수(D-111). 토큰은 넣지 않는다.
pub const HELPER_ENV: &str = "HIGIT_CREDENTIAL_HELPER";
/// helper를 부르는 설정. 경로는 환경 변수로만 넘겨 명령 문자열은 상수다(공백·따옴표가 든 경로도 안전).
const HELPER_CONFIG: &str = "credential.helper=!\"$HIGIT_CREDENTIAL_HELPER\" --git-credential";

/// 사용자가 누른 원격 작업의 시간 제한.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// 저절로 하는 원격 확인의 시간 제한.
pub const BACKGROUND_TIMEOUT: Duration = Duration::from_secs(60);
/// 첫 출력 뒤 이만큼 아무 출력이 없으면 멈춘 것으로 보고 끝낸다(D-080).
pub const STALL: Duration = Duration::from_secs(90);

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
        auth: AuthMode,
    ) -> Result<GitOutput> {
        self.run_network_input(repo, args, mode, auth, None)
    }

    /// `run_network`에 표준 입력을 준다(시험이 `credential fill`을 부를 때).
    pub(crate) fn run_network_input(
        &self,
        repo: &RepoPath,
        args: &[&str],
        mode: NetMode,
        auth: AuthMode,
        stdin: Option<&[u8]>,
    ) -> Result<GitOutput> {
        let dir = Some(repo.as_path());
        let helper = match auth {
            AuthMode::Helper => self.helper_program(),
            AuthMode::UserDefault => None,
        };
        let mut full: Vec<&str> = Vec::new();
        let mut env: Vec<(&str, EnvChange)> = Vec::new();
        if let Some(path) = &helper {
            // 이 호출에서만 사용자 helper 목록을 비우고 hiGit helper 하나를 둔다. 사용자 설정은
            // 고치지 않는다.
            full.extend(["-c", "credential.helper=", "-c", HELPER_CONFIG]);
            env.push((HELPER_ENV, EnvChange::Set(path.as_os_str())));
        }
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
        let scope = match mode {
            NetMode::Foreground => super::CancelScope::User,
            NetMode::Background => super::CancelScope::Background,
        };
        let cancel = self.cancel_flag(repo, scope);
        let observer = self.observer_for(repo);
        self.runner.run_watched(
            dir,
            &full,
            &env,
            stdin,
            &super::runner::Watch {
                timeout: Some(timeout),
                stall: Some(STALL),
                cancel: Some(&cancel),
                observer: observer.as_deref(),
            },
        )
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
