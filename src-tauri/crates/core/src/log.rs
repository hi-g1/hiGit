//! 로그 파일(D-078). `<앱 데이터>/logs/higit.log`에 1MB씩 세 개까지 돌려 쓴다.
//!
//! 줄에는 시각·종류·작업·액션 id·오류 코드·git 하위 명령 이름·종료 코드만 쓴다. git 오류 원문은
//! 파일·브랜치 이름과 hook 출력이 섞여 있어 쓰지 않는다. 모든 칸은 글자·숫자·`-`·`_`·`.`만 남겨
//! 줄바꿈으로 줄을 위조하지 못하게 한다.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use crate::CoreError;
use crate::view::ErrorView;

const FILE: &str = "higit.log";
const MAX_BYTES: u64 = 1024 * 1024;
/// 지금 파일 말고 남기는 지난 파일 수(`higit.1.log`, `higit.2.log`).
const KEEP_OLD: u32 = 2;
const KEEP_DAYS: u64 = 14;

pub struct Logger {
    dir: Option<PathBuf>,
    lock: Mutex<()>,
}

impl Logger {
    /// 로그를 남기지 않는다(테스트, 앱 데이터 폴더를 못 쓸 때).
    pub fn disabled() -> Self {
        Self {
            dir: None,
            lock: Mutex::new(()),
        }
    }

    /// `app_data/logs`를 쓴다. 폴더를 못 만들면 로그 없이 계속한다. 14일 지난 파일은 지운다.
    pub fn open(app_data: &Path) -> Self {
        let dir = app_data.join("logs");
        if std::fs::create_dir_all(&dir).is_err() {
            return Self::disabled();
        }
        prune(&dir, SystemTime::now());
        Self {
            dir: Some(dir),
            lock: Mutex::new(()),
        }
    }

    /// 로그 폴더. 로그를 남기지 않으면 `None`.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// 작업 하나의 결과를 남긴다. `event`는 `execute`·`undo`·`redo`.
    pub(crate) fn action<T>(&self, event: &str, action: &str, result: &Result<T, CoreError>) {
        let line = match result {
            Ok(_) => format!("info {} {}", clean(event), clean(action)),
            Err(e) => {
                let mut line = format!(
                    "warn {} {} code={:?}",
                    clean(event),
                    clean(action),
                    ErrorView::from(e).code
                );
                if let CoreError::Git(git::GitError::Failed { args, code, .. }) = e {
                    line.push_str(&format!(" git={}", clean(&subcommand(args))));
                    if let Some(code) = code {
                        line.push_str(&format!(" exit={code}"));
                    }
                }
                line
            }
        };
        self.write(&line);
    }

    fn write(&self, line: &str) {
        let Some(dir) = &self.dir else {
            return;
        };
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let path = dir.join(FILE);
        if std::fs::metadata(&path).is_ok_and(|m| m.len() >= MAX_BYTES) {
            rotate(dir);
        }
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let stamp = crate::undo::backup::utc_stamp(secs);
        // 로그를 못 써도 앱은 계속한다.
        if let Ok(mut file) = platform::open_private_append(&path) {
            let _ = writeln!(file, "{stamp} {line}");
        }
    }
}

fn old_name(n: u32) -> String {
    format!("higit.{n}.log")
}

/// `higit.log` → `higit.1.log` → `higit.2.log`, 가장 오래된 것은 지운다.
fn rotate(dir: &Path) {
    let _ = std::fs::remove_file(dir.join(old_name(KEEP_OLD)));
    for n in (1..KEEP_OLD).rev() {
        let _ = std::fs::rename(dir.join(old_name(n)), dir.join(old_name(n + 1)));
    }
    let _ = std::fs::rename(dir.join(FILE), dir.join(old_name(1)));
}

