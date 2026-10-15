use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;

use crate::{GitError, Result};

/// git을 실행하기 전에 OS별 설정(예: Windows 콘솔 창 숨기기)을 붙이는 함수.
pub type ConfigureCommand = Arc<dyn Fn(&mut Command) + Send + Sync>;

pub(crate) struct Runner {
    git: PathBuf,
    configure: ConfigureCommand,
}

#[derive(Debug)]
pub(crate) struct GitOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl GitOutput {
    pub fn stdout_utf8(&self) -> Result<String> {
        String::from_utf8(self.stdout.clone())
            .map_err(|e| GitError::Parse(format!("UTF-8이 아닌 출력: {e}")))
    }

    /// 파일 이름처럼 UTF-8이 아닐 수도 있는 출력. 읽을 수 없는 바이트는 대체 문자로 바꾼다.
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

impl Runner {
    pub fn new(git: PathBuf, configure: ConfigureCommand) -> Self {
        Self { git, configure }
    }

    /// `cwd`에서 git을 실행하고, 성공하면 출력을 돌려준다. stdin은 막는다.
    pub fn run(&self, cwd: Option<&Path>, args: &[&str]) -> Result<GitOutput> {
        let mut cmd = self.command(cwd, args);
        cmd.stdin(Stdio::null());
        let out = cmd.output().map_err(GitError::Spawn)?;
        finish(args, out)
    }

    /// `input`을 stdin으로 넘겨 git을 실행한다(예: `--pathspec-from-file=-`).
    pub fn run_with_stdin(
        &self,
        cwd: Option<&Path>,
        args: &[&str],
        input: &[u8],
    ) -> Result<GitOutput> {
        let mut cmd = self.command(cwd, args);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(GitError::Spawn)?;
        // 입력을 다른 스레드에서 써야 큰 입력에서 출력 파이프가 차서 서로 기다리는 일이 없다.
        let mut stdin = child.stdin.take().expect("stdin은 piped");
        let input = input.to_vec();
        let writer = std::thread::spawn(move || stdin.write_all(&input));
        let out = child.wait_with_output().map_err(GitError::Spawn)?;
        // git이 입력을 다 읽기 전에 끝나면(인자 오류 등) 쓰기가 실패할 수 있다. 결과는 종료 코드로 판단한다.
        let _ = writer.join();
        finish(args, out)
    }

    fn command(&self, cwd: Option<&Path>, args: &[&str]) -> Command {
        let mut cmd = Command::new(&self.git);
        // OS별 설정을 먼저 붙이고, 아래 앱 고정 설정이 마지막에 이기게 한다.
        (self.configure)(&mut cmd);
        // 영어 원문 오류(D-021), 프롬프트 없음, 페이저 없음, 읽기 작업이 index 잠금을 잡지 않음.
        // LANGUAGE는 gettext에서 LC_ALL보다 앞설 수 있어 지운다.
        cmd.env("LC_ALL", "C")
            .env_remove("LANGUAGE")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_PAGER", "cat")
            .env("GIT_OPTIONAL_LOCKS", "0")
            // 한글 파일명을 따옴표·8진수 이스케이프 없이 받는다.
            .args(["-c", "core.quotepath=off", "-c", "color.ui=never"])
            .args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd
    }
}

fn finish(args: &[&str], out: Output) -> Result<GitOutput> {
    if out.status.success() {
        Ok(GitOutput {
            stdout: out.stdout,
            stderr: out.stderr,
        })
    } else {
        Err(GitError::Failed {
            args: args.iter().map(|a| a.to_string()).collect(),
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

/// 경로 목록을 `--pathspec-from-file=- --pathspec-file-nul`용 stdin 바이트로 만든다.
/// 명령줄로 넘기지 않으므로 Windows 명령줄 길이 제한(32K)에 걸리지 않는다.
pub(crate) fn nul_separated(paths: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for path in paths {
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
    }
    bytes
}
