//! 작업 목록 `<git 폴더>/higit/journal.jsonl`(D-042, D-063, D-065).
//!
//! 줄을 고치지 않고 덧붙이기만 한다. 줄마다 앱 데이터의 키로 서명하고, 서명이 맞지 않거나
//! 형식이 이상한 줄은 읽을 때 버린다(받은 zip 저장소에 남이 만든 줄이 있을 수 있다).
//! 같은 까닭으로 `higit` 폴더나 journal 파일이 링크면 읽지도 쓰지도 않는다. 링크를 따라 쓰면
//! 저장소 밖 파일(`~/.bashrc` 등)에 줄이 덧붙는다.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{JournalKey, UndoKind, UndoState};
use crate::view::ActionInput;
use crate::{CoreError, Result};

const VERSION: u32 = 2;
/// 옛 형식(줄 하나만 서명). 저장소를 열 때 한 번 v2 사슬로 옮긴다(D-209).
const VERSION_1: u32 = 1;
/// 서명 글자의 도메인.
const DOMAIN: &str = "higit-journal";
/// 이보다 큰 journal은 뒤쪽만 읽는다.
const MAX_READ_BYTES: u64 = 4 * 1024 * 1024;
/// 사슬을 다시 시작하는 경계 줄의 이름. 앞 줄과 이어지지 않았을 때 덧붙이기가 쓴다.
const RESTART: &str = "journal-restart";

// 줄 하나를 읽고 쓰는 데만 쓰는 값이라 변형 크기 차이는 상관없다.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum Body {
    /// 되돌릴 수 있는 작업 하나.
    #[serde(rename_all = "camelCase")]
    Action {
        id: String,
        at: i64,
        action: String,
        input: ActionInput,
        before: UndoState,
        after: UndoState,
        kind: UndoKind,
    },
    /// 이 앞의 줄은 되돌리지 않는다(되돌릴 수 없는 작업이 있었다, D-065).
    Barrier { at: i64, action: String },
    /// `target` 작업을 되돌렸다.
    Undo { target: String, at: i64 },
    /// 되돌린 `target`을 다시 했다.
    Redo { target: String, at: i64 },
}

/// v1 줄의 서명 글자(옛 형식 확인과 옮기기에만 쓴다).
#[derive(Serialize)]
struct UnsignedV1<'a> {
    v: u32,
    #[serde(flatten)]
    body: &'a Body,
}

#[derive(Deserialize)]
struct Record {
    v: u32,
    #[serde(default)]
    seq: Option<u64>,
    #[serde(flatten)]
    body: Body,
    sig: String,
}

#[derive(Serialize)]
struct Signed<'a> {
    v: u32,
    seq: u64,
    #[serde(flatten)]
    body: &'a Body,
    sig: String,
}

/// 줄을 차례로 본 결과.
struct Walk {
    /// 형식이 맞는 줄과 믿는지(v2 사슬에 이어지고 서명이 맞음).
    lines: Vec<(Body, bool)>,
    /// 마지막으로 사슬에 이어진 줄(순번, 서명). 사슬이 끊겼으면 `None`.
    tail: Option<(u64, String)>,
    /// v2 줄이 하나라도 있었는지.
    any_v2: bool,
    /// 사슬이 끊긴 채 끝났는지.
    broken: bool,
}

pub(crate) struct Journal {
    git_dir: PathBuf,
    path: PathBuf,
    key: JournalKey,
}

impl Journal {
    pub(crate) fn at(git_dir: &Path, key: JournalKey) -> Self {
        Self {
            git_dir: git_dir.to_path_buf(),
            path: git_dir.join("higit").join("journal.jsonl"),
            key,
        }
    }

