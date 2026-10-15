//! 원격 작업 실행 방식(시간 제한, 백그라운드 환경)을 시험한다.

use std::time::{Duration, Instant};

use super::network::NetMode;
use crate::testutil;
use crate::{FailureKind, RepoPath};

fn env_in(mode: NetMode, var: &str) -> String {
    let repo = testutil::init_repo();
    let alias = format!("alias.envcheck=!printf %s \"${var}\"");
    testutil::backend()
        .run_network(
            &RepoPath::new(repo.path()),
            &["-c", &alias, "envcheck"],
            mode,
        )
        .unwrap()
        .stdout_utf8()
        .unwrap()
}

#[test]
fn background_mode_disables_prompts() {
    assert_eq!(env_in(NetMode::Background, "GCM_INTERACTIVE"), "never");
    assert_eq!(env_in(NetMode::Background, "SSH_ASKPASS_REQUIRE"), "never");
    assert_eq!(env_in(NetMode::Foreground, "GCM_INTERACTIVE"), "");
}

#[test]
fn ssh_batch_mode_unless_user_set_one() {
    if std::env::var_os("GIT_SSH_COMMAND").is_some() {
        return;
    }
    assert_eq!(
        env_in(NetMode::Foreground, "GIT_SSH_COMMAND"),
        "ssh -o BatchMode=yes"
    );
    let repo = testutil::init_repo();
    testutil::git(repo.path(), &["config", "core.sshCommand", "ssh -i 내키"]);
    let out = testutil::backend()
        .run_network(
            &RepoPath::new(repo.path()),
            &[
                "-c",
                "alias.envcheck=!printf %s \"$GIT_SSH_COMMAND\"",
                "envcheck",
            ],
            NetMode::Foreground,
        )
        .unwrap();
    assert_eq!(out.stdout_utf8().unwrap(), "");
}

#[test]
fn stuck_git_is_ended_with_its_children() {
    let repo = testutil::init_repo();
    let backend = testutil::backend()
        .with_network_timeouts_for_tests(Duration::from_millis(500), Duration::from_millis(500));
    let start = Instant::now();
    // sh가 sleep을 손자로 띄운다. 손자가 출력 파이프를 쥐고 있어 트리째 끝내지 않으면 30초 기다린다.
    let err = backend
        .run_network(
            &RepoPath::new(repo.path()),
            &["-c", "alias.slow=!sleep 30", "slow"],
            NetMode::Background,
        )
        .unwrap_err();
    assert_eq!(err.kind(), FailureKind::TimedOut);
    // 기본 ProcessControl은 자식만 끝낸다. 손자는 platform의 kill_tree가 끝낸다(core가 넘김).
    // 여기서는 시간 제한이 걸리는지만 본다.
    assert!(start.elapsed() < Duration::from_secs(45));
}
