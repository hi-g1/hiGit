use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{GitError, Result};

/// git을 실행하기 전에 OS별 설정(예: Windows 콘솔 창 숨기기)을 붙이는 함수.
pub type ConfigureCommand = Arc<dyn Fn(&mut Command) + Send + Sync>;

/// 시간 제한으로 끝낼 수 있게 프로세스를 띄우고 트리째 끝내는 방법. platform이 준다(D-041).
#[derive(Clone)]
pub struct ProcessControl {
    /// 띄우기 전에 붙인다(Linux: 새 프로세스 그룹).
    pub prepare: ConfigureCommand,
    /// 자식과 손자를 모두 끝낸다.
    pub kill: Arc<dyn Fn(&mut Child) + Send + Sync>,
    /// 끝내 달라고 알린다(취소·멈춤). 2초 안에 끝나지 않으면 `kill`을 부른다.
    pub terminate: Arc<dyn Fn(&mut Child) + Send + Sync>,
}

impl Default for ProcessControl {
    /// 테스트용 기본값: 자식만 끝낸다.
    fn default() -> Self {
        Self {
            prepare: Arc::new(|_| {}),
            kill: Arc::new(|child: &mut Child| {
                let _ = child.kill();
            }),
            terminate: Arc::new(|child: &mut Child| {
                let _ = child.kill();
            }),
        }
    }
}

/// `run_watched`의 감시 조건.
#[derive(Default)]
pub(crate) struct Watch<'a> {
    pub timeout: Option<Duration>,
    /// 첫 출력 뒤 이만큼 출력이 없으면 멈춘 것으로 본다.
    pub stall: Option<Duration>,
    pub cancel: Option<&'a std::sync::atomic::AtomicBool>,
    /// 1초마다 (경과, 마지막 출력 뒤 시간).
    pub observer: Option<&'a (dyn Fn(Duration, Duration) + Send + Sync)>,
}

/// 호출 하나에만 바꾸는 환경 변수.
#[derive(Debug, Clone, Copy)]
pub(crate) enum EnvChange<'a> {
    Set(&'a OsStr),
    Remove,
}

/// 모든 git 호출에 허용하는 전송 방식(D-035). `ext::`, `file://`, 로컬 경로 원격은 막는다.
pub(crate) const ALLOWED_PROTOCOLS: &str = "https:ssh:git";

/// 상위 프로세스에서 물려받으면 엉뚱한 저장소나 설정을 쓰게 하는 변수(D-052).
const REPO_LOCATION_VARS: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
    // 부모 환경이 몰래 설정을 끼워 넣는 길. 앱은 필요한 설정을 `-c`로 직접 준다.
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_EXTERNAL_DIFF",
];

pub(crate) struct Runner {
    git: PathBuf,
    configure: ConfigureCommand,
    allowed_protocols: &'static str,
    pub(crate) process: ProcessControl,
    /// 저장소 폴더별로 모든 호출에 붙이는 `-c 키=값`(믿지 않기로 한 저장소의 실행 설정 끄기, D-039).
    overrides: std::sync::Mutex<std::collections::HashMap<PathBuf, Vec<String>>>,
}

#[derive(Debug)]
pub(crate) struct GitOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// 상한까지만 읽은 출력.
pub(crate) struct Capped {
    pub stdout: Vec<u8>,
    /// 상한에 걸려 뒤를 읽지 않았다.
    pub truncated: bool,
}

impl GitOutput {
    pub fn stdout_utf8(&self) -> Result<String> {
        String::from_utf8(self.stdout.clone())
            .map_err(|e| GitError::Parse(format!("UTF-8이 아닌 출력: {e}")))
    }

