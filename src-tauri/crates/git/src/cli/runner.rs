use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use crate::{GitError, Result};

/// git을 실행하기 전에 OS별 설정(예: Windows 콘솔 창 숨기기)을 붙이는 함수.
pub type ConfigureCommand = Arc<dyn Fn(&mut Command) + Send + Sync>;

pub(crate) struct Runner {
    git: PathBuf,
    configure: ConfigureCommand,
}

pub(crate) struct GitOutput {
    pub stdout: Vec<u8>,
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
}

impl Runner {
    pub fn new(git: PathBuf, configure: ConfigureCommand) -> Self {
        Self { git, configure }
    }

    /// `cwd`에서 git을 실행하고, 성공하면 출력을 돌려준다.
    pub fn run(&self, cwd: Option<&Path>, args: &[&str]) -> Result<GitOutput> {
        let mut cmd = Command::new(&self.git);
        // 영어 원문 오류, 프롬프트 없음, 페이저 없음, 읽기 작업이 index 잠금을 잡지 않음.
        cmd.env("LC_ALL", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_PAGER", "cat")
            .env("GIT_OPTIONAL_LOCKS", "0")
            // 한글 파일명을 따옴표·8진수 이스케이프 없이 받는다.
            .args(["-c", "core.quotepath=off", "-c", "color.ui=never"])
            .args(args)
            .stdin(Stdio::null());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        (self.configure)(&mut cmd);

        let out = cmd.output().map_err(GitError::Spawn)?;
        if out.status.success() {
            Ok(GitOutput { stdout: out.stdout })
        } else {
            Err(GitError::Failed {
                args: args.iter().map(|a| a.to_string()).collect(),
                code: out.status.code(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    }
}