    /// `higit` 폴더와 journal 파일이 링크가 아니고 git 폴더 안에 있는지. 없는 것은 괜찮다.
    fn check_place(&self) -> Result<()> {
        let refuse = || CoreError::Journal("journal 위치가 링크입니다".into());
        let dir = self.git_dir.join("higit");
        for (path, want_dir) in [(&dir, true), (&self.path, false)] {
            match std::fs::symlink_metadata(path) {
                Ok(m) if m.file_type().is_symlink() || m.is_dir() != want_dir => {
                    return Err(refuse());
                }
                Ok(_) | Err(_) => {}
            }
        }
        // Windows 연결 지점(junction)처럼 링크로 보이지 않는 것도 실제 위치로 확인한다.
        if let (Ok(real_dir), Ok(real_git)) = (dir.canonicalize(), self.git_dir.canonicalize())
            && !real_dir.starts_with(&real_git)
        {
            return Err(refuse());
        }
        Ok(())
    }

    /// v2 서명 글자: 도메인·형식·git 폴더 실제 경로·순번·앞 줄 서명·본문(D-209).
    fn signed_text(&self, seq: u64, prev: &str, body: &Body) -> Result<String> {
        let dir = crate::marker::real_dir(&self.git_dir)
            .ok_or_else(|| CoreError::Journal("git 폴더 경로를 알 수 없습니다".into()))?;
        let body = serde_json::to_string(body).map_err(|e| CoreError::Journal(e.to_string()))?;
        Ok(format!("{DOMAIN}\n{VERSION}\n{dir}\n{seq}\n{prev}\n{body}"))
    }

    fn signed_line(&self, seq: u64, prev: &str, body: &Body) -> Result<(String, String)> {
        let sig = self.key.sign(&self.signed_text(seq, prev, body)?);
        let line = serde_json::to_string(&Signed {
            v: VERSION,
            seq,
            body,
            sig: sig.clone(),
        })
        .map_err(|e| CoreError::Journal(e.to_string()))?;
        Ok((line, sig))
    }

