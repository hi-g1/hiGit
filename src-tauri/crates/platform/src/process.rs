use std::process::Command;

/// git 같은 자식 프로세스를 실행하기 전에 OS별 설정을 붙인다.
///
/// Windows: 콘솔 창이 깜빡이지 않게 `CREATE_NO_WINDOW`를 준다. 그 밖의 OS: 아무것도 하지 않는다.
pub fn configure_child(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

/// 시간 제한으로 끝낼 수 있게 자식을 새 프로세스 그룹으로 띄운다(D-041, D-066).
/// Linux: 새 프로세스 그룹. Windows: 따로 할 일 없음(끝낼 때 `taskkill /T`로 트리째).
pub fn prepare_killable(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = cmd;
    }
}

/// `prepare_killable`로 띄운 자식과 그 손자들을 모두 끝낸다. git은 ssh·sh 같은 손자를 띄운다.
pub fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process_group};
        if let Some(pid) = Pid::from_raw(child.id() as i32) {
            let _ = kill_process_group(pid, Signal::KILL);
        }
    }
    #[cfg(windows)]
    {
        let mut taskkill = Command::new("taskkill");
        configure_child(&mut taskkill);
        let _ = taskkill
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .output();
    }
    let _ = child.kill();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_command_still_runs() {
        let git = crate::find_git().unwrap();
        let mut cmd = Command::new(git);
        cmd.arg("--version");
        configure_child(&mut cmd);
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).starts_with("git version"));
    }

    #[test]
    fn kill_tree_ends_grandchildren() {
        let git = crate::find_git().unwrap();
        let mut cmd = Command::new(git);
        // git 별칭이 sh를 띄우고 sh가 sleep을 띄운다(손자). 출력 파이프를 손자가 쥔다.
        cmd.args(["-c", "alias.slow=!sleep 30", "slow"])
            .stdout(std::process::Stdio::piped());
        configure_child(&mut cmd);
        prepare_killable(&mut cmd);
        let mut child = cmd.spawn().unwrap();
        let mut out = child.stdout.take().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        let start = std::time::Instant::now();
        kill_tree(&mut child);
        let _ = child.wait();
        // 손자까지 끝나야 파이프가 닫혀 읽기가 바로 끝난다.
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut out, &mut buf).unwrap();
        assert!(start.elapsed() < std::time::Duration::from_secs(10));
    }
}
