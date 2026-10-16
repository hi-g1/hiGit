//! 처음 여는 저장소의 신뢰 확인(D-039, D-067).
//!
//! 저장소 로컬 설정에 프로그램을 실행시키는 키가 있거나 hooks 폴더에 실행 파일이 있거나, 올리는 곳·연결을
//! 바꾸는 키가 있으면 묻는다(D-212). 결정은 앱 데이터 `trust.json`에 저장소 경로와 그 설정 내용의 해시로
//! 남기고, 내용이 바뀌면 다시 묻는다. "끄고 열기"는 그 저장소의 모든 git 호출에 덮어쓰기 `-c`를 붙이고,
//! 올리는 곳을 바꾸는 설정이 있으면 원격 작업을 git 크레이트에서 막는다.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use git::{GitBackend, RepoPath, RepoPolicy};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Result;
use crate::view::TrustRedirect;

const FILE_NAME: &str = "trust.json";
const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrustDecision {
    /// 믿고 연다. 설정과 hook이 그대로 돈다.
    Trust,
    /// 실행되는 설정을 끄고 연다.
    Disable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    path: String,
    hash: String,
    decision: TrustDecision,
}

#[derive(Serialize, Deserialize)]
struct FileFormat {
    version: u32,
    entries: Vec<Entry>,
}

/// 저장소에서 찾은 "실행되는 설정".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Executing {
    /// 화면에 보여줄 이름(설정 키, `hooks/pre-commit`).
    pub keys: Vec<String>,
    /// 올리는 곳·연결을 바꾸는 설정(화면에 보일 이름과 호스트). 끄고 열면 원격 작업을 막는다(D-212).
    pub redirects: Vec<TrustRedirect>,
    /// 끄고 열 때 붙일 `-c 키=값`.
    pub overrides: Vec<String>,
    /// 끄고 열 수 있는지. 하위 이름에 `=`가 든 키(`[filter "a=b"]`)는 `-c 키=값`으로 덮어쓸 수 없다
    /// (git이 첫 `=`에서 키와 값을 나눈다). 그런 키가 있으면 "믿고 열기"만 남긴다.
    pub can_disable: bool,
    hash: String,
}

impl Executing {
    /// "끄고 열기"의 git 정책.
    pub(crate) fn disabled_policy(&self) -> RepoPolicy {
        RepoPolicy {
            overrides: self.overrides.clone(),
            network_off: !self.redirects.is_empty(),
        }
    }
}

/// 신뢰 확인 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TrustState {
    /// 실행되는 설정이 없다. 묻지 않고 연다.
    Clean,
    Decided(TrustDecision, Executing),
    /// 처음이거나 설정이 바뀌었다. 물어야 한다.
    Unknown(Executing),
}

pub struct TrustStore {
    file: Option<PathBuf>,
    /// 끄고 열 때 hook 대신 쓰는 빈 폴더.
    empty_hooks: PathBuf,
    entries: Mutex<Vec<Entry>>,
}

impl TrustStore {
    /// 메모리에만 두는 기록(테스트, 앱 데이터 폴더를 못 쓸 때).
    /// 빈 hooks 폴더 이름은 무작위로 정한다. 정해진 이름이면 같은 PC의 다른 사용자가 임시 폴더에
    /// 먼저 만들어 hook을 넣어 둘 수 있다.
    pub fn in_memory() -> Self {
        let mut bytes = [0u8; 16];
        if getrandom::fill(&mut bytes).is_err() {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            bytes = (nanos ^ u128::from(std::process::id())).to_le_bytes();
        }
        Self {
            file: None,
            empty_hooks: std::env::temp_dir().join(format!("higit-empty-hooks-{}", hex(&bytes))),
            entries: Mutex::new(Vec::new()),
        }
    }

    /// `dir/trust.json`을 읽는다. 깨졌으면 빈 기록으로 시작한다(다시 묻게 된다).
    pub fn open(dir: &Path) -> Self {
        let file = dir.join(FILE_NAME);
        let entries = std::fs::read_to_string(&file)
            .ok()
            .and_then(|t| serde_json::from_str::<FileFormat>(&t).ok())
            .filter(|f| f.version == FORMAT_VERSION)
            .map(|f| f.entries)
            .unwrap_or_default();
        Self {
            file: Some(file),
            empty_hooks: dir.join("empty-hooks"),
            entries: Mutex::new(entries),
        }
    }

    pub(crate) fn check(&self, git: &dyn GitBackend, repo: &RepoPath) -> Result<TrustState> {
        let Some(found) = self.executing(git, repo)? else {
            return Ok(TrustState::Clean);
        };
        let entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        let decided = entries
            .iter()
            .find(|e| {
                platform::same_path(Path::new(&e.path), repo.as_path()) && e.hash == found.hash
            })
            .map(|e| e.decision);
        Ok(match decided {
            Some(d) => TrustState::Decided(d, found),
            None => TrustState::Unknown(found),
        })
    }