    /// 파일 이름처럼 UTF-8이 아닐 수도 있는 출력. 읽을 수 없는 바이트는 대체 문자로 바꾼다.
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

impl Runner {
    pub fn new(git: PathBuf, configure: ConfigureCommand) -> Self {
        Self {
            git,
            configure,
            allowed_protocols: ALLOWED_PROTOCOLS,
            process: ProcessControl::default(),
            overrides: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// `repo` 폴더에서 도는 모든 호출에 `-c 키=값`을 붙인다. 빈 목록이면 지운다.
    pub fn set_overrides(&self, repo: &Path, pairs: Vec<String>) {
        let mut map = self
            .overrides
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pairs.is_empty() {
            map.remove(repo);
        } else {
            map.insert(repo.to_path_buf(), pairs);
        }
    }

    /// 테스트 전용: 로컬 경로 원격(`file`)을 허용한다.
    pub fn allow_local_remotes(&mut self) {
        self.allowed_protocols = "file:https:ssh:git";
    }

    /// `cwd`에서 git을 실행하고, 성공하면 출력을 돌려준다. stdin은 막는다.
    pub fn run(&self, cwd: Option<&Path>, args: &[&str]) -> Result<GitOutput> {
        let mut cmd = self.command(cwd, args);
        cmd.stdin(Stdio::null());
        let out = cmd.output().map_err(GitError::Spawn)?;
        finish(args, out)
    }

    /// 이 호출에만 환경 변수를 더해 실행한다(예: 임시 `GIT_INDEX_FILE`). 공통 설정 뒤에 붙어 이긴다.
    pub fn run_env(
        &self,
        cwd: Option<&Path>,
        args: &[&str],
        envs: &[(&str, &OsStr)],
    ) -> Result<GitOutput> {
        let mut cmd = self.command(cwd, args);
        cmd.stdin(Stdio::null());
        for (key, value) in envs {
            cmd.env(key, value);
        }
        let out = cmd.output().map_err(GitError::Spawn)?;
        finish(args, out)
    }

    /// 지켜보며 실행한다: 시간 제한(`TimedOut`), 첫 출력 뒤 오래 조용하면 멈춤(`Stalled`), 취소 깃발
    /// (`Cancelled`), 1초마다 진행 알림. 취소·멈춤은 먼저 끝내 달라고 알리고 2초 뒤 강제로 끝낸다(D-080).
    pub fn run_watched(
        &self,
        cwd: Option<&Path>,
        args: &[&str],
        envs: &[(&str, EnvChange<'_>)],
        stdin: Option<&[u8]>,
        watch: &Watch<'_>,
    ) -> Result<GitOutput> {
        let mut cmd = self.command(cwd, args);
        for (key, change) in envs {
            match change {
                EnvChange::Set(value) => cmd.env(key, value),
                EnvChange::Remove => cmd.env_remove(key),
            };
        }
        (self.process.prepare)(&mut cmd);
        cmd.stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(GitError::Spawn)?;
        let writer = match (stdin, child.stdin.take()) {
            (Some(input), Some(mut pipe)) => {
                let input = input.to_vec();
                Some(std::thread::spawn(move || pipe.write_all(&input)))
            }
            _ => None,
        };
        let out_reader = read_all(child.stdout.take());
        // stderr는 조각마다 "마지막 출력 시각"을 고친다. 내용은 해석하지 않는다(D-003).
        let last_output: Arc<std::sync::Mutex<Option<Instant>>> = Arc::default();
        let err_reader = {
            let last = last_output.clone();
            let mut pipe = child.stderr.take();
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                while let Some(p) = pipe.as_mut() {
                    match p.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&chunk[..n]);
                            *last
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                Some(Instant::now());
                        }
                    }
                }
                buf
            })
        };
        let start = Instant::now();
        let mut next_report = start + Duration::from_secs(1);
        let refused = |kind: crate::FailureKind| GitError::Refused {
            kind,
            detail: format!("git {}", args.join(" ")),
        };
        let status = loop {
            if let Some(status) = child.try_wait().map_err(GitError::Spawn)? {
                break status;
            }
            let elapsed = start.elapsed();
            let last = *last_output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let idle = last.map_or(elapsed, |t| t.elapsed());
            if watch
                .cancel
                .is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst))
            {
                self.stop(&mut child);
                return Err(refused(crate::FailureKind::Cancelled));
            }
            if watch.timeout.is_some_and(|t| elapsed >= t) {
                (self.process.kill)(&mut child);
                let _ = child.wait();
                return Err(GitError::TimedOut {
                    args: args.iter().map(|a| a.to_string()).collect(),
                });
            }
            // 연결 전(인증 창을 기다리는 중)에는 멈춤으로 보지 않는다. 첫 출력이 온 뒤부터 잰다.
            if let (Some(stall), Some(_)) = (watch.stall, last)
                && idle >= stall
            {
                self.stop(&mut child);
                return Err(refused(crate::FailureKind::Stalled));
            }
            if let Some(observe) = watch.observer
                && Instant::now() >= next_report
            {
                observe(elapsed, idle);
                next_report += Duration::from_secs(1);
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        if let Some(w) = writer {
            let _ = w.join();
        }
        let stdout = out_reader.join().unwrap_or_default();
        let stderr = err_reader.join().unwrap_or_default();
        finish(
            args,
            Output {
                status,
                stdout,
                stderr,
            },
        )
    }

