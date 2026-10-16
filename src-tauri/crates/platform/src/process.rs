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

/// `prepare_killable`로 띄운 자식과 손자들에게 끝내 달라고 알린다(취소·멈춤, D-080). git이 잠금
/// 파일을 지우고 끝날 틈을 준다. 끝나지 않으면 부르는 쪽이 잠시 뒤 `kill_tree`를 부른다.
/// Linux: 프로세스 그룹에 SIGTERM. Windows: `taskkill /T`(강제 아님).
pub fn terminate_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process_group};
        if let Some(pid) = Pid::from_raw(child.id() as i32) {
            let _ = kill_process_group(pid, Signal::TERM);
        }
    }
    #[cfg(windows)]
    {
        let mut taskkill = Command::new("taskkill");
        configure_child(&mut taskkill);
        let _ = taskkill
            .args(["/T", "/PID", &child.id().to_string()])
            .output();
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

/// 시스템에 도는 git 프로세스의 pid(다른 사용자 것은 OS가 보여 주는 만큼). 알아내지 못하면
/// `None`. 잠금 파일이 정말 버려진 것인지 가를 때 쓴다(D-081).
/// Linux: `/proc/*/comm`이 `git`이거나 `git-`로 시작. Windows: `tasklist`의 `git.exe`, `git-*.exe`.
pub fn running_git_pids() -> Option<Vec<u32>> {
    #[cfg(target_os = "linux")]
    {
        let mut pids = Vec::new();
        for entry in std::fs::read_dir("/proc").ok()?.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<u32>().ok())
            else {
                continue;
            };
            // 그 사이 끝난 프로세스는 읽히지 않는다.
            let Ok(comm) = std::fs::read_to_string(entry.path().join("comm")) else {
                continue;
            };
            if is_git_name(comm.trim_end()) {
                pids.push(pid);
            }
        }
        Some(pids)
    }
    #[cfg(windows)]
    {
        let mut tasklist = Command::new("tasklist");
        configure_child(&mut tasklist);
        let out = tasklist
            .args(["/FI", "IMAGENAME eq git*", "/FO", "CSV", "/NH"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        Some(parse_tasklist(&String::from_utf8_lossy(&out.stdout)))
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        None
    }
}

/// git 본체나 git이 띄우는 도우미(`git-remote-https`, comm은 15자로 잘림)인지.
#[cfg(any(target_os = "linux", windows, test))]
fn is_git_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);
    name == "git" || name.starts_with("git-")
}

/// `"git.exe","1234","Console","1","10,000 K"` 줄들. 맞는 것이 없다는 안내 줄(번역됨)은 건너뛴다.
#[cfg(any(windows, test))]
fn parse_tasklist(out: &str) -> Vec<u32> {
    out.lines()
        .filter_map(|line| {
            let mut fields = line.split("\",\"");
            let name = fields.next()?.strip_prefix('"')?;
            let pid = fields.next()?.parse().ok()?;
            is_git_name(name).then_some(pid)
        })
        .collect()
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
        use std::io::{BufRead, Read};
        let git = crate::find_git().unwrap();
        let mut cmd = Command::new(git);
        // git 별칭이 sh를 띄우고 sh가 sleep을 띄운다(손자). 출력 파이프를 손자가 쥔다. sleep을 먼저 띄운 뒤
        // "ready"를 써서, 느린 러너에서도 끝내기 전에 손자가 이미 있게 한다(고정 대기는 Windows CI에서 모자랐다).
        cmd.args(["-c", "alias.slow=!sleep 30 & echo ready; wait", "slow"])
            .stdout(std::process::Stdio::piped());
        configure_child(&mut cmd);
        prepare_killable(&mut cmd);
        let mut child = cmd.spawn().unwrap();
        let mut out = std::io::BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        out.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "ready");
        let start = std::time::Instant::now();
        kill_tree(&mut child);
        let _ = child.wait();
        // 손자까지 끝나야 파이프가 닫혀 읽기가 바로 끝난다.
        let mut buf = Vec::new();
        out.read_to_end(&mut buf).unwrap();
        assert!(start.elapsed() < std::time::Duration::from_secs(10));
    }

    #[test]
    fn git_names_and_tasklist_lines() {
        assert!(is_git_name("git"));
        assert!(is_git_name("git-remote-http"));
        assert!(is_git_name("Git.exe"));
        assert!(!is_git_name("gitk"));
        assert!(!is_git_name("GitHubDesktop.exe"));
        let out = "\"git.exe\",\"1234\",\"Console\",\"1\",\"10,000 K\"\r\n\
                   \"GitHubDesktop.exe\",\"77\",\"Console\",\"1\",\"9 K\"\r\n\
                   \"git-remote-https.exe\",\"88\",\"Console\",\"1\",\"9 K\"\r\n";
        assert_eq!(parse_tasklist(out), vec![1234, 88]);
        assert!(parse_tasklist("정보: 지정된 조건에 맞는 작업이 없습니다.\r\n").is_empty());
    }

    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn running_git_is_seen() {
        let git = crate::find_git().unwrap();
        let mut cmd = Command::new(git);
        cmd.args(["-c", "alias.slow=!sleep 30", "slow"]);
        configure_child(&mut cmd);
        prepare_killable(&mut cmd);
        let mut child = cmd.spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        let pids = running_git_pids().unwrap();
        kill_tree(&mut child);
        let _ = child.wait();
        assert!(pids.contains(&child.id()), "{pids:?}");
    }

    #[cfg(unix)]
    #[test]
    fn terminate_tree_asks_the_group_to_end() {
        let git = crate::find_git().unwrap();
        let mut cmd = Command::new(git);
        cmd.args(["-c", "alias.slow=!sleep 30", "slow"]);
        configure_child(&mut cmd);
        prepare_killable(&mut cmd);
        let mut child = cmd.spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        terminate_tree(&mut child);
        let start = std::time::Instant::now();
        while child.try_wait().unwrap().is_none() {
            assert!(start.elapsed().as_secs() < 5, "SIGTERM으로 끝나야 한다");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}
