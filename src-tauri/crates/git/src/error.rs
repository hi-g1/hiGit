use std::io;

use crate::failure::{self, FailureKind};

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// git 실행 파일을 실행하지 못했다(경로가 틀렸거나 권한이 없음).
    #[error("git을 실행하지 못했습니다")]
    Spawn(#[source] io::Error),

    /// git이 0이 아닌 코드로 끝났다. `stderr`는 원문 그대로 보존해 "자세히 보기"에 보여준다.
    #[error("git 명령이 실패했습니다: git {}", .args.join(" "))]
    Failed {
        args: Vec<String>,
        code: Option<i32>,
        /// 일부 명령(예: commit의 "nothing to commit")은 실패 이유를 stdout에 쓴다.
        stdout: String,
        stderr: String,
    },

    /// git을 부르기 전이나 실패를 정리한 뒤 이 크레이트가 스스로 멈췄다.
    #[error("git 작업을 하지 않았습니다: {detail}")]
    Refused { kind: FailureKind, detail: String },

    /// 시간 제한이 지나 git을 끝냈다(원격·백그라운드 작업, D-041).
    #[error("git 명령이 시간 안에 끝나지 않았습니다: git {}", .args.join(" "))]
    TimedOut { args: Vec<String> },

    /// git 출력이 기대한 기계용 형식과 다르다.
    #[error("git 출력을 읽지 못했습니다: {0}")]
    Parse(String),
}

impl GitError {
    /// 실패 종류(D-021). git 실행 자체가 안 되었거나 출력이 이상하면 `Other`.
    pub fn kind(&self) -> FailureKind {
        match self {
            GitError::Failed { stdout, stderr, .. } => failure::classify(stdout, stderr),
            GitError::Refused { kind, .. } => *kind,
            GitError::TimedOut { .. } => FailureKind::TimedOut,
            _ => FailureKind::Other,
        }
    }

    /// 사용자에게 "자세히 보기"로 보여줄 원문.
    pub fn detail(&self) -> String {
        match self {
            GitError::Spawn(e) => e.to_string(),
            GitError::Failed { stdout, stderr, .. } => {
                if stderr.trim().is_empty() {
                    stdout.clone()
                } else {
                    stderr.clone()
                }
            }
            GitError::Refused { detail, .. } => detail.clone(),
            GitError::TimedOut { args } => format!("timed out: git {}", args.join(" ")),
            GitError::Parse(msg) => msg.clone(),
        }
    }
}