    /// 끝내 달라고 알리고, 2초 안에 끝나지 않으면 강제로 끝낸다.
    fn stop(&self, child: &mut Child) {
        (self.process.terminate)(child);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(Some(_)) = child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        (self.process.kill)(child);
        let _ = child.wait();
    }

    /// `git merge-file -p --quiet <우리> <기준> <그쪽>`이 겹침 없이 합쳐지는지. 파일은 바꾸지 않는다.
    /// 종료 코드 0만 깨끗하다(겹친 곳 수, 이진 파일·오류는 255).
    pub fn merge_file_quiet(&self, cwd: Option<&Path>, files: &[&OsStr]) -> Result<bool> {
        let mut cmd = self.command(cwd, &["merge-file", "-p", "--quiet"]);
        cmd.args(files).stdin(Stdio::null());
        let out = cmd.output().map_err(GitError::Spawn)?;
        Ok(out.status.success())
    }

    /// `git merge-file -p <옵션> <우리> <기준> <그쪽>`의 (종료 코드, 출력). 종료 코드는 겹친 곳 수,
    /// 이진 파일·오류면 255다. 실패로 보지 않고 그대로 돌려준다.
    pub fn merge_file_print(
        &self,
        cwd: Option<&Path>,
        options: &[&str],
        files: &[&OsStr],
    ) -> Result<(i32, Vec<u8>)> {
        let mut args = vec!["merge-file", "-p"];
        args.extend_from_slice(options);
        let mut cmd = self.command(cwd, &args);
        cmd.args(files).stdin(Stdio::null());
        let out = cmd.output().map_err(GitError::Spawn)?;
        Ok((out.status.code().unwrap_or(255), out.stdout))
    }