    pub(crate) fn record(&self, repo: &Path, found: &Executing, decision: TrustDecision) {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries.retain(|e| !platform::same_path(Path::new(&e.path), repo));
        entries.push(Entry {
            path: repo.to_string_lossy().into_owned(),
            hash: found.hash.clone(),
            decision,
        });
        if let Some(file) = &self.file {
            let data = FileFormat {
                version: FORMAT_VERSION,
                entries: entries.clone(),
            };
            if let Ok(text) = serde_json::to_string_pretty(&data) {
                // 못 쓰면 다음 실행에서 다시 묻는다(안전한 쪽).
                let _ = platform::write_private_file(file, text.as_bytes());
            }
        }
    }

    /// 실행되는 설정을 찾는다. 없으면 `None`.
    fn executing(&self, git: &dyn GitBackend, repo: &RepoPath) -> Result<Option<Executing>> {
        let mut keys = Vec::new();
        let mut redirects = Vec::new();
        let mut overrides = Vec::new();
        let mut fingerprint = Vec::new();
        let mut can_disable = true;
        let home = platform::home_dir().map(|h| h.to_string_lossy().into_owned());
        for (key, value) in git.local_config(repo)? {
            if redirect_key(&key) {
                fingerprint.push(format!("{key}={value}"));
                redirects.push(redirect_item(&key, &value, home.as_deref()));
            } else if let Some(safe) = safe_value(&key, &value) {
                can_disable &= !key.contains('=');
                keys.push(key.clone());
                fingerprint.push(format!("{key}={value}"));
                overrides.push(format!("{key}={safe}"));
                if key.starts_with("filter.") {
                    let driver = &key[..key.rfind('.').unwrap_or(key.len())];
                    overrides.push(format!("{driver}.required=false"));
                }
            }
        }
        // hooks는 worktree끼리 함께 쓰는 git 폴더에 있다(연결된 worktree의 git 폴더가 아니다).
        let hooks = hook_files(&git.common_dir(repo)?.join("hooks"));
        for (name, content) in &hooks {
            keys.push(format!("hooks/{name}"));
            fingerprint.push(format!("hooks/{name}:{}", hex(&Sha256::digest(content))));
        }
        if keys.is_empty() && redirects.is_empty() {
            return Ok(None);
        }
        // hook은 설정이 아니라 파일이라, 빈 폴더를 hooksPath로 줘서 끈다.
        if !hooks.is_empty() || keys.iter().any(|k| k == "core.hookspath") {
            let _ = std::fs::create_dir_all(&self.empty_hooks);
            overrides.retain(|o| !o.starts_with("core.hookspath="));
            overrides.push(format!("core.hookspath={}", self.empty_hooks.display()));
        }
        fingerprint.sort();
        keys.sort();
        keys.dedup();
        redirects.sort_by(|a, b| (&a.key, &a.target).cmp(&(&b.key, &b.target)));
        redirects.dedup();
        Ok(Some(Executing {
            keys,
            redirects,
            overrides,
            can_disable,
            hash: hex(&Sha256::digest(fingerprint.join("\n").as_bytes())),
        }))
    }
}

/// 올리는 곳·연결을 바꾸는 설정 키인지(git이 준 소문자 형태, D-111, D-212). 받은 저장소가 코드를 다른 곳으로
/// 올리게 하거나(`pushurl`, `insteadOf`), 연결을 돌리거나 TLS 검증을 끄거나(`http.*`, `proxy`), 원격 쪽 명령을
/// 바꿀 수 있다(`receivepack`, `uploadpack`). 여러 값이라 `-c`로 지울 수 없어 끄고 열면 원격 작업을 막는다.
pub(crate) fn redirect_key(key: &str) -> bool {
    if key.starts_with("http.") {
        return true;
    }
    if let Some(rest) = key.strip_prefix("remote.") {
        return rest.contains('.')
            && [".pushurl", ".proxy", ".receivepack", ".uploadpack"]
                .iter()
                .any(|s| rest.ends_with(s));
    }
    key.starts_with("url.") && (key.ends_with(".insteadof") || key.ends_with(".pushinsteadof"))
}

