//! 전역 이름·이메일 읽기·쓰기(D-125). 저장소 밖에서 돌려 저장소 설정이 섞이지 않게 하고,
//! `GIT_CONFIG_GLOBAL` 같은 사용자 환경 변수는 그대로 따른다.

use super::CliBackend;
use crate::{GitError, GitIdentity, GlobalOp, OpOutput, Result};

/// 읽은 값의 상한. 이보다 길면 없는 것으로 본다.
const MAX_VALUE: usize = 4096;
const KEYS: [&str; 2] = ["user.name", "user.email"];

impl CliBackend {
    pub(super) fn identity_impl(&self) -> Result<GitIdentity> {
        let mut values = [None, None];
        for (slot, key) in values.iter_mut().zip(KEYS) {
            *slot = match self.config_get("--global", key)? {
                Some(v) => Some(v),
                None => self.config_get("--system", key)?,
            };
        }
        let [name, email] = values;
        Ok(GitIdentity { name, email })
    }

    /// `git config <scope> --includes --get <키>`. 없으면(종료 코드 1) `None`, 그 밖의 실패는 오류.
    fn config_get(&self, scope: &str, key: &str) -> Result<Option<String>> {
        match self
            .runner
            .run(None, &["config", scope, "--includes", "--get", key])
        {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let value = text.trim();
                Ok((!value.is_empty() && value.len() <= MAX_VALUE).then(|| value.to_string()))
            }
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub(super) fn run_global_impl(&self, op: &GlobalOp) -> Result<OpOutput> {
        match op {
            GlobalOp::SetIdentity { name, email } => {
                // 이름이 먼저다. 이메일을 쓰지 못하면 이름은 그대로 두고 오류(되돌리지 않음).
                self.runner
                    .run(None, &["config", "--global", "user.name", name])?;
                let out = self
                    .runner
                    .run(None, &["config", "--global", "user.email", email])?;
                Ok(OpOutput {
                    stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                    oid: None,
                })
            }
        }
    }
}
