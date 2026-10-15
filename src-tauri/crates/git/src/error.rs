use std::io;

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
        stderr: String,
    },

    /// git 출력이 기대한 기계용 형식과 다르다.
    #[error("git 출력을 읽지 못했습니다: {0}")]
    Parse(String),
}

impl GitError {
    /// 사용자에게 "자세히 보기"로 보여줄 원문.
    pub fn detail(&self) -> String {
        match self {
            GitError::Spawn(e) => e.to_string(),
            GitError::Failed { stderr, .. } => stderr.clone(),
            GitError::Parse(msg) => msg.clone(),
        }
    }
}