    /// 출력이 `cap` 바이트를 넘으면 거기까지만 읽고 git을 끝낸다(큰 파일 비교, 001 10b).
    /// 종료 코드 0·1을 성공으로 본다(`diff --no-index`는 차이가 있으면 1).
    pub fn run_capped(&self, cwd: Option<&Path>, args: &[&str], cap: usize) -> Result<Capped> {
        let mut cmd = self.command(cwd, args);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(GitError::Spawn)?;
        let mut stderr = child.stderr.take();
        let err_reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(e) = stderr.as_mut() {
                let _ = e.read_to_end(&mut buf);
            }
            buf
        });
        let mut stdout = Vec::new();
        let mut truncated = false;
        if let Some(mut out) = child.stdout.take() {
            let mut chunk = [0u8; 64 * 1024];
            loop {
                let n = out.read(&mut chunk).map_err(GitError::Spawn)?;
                if n == 0 {
                    break;
                }
                stdout.extend_from_slice(&chunk[..n]);
                if stdout.len() >= cap {
                    stdout.truncate(cap);
                    truncated = true;
                    break;
                }
            }
        }
        if truncated {
            let _ = child.kill();
        }
        let status = child.wait().map_err(GitError::Spawn)?;
        let stderr = err_reader.join().unwrap_or_default();
        if !truncated && !matches!(status.code(), Some(0 | 1)) {
            return Err(GitError::Failed {
                args: args.iter().map(|a| a.to_string()).collect(),
                code: status.code(),
                stdout: String::from_utf8_lossy(&stdout).into_owned(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }
        Ok(Capped { stdout, truncated })
    }

    /// `input`을 stdin으로 넘겨 git을 실행한다(예: `--pathspec-from-file=-`).
    pub fn run_with_stdin(
        &self,
        cwd: Option<&Path>,
        args: &[&str],
        input: &[u8],
    ) -> Result<GitOutput> {
        self.run_with_stdin_env(cwd, args, input, &[])
    }

    /// `run_with_stdin`에 이 호출에만 쓸 환경 변수를 더한다(예: 임시 `GIT_INDEX_FILE`).
    pub fn run_with_stdin_env(
        &self,
        cwd: Option<&Path>,
        args: &[&str],
        input: &[u8],
        envs: &[(&str, &OsStr)],
    ) -> Result<GitOutput> {
        let mut cmd = self.command(cwd, args);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(GitError::Spawn)?;
        // 입력을 다른 스레드에서 써야 큰 입력에서 출력 파이프가 차서 서로 기다리는 일이 없다.
        let Some(mut stdin) = child.stdin.take() else {
            return Err(GitError::Spawn(std::io::Error::other(
                "stdin을 열지 못했어요",
            )));
        };
        let input = input.to_vec();
        let writer = std::thread::spawn(move || stdin.write_all(&input));
        let out = child.wait_with_output().map_err(GitError::Spawn)?;
        // git이 입력을 다 읽기 전에 끝나면(인자 오류 등) 쓰기가 실패할 수 있다. 결과는 종료 코드로 판단한다.
        let _ = writer.join();
        finish(args, out)
    }

    fn command(&self, cwd: Option<&Path>, args: &[&str]) -> Command {
        let mut cmd = Command::new(&self.git);
        // OS별 설정을 먼저 붙이고, 아래 앱 고정 설정이 마지막에 이기게 한다.
        (self.configure)(&mut cmd);
        for var in REPO_LOCATION_VARS {
            cmd.env_remove(var);
        }
        // 영어 원문 오류(D-021), 프롬프트 없음, 페이저 없음, 읽기 작업이 index 잠금을 잡지 않음.
        // LANGUAGE는 gettext에서 LC_ALL보다 앞설 수 있어 지운다.
        // GIT_EDITOR=":"는 git이 편집기를 띄우지 않는 특별 값이라 셸이 없어도 된다(D-052).
        cmd.env("LC_ALL", "C")
            .env_remove("LANGUAGE")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_PAGER", "cat")
            .env("PAGER", "cat")
            .env("GIT_EDITOR", ":")
            .env("GIT_ALLOW_PROTOCOL", self.allowed_protocols)
            .env("GIT_OPTIONAL_LOCKS", "0")
            // 한글 파일명을 따옴표·8진수 이스케이프 없이 받는다.
            .args(["-c", "core.quotepath=off", "-c", "color.ui=never"]);
        if let Some(dir) = cwd
            && let Some(pairs) = self
                .overrides
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(dir)
        {
            for pair in pairs {
                cmd.arg("-c").arg(pair);
            }
        }
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd
    }
}

/// 파이프를 다른 스레드에서 끝까지 읽는다(파이프가 차서 git이 멈추지 않게).
fn read_all<R: Read + Send + 'static>(pipe: Option<R>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    })
}

fn finish(args: &[&str], out: Output) -> Result<GitOutput> {
    if out.status.success() {
        Ok(GitOutput {
            stdout: out.stdout,
            stderr: out.stderr,
        })
    } else {
        Err(GitError::Failed {
            args: args.iter().map(|a| a.to_string()).collect(),
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

/// 경로 목록을 `--pathspec-from-file=- --pathspec-file-nul`용 stdin 바이트로 만든다.
/// 명령줄로 넘기지 않으므로 Windows 명령줄 길이 제한(32K)에 걸리지 않는다.
pub(crate) fn nul_separated(paths: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for path in paths {
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
    }
    bytes
}