/// 수정한 지 14일 지난 로그 파일을 지운다.
fn prune(dir: &Path, now: SystemTime) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let limit = Duration::from_secs(KEEP_DAYS * 24 * 60 * 60);
    for entry in read.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !(name.starts_with("higit") && name.ends_with(".log")) {
            continue;
        }
        let old = entry
            .metadata()
            .ok()
            .filter(|m| m.is_file())
            .and_then(|m| m.modified().ok())
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age > limit);
        if old {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// git 인자에서 하위 명령 이름(첫 옵션이 아닌 인자). `-c 키=값`은 건너뛴다.
fn subcommand(args: &[String]) -> String {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-c" || arg == "-C" {
            iter.next();
        } else if !arg.starts_with('-') {
            return arg.clone();
        }
    }
    String::new()
}

/// 글자·숫자·`-`·`_`·`.`만 남긴다. 줄바꿈·공백·제어 문자·`=`는 지운다.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .take(64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_all(dir: &Path) -> String {
        std::fs::read_to_string(dir.join("logs").join(FILE)).unwrap()
    }

    #[test]
    fn lines_hold_codes_but_no_names_or_secrets() {
        let data = tempfile::tempdir().unwrap();
        let logger = Logger::open(data.path());
        let failed: Result<(), CoreError> = Err(CoreError::Git(git::GitError::Failed {
            args: vec![
                "-c".into(),
                "credential.helper=".into(),
                "push".into(),
                "https://user:ghp_secret@github.com/me/내 프로젝트.git".into(),
            ],
            code: Some(128),
            stdout: String::new(),
            stderr: "fatal: could not read from /home/me/비밀.txt\nme@example.com".into(),
        }));
        logger.action("execute", "push", &failed);
        logger.action("undo", "commit\nfake line", &Ok::<(), CoreError>(()));
        let text = read_all(data.path());
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "{text}");
        assert!(
            lines[0].ends_with("warn execute push code=GitFailed git=push exit=128"),
            "{text}"
        );
        assert!(lines[1].ends_with("info undo commitfakeline"), "{text}");
        for secret in [
            "ghp_secret",
            "user:",
            "example.com",
            "/home/me",
            "비밀",
            "내 프로젝트",
        ] {
            assert!(!text.contains(secret), "{secret} in {text}");
        }
    }

    #[test]
    fn files_rotate_and_old_ones_are_pruned() {
        let data = tempfile::tempdir().unwrap();
        let logger = Logger::open(data.path());
        let dir = data.path().join("logs");
        std::fs::write(dir.join(FILE), vec![b'x'; MAX_BYTES as usize]).unwrap();
        logger.action("execute", "commit", &Ok::<(), CoreError>(()));
        assert!(dir.join(old_name(1)).exists());
        assert!(std::fs::metadata(dir.join(FILE)).unwrap().len() < 100);
        // 다시 넘치면 한 칸씩 밀리고 가장 오래된 것은 남지 않는다.
        std::fs::write(dir.join(FILE), vec![b'y'; MAX_BYTES as usize]).unwrap();
        logger.action("execute", "commit", &Ok::<(), CoreError>(()));
        std::fs::write(dir.join(FILE), vec![b'z'; MAX_BYTES as usize]).unwrap();
        logger.action("execute", "commit", &Ok::<(), CoreError>(()));
        assert!(dir.join(old_name(2)).exists());
        assert!(!dir.join(old_name(3)).exists());
        assert!(
            std::fs::read(dir.join(old_name(2)))
                .unwrap()
                .starts_with(b"y")
        );

        // 14일 지난 파일은 다음에 열 때 지운다.
        let stale = dir.join(old_name(2));
        let past = SystemTime::now() - Duration::from_secs(15 * 24 * 60 * 60);
        std::fs::File::options()
            .write(true)
            .open(&stale)
            .unwrap()
            .set_modified(past)
            .unwrap();
        std::fs::write(dir.join("other.txt"), "x").unwrap();
        let _ = Logger::open(data.path());
        assert!(!stale.exists());
        assert!(dir.join(FILE).exists());
        assert!(dir.join("other.txt").exists());
    }

    #[test]
    fn a_disabled_logger_writes_nothing() {
        let logger = Logger::disabled();
        assert!(logger.dir().is_none());
        logger.action("execute", "commit", &Ok::<(), CoreError>(()));
    }
}