    /// 줄을 하나 덧붙인다. 앞 줄과 사슬로 잇고, 사슬이 끊겨 있으면 다시 시작하는 경계 줄을 먼저 쓴다.
    /// 실행기가 저장소 쓰기 잠금 안에서만 부르므로 두 줄이 같은 순번을 받지 않는다.
    pub(crate) fn append(&self, body: &Body) -> Result<()> {
        self.check_place()?;
        // 서명에 git 폴더의 실제 경로가 들어가므로 폴더를 먼저 만든다.
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| CoreError::Journal(e.to_string()))?;
        }
        // 만든 사이에 링크로 바뀌지 않았는지 한 번 더 본다.
        self.check_place()?;
        let walk = self.walk();
        let mut text = String::new();
        let (seq, prev) = match (&walk.tail, walk.broken) {
            (Some((seq, sig)), false) => (seq + 1, sig.clone()),
            (None, false) if !walk.any_v2 => (1, String::new()),
            // 끊긴 사슬 뒤에는 순번 1의 경계 줄로 새로 시작한다(그 앞 줄은 되돌리지 않는다).
            _ => {
                let restart = Body::Barrier {
                    at: now(),
                    action: RESTART.into(),
                };
                let (line, sig) = self.signed_line(1, "", &restart)?;
                text.push_str(&line);
                text.push('\n');
                (2, sig)
            }
        };
        let (line, _) = self.signed_line(seq, &prev, body)?;
        text.push_str(&line);
        text.push('\n');
        self.check_place()?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CoreError::Journal(e.to_string()))?;
        file.write_all(text.as_bytes())
            .and_then(|()| file.sync_data())
            .map_err(|e| CoreError::Journal(e.to_string()))
    }

    /// 믿을 수 있는 줄만 순서대로(사슬에 이어진 v2 줄). 파일이 없으면 빈 목록.
    pub(crate) fn read(&self) -> Vec<Body> {
        self.walk()
            .lines
            .into_iter()
            .filter_map(|(body, trusted)| trusted.then_some(body))
            .collect()
    }

    /// 작업 내역용: 형식이 맞는 줄을 믿는지와 함께. 믿지 않는 줄(다른 컴퓨터, 받은 zip, 끊긴 사슬 뒤,
    /// 옛 형식)도 보여 주되 되살리기는 하지 않는다(D-063, D-085, D-209).
    pub(crate) fn read_all(&self) -> Vec<(Body, bool)> {
        self.walk().lines
    }

    /// 파일을 처음부터(크면 뒤쪽 4MB) 보며 사슬을 확인한다. 잘린 앞쪽이 있으면 첫 v2 줄은 기준으로만 쓰고
    /// 믿지 않는다(그 앞 줄 서명을 모른다).
    fn walk(&self) -> Walk {
        let mut walk = Walk {
            lines: Vec::new(),
            tail: None,
            any_v2: false,
            broken: false,
        };
        if self.check_place().is_err() {
            return walk;
        }
        let Ok((bytes, truncated)) = read_tail(&self.path) else {
            return walk;
        };
        let text = String::from_utf8_lossy(&bytes);
        let mut lines = text.lines();
        if truncated {
            // 앞이 잘린 첫 줄은 버린다.
            lines.next();
        }
        // 잘리지 않았으면 순번 1·빈 앞 서명에서 시작한다.
        let mut state: Option<(u64, String)> = (!truncated).then(|| (0, String::new()));
        for line in lines {
            let Ok(record) = serde_json::from_str::<Record>(line) else {
                continue;
            };
            let valid = valid_body(&record.body);
            match (record.v, record.seq) {
                (VERSION_1, _) => {
                    // 옛 줄은 더 믿지 않는다. 옮기기 전에도 작업 내역에는 보인다.
                    if valid {
                        walk.lines.push((record.body, false));
                    }
                }
                (VERSION, Some(seq)) => {
                    walk.any_v2 = true;
                    let restart = seq == 1
                        && matches!(&record.body, Body::Barrier { action, .. } if action == RESTART)
                        && self.verifies(1, "", &record.body, &record.sig);
                    let linked = match &state {
                        Some((last, prev)) if !walk.broken => {
                            seq == last + 1 && self.verifies(seq, prev, &record.body, &record.sig)
                        }
                        _ => false,
                    };
                    if restart {
                        // 그 앞에 믿은 줄은 되돌리지 않는다(경계처럼).
                        for l in &mut walk.lines {
                            l.1 = false;
                        }
                        walk.broken = false;
                        state = Some((1, record.sig));
                        walk.lines.push((record.body, true));
                    } else if linked {
                        state = Some((seq, record.sig));
                        if valid {
                            walk.lines.push((record.body, true));
                        }
                    } else if truncated && state.is_none() && !walk.broken {
                        // 잘린 파일의 첫 v2 줄: 다음 줄의 기준으로만 쓴다.
                        state = Some((seq, record.sig));
                        if valid {
                            walk.lines.push((record.body, false));
                        }
                    } else {
                        walk.broken = true;
                        if valid {
                            walk.lines.push((record.body, false));
                        }
                    }
                }
                _ => {}
            }
        }
        walk.tail = if walk.broken {
            None
        } else {
            state.filter(|(seq, _)| *seq > 0)
        };
        walk
    }

    fn verifies(&self, seq: u64, prev: &str, body: &Body, sig: &str) -> bool {
        self.signed_text(seq, prev, body)
            .is_ok_and(|text| self.key.verify(&text, sig))
    }

    /// 옛 v1 줄을 v2 사슬로 한 번 옮긴다(D-209). 파일에 v2 줄이 하나도 없을 때만 하고, 지금 키로 서명이
    /// 맞는 v1 줄만 옮기며 서명이 맞지 않는 줄은 그 자리에 그대로 둔다. 옮긴 줄 수를 돌려준다. 실패하면
    /// 아무것도 바꾸지 않는다(임시 파일에 쓰고 바꿔 끼운다).
    pub(crate) fn migrate_v1(&self) -> Result<usize> {
        self.check_place()?;
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return Ok(0);
        };
        if u64::try_from(text.len()).unwrap_or(u64::MAX) > MAX_READ_BYTES {
            return Ok(0);
        }
        let mut out = String::new();
        let mut moved = 0;
        let mut seq = 0u64;
        let mut prev = String::new();
        for line in text.lines() {
            let record = serde_json::from_str::<Record>(line).ok();
            match record {
                Some(r) if r.v == VERSION => return Ok(0),
                Some(r) if r.v == VERSION_1 && self.v1_verifies(&r.body, &r.sig) => {
                    seq += 1;
                    let (new, sig) = self.signed_line(seq, &prev, &r.body)?;
                    prev = sig;
                    out.push_str(&new);
                    moved += 1;
                }
                _ => out.push_str(line),
            }
            out.push('\n');
        }
        if moved == 0 {
            return Ok(0);
        }
        let tmp = self.path.with_extension("jsonl.tmp");
        let write = || -> std::io::Result<()> {
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(out.as_bytes())?;
            file.sync_data()?;
            drop(file);
            // 쓰는 사이에 링크로 바뀌었으면 바꿔 끼우지 않는다.
            if self.check_place().is_err() {
                return Err(std::io::Error::other("journal 위치가 링크입니다"));
            }
            std::fs::rename(&tmp, &self.path)
        };
        if let Err(e) = write() {
            let _ = std::fs::remove_file(&tmp);
            return Err(CoreError::Journal(e.to_string()));
        }
        Ok(moved)
    }

    fn v1_verifies(&self, body: &Body, sig: &str) -> bool {
        serde_json::to_string(&UnsignedV1 { v: VERSION_1, body })
            .is_ok_and(|text| self.key.verify(&text, sig))
            && valid_body(body)
    }
}

