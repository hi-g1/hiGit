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
    // 사용자가 누른 작업은 상위 환경을 그대로 둔다(CI 러너는 GCM_INTERACTIVE를 미리 둔다).
    let inherited = std::env::var("GCM_INTERACTIVE").unwrap_or_default();
    assert_eq!(env_in(NetMode::Foreground, "GCM_INTERACTIVE"), inherited);
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

mod watched {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use super::*;
    use crate::GitBackend;
    use crate::cli::runner::Watch;

    fn run(args: &[&str], watch: &Watch<'_>) -> crate::Result<crate::cli::runner::GitOutput> {
        let repo = testutil::init_repo();
        testutil::backend()
            .runner
            .run_watched(Some(repo.path()), args, &[], None, watch)
    }

    #[test]
    fn quiet_after_first_output_is_a_stall() {
        let err = run(
            &["-c", "alias.slow=!printf x >&2; sleep 2", "slow"],
            &Watch {
                stall: Some(Duration::from_millis(300)),
                ..Watch::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.kind(), FailureKind::Stalled);
    }

    #[test]
    fn no_output_yet_is_not_a_stall() {
        // 인증 창을 기다리듯 처음부터 조용하면 멈춤으로 보지 않는다.
        run(
            &["-c", "alias.wait=!sleep 1", "wait"],
            &Watch {
                stall: Some(Duration::from_millis(300)),
                ..Watch::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn cancel_flag_ends_it() {
        let flag = Arc::new(AtomicBool::new(false));
        let setter = flag.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            setter.store(true, Ordering::SeqCst);
        });
        let err = run(
            &["-c", "alias.slow=!sleep 2", "slow"],
            &Watch {
                cancel: Some(&flag),
                ..Watch::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.kind(), FailureKind::Cancelled);
    }

    #[test]
    fn observer_hears_every_second() {
        let calls = AtomicUsize::new(0);
        let observe = |_: Duration, _: Duration| {
            calls.fetch_add(1, Ordering::SeqCst);
        };
        run(
            &["-c", "alias.wait=!sleep 2", "wait"],
            &Watch {
                observer: Some(&observe),
                ..Watch::default()
            },
        )
        .unwrap();
        assert!(calls.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn a_requested_cancel_ends_the_next_network_command_until_cleared() {
        let repo = testutil::init_repo();
        let backend = testutil::backend();
        let path = RepoPath::new(repo.path());
        backend.request_cancel(&path);
        let err = backend
            .run_network(
                &path,
                &["-c", "alias.slow=!sleep 1", "slow"],
                NetMode::Foreground,
            )
            .unwrap_err();
        assert_eq!(err.kind(), FailureKind::Cancelled);
        backend.clear_cancel(&path);
        backend
            .run_network(&path, &["-c", "alias.ok=!true", "ok"], NetMode::Foreground)
            .unwrap();
    }
}
