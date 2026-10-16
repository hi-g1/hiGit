//! 빌드된 앱을 `--git-credential`로 실행한다(D-111). 창을 띄우지 않고, github.com https가 아니거나
//! `get`이 아니면 키체인을 읽지 않고 아무것도 쓰지 않은 채 0으로 끝나야 한다.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::Write;
use std::process::{Command, Stdio};

fn helper(args: &[&str], input: &str) -> (i32, Vec<u8>, Vec<u8>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_higit"))
        .arg("--git-credential")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(-1), out.stdout, out.stderr)
}

#[test]
fn silent_for_other_hosts_and_operations() {
    let cases: &[(&[&str], &str)] = &[
        (&["get"], "protocol=https\nhost=gitlab.com\n\n"),
        (&["get"], "protocol=https\nhost=github.com.evil.example\n\n"),
        (&["get"], "protocol=http\nhost=github.com\n\n"),
        (
            &["get"],
            "protocol=https\nhost=github.com\nusername=mona\n\n",
        ),
        (
            &["store"],
            "protocol=https\nhost=github.com\nusername=x\npassword=y\n\n",
        ),
        (&["erase"], "protocol=https\nhost=github.com\n\n"),
        (&["--version"], ""),
        (&[], ""),
    ];
    for (args, input) in cases {
        let (code, stdout, stderr) = helper(args, input);
        assert_eq!(code, 0, "{args:?}");
        assert!(
            stdout.is_empty(),
            "{args:?}: {}",
            String::from_utf8_lossy(&stdout)
        );
        assert!(
            stderr.is_empty(),
            "{args:?}: {}",
            String::from_utf8_lossy(&stderr)
        );
    }
}