/// 옛 앱(v1)이 쓴 것과 같은 줄(시험에서 옮기기를 확인할 때).
#[cfg(test)]
pub(crate) fn v1_line_for_tests(key: &JournalKey, body: &Body) -> String {
    #[derive(Serialize)]
    struct SignedV1<'a> {
        v: u32,
        #[serde(flatten)]
        body: &'a Body,
        sig: String,
    }
    let text = serde_json::to_string(&UnsignedV1 { v: VERSION_1, body }).unwrap_or_default();
    serde_json::to_string(&SignedV1 {
        v: VERSION_1,
        body,
        sig: key.sign(&text),
    })
    .unwrap_or_default()
}

/// 형식 검사: 이름·해시·길이(받은 저장소의 줄일 수 있다).
fn valid_body(body: &Body) -> bool {
    match body {
        Body::Action {
            id, before, after, ..
        } => id.len() <= 64 && before.is_valid() && after.is_valid(),
        Body::Undo { target, .. } | Body::Redo { target, .. } => target.len() <= 64,
        Body::Barrier { .. } => true,
    }
}

/// 파일의 뒤쪽 4MB까지와, 앞이 잘렸는지.
fn read_tail(path: &Path) -> std::io::Result<(Vec<u8>, bool)> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    let truncated = len > MAX_READ_BYTES;
    if truncated {
        file.seek(SeekFrom::Start(len - MAX_READ_BYTES))?;
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok((buf, truncated))
}

/// 새 작업 id. 시각과 순번으로 만들어 한 저장소 안에서 겹치지 않는다.
pub(crate) fn new_id(at: i64) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    format!("{at:x}-{:x}", NEXT.fetch_add(1, Ordering::Relaxed))
}

pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::CommitInput;

    fn action(id: &str) -> Body {
        Body::Action {
            id: id.into(),
            at: 1,
            action: "commit".into(),
            input: ActionInput::Commit(CommitInput {
                message: "버전".into(),
                stage_all: false,
            }),
            before: UndoState {
                head: Some("a".repeat(40)),
                branch: Some("main".into()),
                ..Default::default()
            },
            after: UndoState {
                head: Some("b".repeat(40)),
                branch: Some("main".into()),
                ..Default::default()
            },
            kind: UndoKind::RestoreRef,
        }
    }

    #[test]
    fn append_and_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::at(dir.path(), JournalKey::ephemeral());
        journal.append(&action("1")).unwrap();
        journal
            .append(&Body::Barrier {
                at: 2,
                action: "stage".into(),
            })
            .unwrap();
        assert_eq!(journal.read().len(), 2);
        assert_eq!(journal.read()[0], action("1"));
    }

    #[test]
    fn foreign_broken_and_invalid_lines_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let mine = Journal::at(dir.path(), JournalKey::ephemeral());
        let other = Journal::at(dir.path(), JournalKey::ephemeral());
        other.append(&action("남의 줄")).unwrap();
        // 남의 줄 뒤에서는 사슬이 끊겨 있으므로 다시 시작하는 경계 줄을 먼저 쓴다.
        mine.append(&action("내 줄")).unwrap();
        // 이름이 옵션처럼 보이는 줄은 서명이 맞아도 버린다(사슬은 이어진다).
        let mut bad = action("나쁜 줄");
        if let Body::Action { before, .. } = &mut bad {
            before.branch = Some("-x".into());
        }
        mine.append(&bad).unwrap();
        mine.append(&action("다음 줄")).unwrap();
        // 쓰다 끊긴 마지막 줄.
        let path = dir.path().join("higit/journal.jsonl");
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(b"{\"v\":2,\"seq\":9,\"type\":\"act").unwrap();
        let lines = mine.read();
        assert!(matches!(&lines[0], Body::Barrier { action, .. } if action == RESTART));
        assert_eq!(&lines[1..], &[action("내 줄"), action("다음 줄")]);
    }

    /// 줄을 차례로 읽어 다시 쓴다(시험에서 줄을 지우거나 바꾸려고).
    fn rewrite(dir: &Path, f: impl FnOnce(&mut Vec<String>)) {
        let path = dir.join("higit/journal.jsonl");
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        f(&mut lines);
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();
    }

    fn trusted_ids(journal: &Journal) -> Vec<String> {
        journal
            .read()
            .into_iter()
            .filter_map(|b| match b {
                Body::Action { id, .. } => Some(id),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn lines_moved_to_another_repo_are_not_trusted() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let key = JournalKey::fixed(3);
        Journal::at(a.path(), key.clone())
            .append(&action("1"))
            .unwrap();
        std::fs::create_dir_all(b.path().join("higit")).unwrap();
        std::fs::copy(
            a.path().join("higit/journal.jsonl"),
            b.path().join("higit/journal.jsonl"),
        )
        .unwrap();
        let there = Journal::at(b.path(), key.clone());
        assert!(there.read().is_empty());
        // 작업 내역에는 보이되 믿지 않는다.
        assert_eq!(there.read_all(), vec![(action("1"), false)]);
        assert_eq!(trusted_ids(&Journal::at(a.path(), key)), ["1"]);
    }

    #[test]
    fn deleting_swapping_or_repeating_lines_breaks_the_chain() {
        let key = JournalKey::fixed(5);
        let make = || {
            let dir = tempfile::tempdir().unwrap();
            let j = Journal::at(dir.path(), key.clone());
            for id in ["1", "2", "3", "4"] {
                j.append(&action(id)).unwrap();
            }
            dir
        };
        // 줄 하나 지우기: 그 뒤는 믿지 않는다.
        let dir = make();
        rewrite(dir.path(), |l| {
            l.remove(1);
        });
        assert_eq!(trusted_ids(&Journal::at(dir.path(), key.clone())), ["1"]);
        // 두 줄 바꾸기
        let dir = make();
        rewrite(dir.path(), |l| l.swap(1, 2));
        assert_eq!(trusted_ids(&Journal::at(dir.path(), key.clone())), ["1"]);
        // 같은 줄 두 번
        let dir = make();
        rewrite(dir.path(), |l| {
            let dup = l[3].clone();
            l.push(dup);
        });
        let j = Journal::at(dir.path(), key.clone());
        assert_eq!(trusted_ids(&j), ["1", "2", "3", "4"]);
        assert_eq!(j.read_all().last().map(|l| l.1), Some(false));
        // 끊긴 뒤 새로 덧붙이면 경계 줄로 다시 시작하고, 그 앞은 되돌리지 않는다.
        j.append(&action("5")).unwrap();
        assert_eq!(trusted_ids(&j), ["5"]);
        assert!(matches!(&j.read()[0], Body::Barrier { action, .. } if action == RESTART));
    }

    #[test]
    fn a_replayed_first_line_cannot_restart_the_chain() {
        let dir = tempfile::tempdir().unwrap();
        let key = JournalKey::fixed(6);
        let j = Journal::at(dir.path(), key.clone());
        j.append(&action("1")).unwrap();
        j.append(&action("2")).unwrap();
        // 순번 1인 옛 작업 줄을 끝에 다시 붙여도 다시 시작하는 경계가 아니라 끊김이다.
        rewrite(dir.path(), |l| {
            let first = l[0].clone();
            l.push(first);
        });
        assert_eq!(trusted_ids(&j), ["1", "2"]);
        assert_eq!(j.read_all().len(), 3);
        assert!(!j.read_all()[2].1);
    }

    #[test]
    fn a_large_file_is_read_from_the_back() {
        let dir = tempfile::tempdir().unwrap();
        let key = JournalKey::fixed(8);
        let j = Journal::at(dir.path(), key.clone());
        // 4MB를 넘게 쌓는다(줄 하나 약 400바이트). 덧붙이기는 줄마다 사슬을 다시 보므로 파일을 바로 만든다.
        let lines = (MAX_READ_BYTES / 350) as usize + 50;
        let mut text = String::new();
        let mut prev = String::new();
        for i in 0..lines {
            let (line, sig) = j
                .signed_line(i as u64 + 1, &prev, &action(&i.to_string()))
                .unwrap();
            text.push_str(&line);
            text.push('\n');
            prev = sig;
        }
        std::fs::create_dir_all(dir.path().join("higit")).unwrap();
        std::fs::write(dir.path().join("higit/journal.jsonl"), text).unwrap();
        assert!(
            std::fs::metadata(dir.path().join("higit/journal.jsonl"))
                .unwrap()
                .len()
                > MAX_READ_BYTES
        );
        let ids = trusted_ids(&j);
        // 잘린 앞쪽의 첫 줄은 기준으로만 쓰고, 그 뒤는 끝까지 믿는다.
        assert!(!ids.is_empty() && ids.len() < lines);
        assert_eq!(ids.last().unwrap(), &(lines - 1).to_string());
        j.append(&action("끝")).unwrap();
        assert_eq!(trusted_ids(&j).last().unwrap(), "끝");
    }

    #[test]
    fn links_are_never_followed() {
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("bashrc");
        std::fs::write(&target, "원래 내용\n").unwrap();

        // journal 파일이 저장소 밖 파일로 가는 링크.
        let git = tempfile::tempdir().unwrap();
        std::fs::create_dir(git.path().join("higit")).unwrap();
        if let Err(e) = platform::symlink_file(&target, &git.path().join("higit/journal.jsonl")) {
            eprintln!("링크를 만들 수 없어 건너뜀: {e}");
            return;
        }
        let journal = Journal::at(git.path(), JournalKey::ephemeral());
        assert!(journal.append(&action("1")).is_err());
        assert!(journal.read().is_empty());

        // higit 폴더가 저장소 밖 폴더로 가는 링크.
        let git2 = tempfile::tempdir().unwrap();
        platform::symlink_dir(outside.path(), &git2.path().join("higit")).unwrap();
        let journal = Journal::at(git2.path(), JournalKey::ephemeral());
        assert!(journal.append(&action("1")).is_err());

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "원래 내용\n");
        assert!(!outside.path().join("journal.jsonl").exists());
    }

    /// 1b 앱이 쓴 줄(고정 키로 서명). 새 필드를 더해도 이 줄이 그대로 읽혀야 한다(D-072).
    /// 새 필드는 비어 있으면 쓰지 않아야(`skip_serializing_if`) 다시 직렬화한 글자가 같다.
    const LINE_1B: &str = r#"{"v":1,"type":"action","id":"1","at":1,"action":"commit","input":{"id":"commit","input":{"message":"버전","stageAll":false}},"before":{"head":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","branch":"main","parked":[],"indexTree":null},"after":{"head":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","branch":"main","parked":[],"indexTree":null},"kind":"restoreRef","sig":"7c610046e0130a8c83e3d3200968145a27da8d105e36586b902632e55cc9d052"}"#;

    #[test]
    fn lines_written_by_older_versions_are_moved_once() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("higit")).unwrap();
        // 1b 앱이 쓴 줄과, 다른 키로 서명한 옛 줄.
        let foreign = LINE_1B.replace("7c61", "0000");
        std::fs::write(
            dir.path().join("higit/journal.jsonl"),
            format!("{LINE_1B}\n{foreign}\n"),
        )
        .unwrap();
        let journal = Journal::at(dir.path(), JournalKey::fixed(7));
        // 옮기기 전에는 믿지 않는다(보이기만).
        assert!(journal.read().is_empty());
        // 형식이 맞는 옛 줄은 서명과 상관없이 작업 내역에 보인다.
        assert!(journal.read_all().iter().all(|(_, trusted)| !trusted));
        assert_eq!(journal.read_all().len(), 2);
        assert_eq!(journal.migrate_v1().unwrap(), 1);
        assert_eq!(journal.read(), vec![action("1")]);
        // 서명이 맞지 않는 옛 줄은 그 자리에 그대로 남는다.
        let text = std::fs::read_to_string(dir.path().join("higit/journal.jsonl")).unwrap();
        assert!(text.contains(&foreign));
        assert!(
            text.lines()
                .next()
                .unwrap()
                .starts_with("{\"v\":2,\"seq\":1,")
        );
        // v2 줄이 있으면 다시 옮기지 않는다.
        assert_eq!(journal.migrate_v1().unwrap(), 0);
        // 이어 쓴 줄도 사슬에 붙는다.
        journal.append(&action("2")).unwrap();
        assert_eq!(trusted_ids(&journal), ["1", "2"]);
        // 다른 키로는 옮기지 않는다.
        let again = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(again.path().join("higit")).unwrap();
        std::fs::write(
            again.path().join("higit/journal.jsonl"),
            format!("{LINE_1B}\n"),
        )
        .unwrap();
        assert_eq!(
            Journal::at(again.path(), JournalKey::fixed(9))
                .migrate_v1()
                .unwrap(),
            0
        );
    }

    #[test]
    fn moving_refuses_a_linked_folder() {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("journal.jsonl"), format!("{LINE_1B}\n")).unwrap();
        let git = tempfile::tempdir().unwrap();
        if platform::symlink_dir(outside.path(), &git.path().join("higit")).is_err() {
            eprintln!("링크를 만들 수 없어 건너뜀");
            return;
        }
        let journal = Journal::at(git.path(), JournalKey::fixed(7));
        assert!(journal.migrate_v1().is_err());
        assert_eq!(
            std::fs::read_to_string(outside.path().join("journal.jsonl")).unwrap(),
            format!("{LINE_1B}\n")
        );
    }

    #[test]
    fn a_folder_in_place_of_the_file_is_refused() {
        let git = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(git.path().join("higit/journal.jsonl")).unwrap();
        let journal = Journal::at(git.path(), JournalKey::ephemeral());
        assert!(journal.append(&action("1")).is_err());
    }
}
