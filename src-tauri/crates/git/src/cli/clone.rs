//! 복제(D-115). 원격 작업과 같은 감시(시간 제한·멈춤·취소·진행)로 돌린다.
//!
//! cwd는 부모 폴더이고, 진행 알림·취소 표시의 키는 대상 폴더(`부모/이름`)다. 실패 뒤 폴더 정리는
//! core가 한다(git은 신호로 끝날 때만 스스로 지운다).

use std::time::Duration;

use super::CliBackend;
use super::args::{Cmd, push_revs};
use super::network::{STALL, push_helper, ssh_batch};
use super::runner::{EnvChange, GitOutput, Watch};
use crate::{AuthMode, RepoPath, Result};

/// 복제 전체의 시간 제한. 멈춤 감지(90초)는 원격 작업과 같다.
pub const CLONE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

impl CliBackend {
    pub(super) fn clone_repo(
        &self,
        parent: &RepoPath,
        url: &str,
        name: &str,
        auth: AuthMode,
    ) -> Result<GitOutput> {
        let target = RepoPath::new(parent.as_path().join(name));
        let helper = self.helper_for(auth);
        let mut full: Vec<&str> = Vec::new();
        let mut env: Vec<(&str, EnvChange)> = Vec::new();
        push_helper(&helper, &mut full, &mut env);
        if self.global_ssh_command()?.is_none() {
            env.push(ssh_batch());
        }
        // 서브모듈은 받지 않는다(그 안에서 명령이 돌 수 있다). 원격 이름은 사용자 설정과 상관없이
        // origin. 주소가 `-`로 시작해도 옵션으로 읽히지 않게 `--end-of-options` 뒤에 둔다.
        full.extend([
            "clone",
            "--progress",
            "--no-recurse-submodules",
            "--origin",
            "origin",
        ]);
        push_revs(&mut full, Cmd::Clone, self.git_version(), &[url, name]);
        let cancel = self.cancel_flag(&target, super::CancelScope::User);
        let observer = self.observer_for(&target);
        self.runner.run_watched(
            Some(parent.as_path()),
            &full,
            &env,
            None,
            &Watch {
                timeout: Some(self.clone_timeout),
                stall: Some(STALL),
                cancel: Some(&cancel),
                observer: observer.as_deref(),
            },
        )
    }
}