/// 화면에 보일 이름과 대상. 주소는 호스트만, 경로는 가려서, `http.*`·명령 값은 보이지 않는다(D-040).
fn redirect_item(key: &str, value: &str, home: Option<&str>) -> TrustRedirect {
    // 하위 이름이 주소인 키(`url.<주소>.insteadof`, `http.<주소>.<이름>`)는 하위 이름 자리에 호스트를 둔다.
    let split = |section: &str| {
        let rest = key.strip_prefix(section)?;
        let dot = rest.rfind('.')?;
        Some((&rest[..dot], &rest[dot + 1..]))
    };
    let (shown, target) = if let Some((base, name)) = split("url.") {
        (format!("url.{name}"), host_or_path(base, home))
    } else if let Some((base, name)) = split("http.") {
        (format!("http.{name}"), host_or_path(base, home))
    } else if key.ends_with(".pushurl") || key.ends_with(".proxy") {
        (key.to_string(), host_or_path(value, home))
    } else {
        (key.to_string(), None)
    };
    TrustRedirect {
        key: clip(&shown, 120),
        target,
    }
}

/// 주소면 호스트(사용자 정보·경로 뺌), `사용자@호스트:경로`면 호스트, 아니면 가린 경로.
fn host_or_path(value: &str, home: Option<&str>) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let authority = if let Some((_, rest)) = value.split_once("://") {
        Some(rest.split(['/', '?', '#']).next().unwrap_or(""))
    } else {
        // `host:path`(scp 형식). `C:\…`처럼 한 글자 앞 콜론은 Windows 경로다.
        value
            .split_once(':')
            .filter(|(before, _)| before.len() > 1 && !before.contains(['/', '\\']))
            .map(|(before, _)| before)
    };
    let text = match authority {
        Some(a) => a.rsplit('@').next().unwrap_or("").to_string(),
        None => crate::redact::redact(value, home),
    };
    let shown = clip(&text, 100);
    (!shown.is_empty()).then_some(shown)
}

/// 화면용 글자: 제어·방향 문자를 빼고 `max`글자로 자른다.
fn clip(text: &str, max: usize) -> String {
    let mut out: String = text
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
        .take(max + 1)
        .collect();
    if out.chars().count() > max {
        out = out.chars().take(max).collect();
        out.push('…');
    }
    out
}

/// 프로그램을 실행시키는 설정이면 끌 때 쓸 안전한 값. 아니면 `None`. 키는 git이 준 소문자 형태.
fn safe_value(key: &str, value: &str) -> Option<&'static str> {
    let exact = match key {
        // git 내장 감시는 프로그램을 실행하지 않는다.
        "core.fsmonitor" => {
            let v = value.to_ascii_lowercase();
            return (!matches!(
                v.as_str(),
                "" | "false" | "true" | "0" | "1" | "no" | "yes" | "off" | "on"
            ))
            .then_some("false");
        }
        "core.hookspath" => Some(""),
        // 질문을 끈 값으로 덮어쓴다. 앱은 이 값을 사용자가 정한 SSH 명령으로 보고 그대로 쓴다(D-023).
        "core.sshcommand" => Some("ssh -o BatchMode=yes"),
        "core.editor" | "sequence.editor" => Some(":"),
        "core.pager" => Some("cat"),
        "core.askpass"
        | "credential.helper"
        | "diff.external"
        | "uploadpack.packobjectshook"
        | "core.gitproxy"
        | "core.alternaterefscommand"
        | "gpg.ssh.defaultkeycommand" => Some(""),
        "gpg.program" => Some("gpg"),
        "include.path" => Some(""),
        _ => None,
    };
    if exact.is_some() {
        return exact;
    }
    let ends = |prefix: &str, suffixes: &[&str]| {
        key.starts_with(prefix) && suffixes.iter().any(|s| key.ends_with(s))
    };
    if ends("filter.", &[".clean", ".smudge"]) {
        Some("cat")
    } else if ends("filter.", &[".process"])
        || ends("diff.", &[".command", ".textconv"])
        || ends("merge.", &[".driver"])
        || ends("credential.", &[".helper"])
        || ends("gpg.", &[".program"])
        || ends("includeif.", &[".path"])
        || ends("difftool.", &[".cmd"])
        || ends("mergetool.", &[".cmd"])
    {
        Some("")
    } else {
        None
    }
}

/// 지문에 쓰는 hook 내용 상한. hook은 작은 스크립트라 이보다 길면 앞부분만 본다.
const HOOK_READ_LIMIT: u64 = 1024 * 1024;

