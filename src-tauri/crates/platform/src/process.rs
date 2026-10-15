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
}