/// hooks 폴더에서 `.sample`이 아닌 파일과 링크(이름, 지문 재료). git은 링크된 hook도 실행한다.
/// 링크는 따라가 읽지 않고 대상 경로를 지문으로 쓴다(`/dev/zero` 같은 곳을 읽지 않게).
fn hook_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    use std::io::Read;
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(String, Vec<u8>)> = read
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".sample") {
                return None;
            }
            let kind = e.file_type().ok()?;
            let content = if kind.is_symlink() {
                let target = std::fs::read_link(e.path()).unwrap_or_default();
                format!("link:{}", target.display()).into_bytes()
            } else if kind.is_file() {
                let mut buf = Vec::new();
                if let Ok(file) = platform::open_regular_no_follow(&e.path()) {
                    let _ = file.take(HOOK_READ_LIMIT).read_to_end(&mut buf);
                }
                buf
            } else {
                return None;
            };
            Some((name, content))
        })
        .collect();
    found.sort();
    found
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executing_keys() {
        assert_eq!(safe_value("core.fsmonitor", "touch x"), Some("false"));
        assert_eq!(safe_value("core.fsmonitor", "true"), None);
        assert_eq!(safe_value("filter.lfs.clean", "git-lfs clean"), Some("cat"));
        assert_eq!(safe_value("filter.lfs.process", "x"), Some(""));
        assert_eq!(safe_value("diff.pdf.textconv", "x"), Some(""));
        assert_eq!(safe_value("credential.https://a.helper", "x"), Some(""));
        assert_eq!(safe_value("includeif.gitdir:/x.path", "x"), Some(""));
        assert_eq!(safe_value("user.name", "x"), None);
        assert_eq!(safe_value("core.autocrlf", "true"), None);
        assert_eq!(safe_value("remote.origin.url", "x"), None);
        assert_eq!(
            safe_value("core.sshcommand", "ssh -o ProxyCommand=x"),
            Some("ssh -o BatchMode=yes")
        );
        assert_eq!(safe_value("core.gitproxy", "x"), Some(""));
        assert_eq!(safe_value("core.alternaterefscommand", "x"), Some(""));
        assert_eq!(safe_value("gpg.ssh.defaultkeycommand", "x"), Some(""));
        assert_eq!(safe_value("mergetool.x.cmd", "x"), Some(""));
    }

    #[test]
    fn redirect_keys() {
        for key in [
            "remote.origin.pushurl",
            "remote.a.b.pushurl",
            "url.https://evil.example/.insteadof",
            "url.git@evil.example:.pushinsteadof",
            "remote.origin.proxy",
            "remote.origin.receivepack",
            "remote.origin.uploadpack",
            "http.sslverify",
            "http.https://github.com/.extraheader",
        ] {
            assert!(redirect_key(key), "{key}");
        }
        for key in [
            "remote.origin.url",
            "remote.origin.fetch",
            "remote.pushdefault",
            "url.https://a/.other",
            "core.gitproxy",
            "user.name",
        ] {
            assert!(!redirect_key(key), "{key}");
        }
    }

    #[test]
    fn redirect_items_show_hosts_only() {
        let item = |k: &str, v: &str| {
            let r = redirect_item(k, v, Some("/home/me"));
            (r.key, r.target)
        };
        assert_eq!(
            item(
                "remote.origin.pushurl",
                "https://user:ghp_secret@evil.example:8443/x.git"
            ),
            (
                "remote.origin.pushurl".into(),
                Some("evil.example:8443".into())
            )
        );
        assert_eq!(
            item("remote.origin.pushurl", "git@evil.example:me/x.git"),
            ("remote.origin.pushurl".into(), Some("evil.example".into()))
        );
        assert_eq!(
            item("remote.origin.pushurl", "/home/me/elsewhere"),
            ("remote.origin.pushurl".into(), Some("~/elsewhere".into()))
        );
        assert_eq!(
            item(
                "url.https://evil.example/path/.insteadof",
                "https://github.com/"
            ),
            ("url.insteadof".into(), Some("evil.example".into()))
        );
        assert_eq!(
            item(
                "http.https://github.com/.extraheader",
                "Authorization: Bearer secret"
            ),
            ("http.extraheader".into(), Some("github.com".into()))
        );
        assert_eq!(
            item("http.sslverify", "false"),
            ("http.sslverify".into(), None)
        );
        assert_eq!(
            item("remote.origin.uploadpack", "touch /tmp/x"),
            ("remote.origin.uploadpack".into(), None)
        );
        assert_eq!(
            item("remote.origin.proxy", "http://\u{202e}evil.example"),
            ("remote.origin.proxy".into(), Some("evil.example".into()))
        );
    }

    #[test]
    fn in_memory_hooks_folder_is_not_guessable() {
        let a = TrustStore::in_memory();
        let b = TrustStore::in_memory();
        assert_ne!(a.empty_hooks, b.empty_hooks);
        assert!(!a.empty_hooks.exists());
    }

    #[test]
    fn linked_hooks_are_found_without_reading_the_target() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let evil = outside.path().join("evil.sh");
        std::fs::write(&evil, "#!/bin/sh\n").unwrap();
        std::fs::write(dir.path().join("pre-commit.sample"), "x").unwrap();
        if let Err(e) = platform::symlink_file(&evil, &dir.path().join("post-checkout")) {
            eprintln!("링크를 만들 수 없어 건너뜀: {e}");
            return;
        }
        let found = hook_files(dir.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "post-checkout");
        assert!(String::from_utf8_lossy(&found[0].1).starts_with("link:"));
    }
}
