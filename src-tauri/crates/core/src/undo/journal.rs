//! 작업 목록 `<git 폴더>/higit/journal.jsonl`(D-042, D-063, D-065).
//!
//! 줄을 고치지 않고 덧붙이기만 한다. 줄마다 앱 데이터의 키로 서명하고, 서명이 맞지 않거나
//! 형식이 이상한 줄은 읽을 때 버린다(받은 zip 저장소에 남이 만든 줄이 있을 수 있다).

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{JournalKey, UndoKind, UndoState};
use crate::view::ActionInput;
use crate::{CoreError, Result};

const VERSION: u32 = 1;
/// 이보다 큰 journal은 뒤쪽만 읽는다.
const MAX_READ_BYTES: u64 = 4 * 1024 * 1024;

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

#[derive(Serialize)]
struct Unsigned<'a> {
    v: u32,
    #[serde(flatten)]
    body: &'a Body,
}

#[derive(Deserialize)]
struct Record {
    v: u32,
    #[serde(flatten)]
    body: Body,
    sig: String,
}

#[derive(Serialize)]
struct Signed<'a> {
    v: u32,
    #[serde(flatten)]
    body: &'a Body,
    sig: String,
}

pub(crate) struct Journal {
    path: PathBuf,
    key: JournalKey,
}

impl Journal {
    pub(crate) fn at(git_dir: &Path, key: JournalKey) -> Self {
        Self {
            path: git_dir.join("higit").join("journal.jsonl"),
            key,
        }
    }

    pub(crate) fn append(&self, body: &Body) -> Result<()> {
        let text = canonical(body)?;
        let line = serde_json::to_string(&Signed {
            v: VERSION,
            body,
            sig: self.key.sign(&text),
        })
        .map_err(|e| CoreError::Journal(e.to_string()))?;
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| CoreError::Journal(e.to_string()))?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CoreError::Journal(e.to_string()))?;
        file.write_all(format!("{line}\n").as_bytes())
            .and_then(|()| file.sync_data())
            .map_err(|e| CoreError::Journal(e.to_string()))
    }

    /// 믿을 수 있는 줄만 순서대로. 파일이 없으면 빈 목록.
    pub(crate) fn read(&self) -> Vec<Body> {
        let Ok(bytes) = read_tail(&self.path) else {
            return Vec::new();
        };
        String::from_utf8_lossy(&bytes)
            .lines()
            .filter_map(|line| self.parse(line))
            .collect()
    }

    fn parse(&self, line: &str) -> Option<Body> {
        let record: Record = serde_json::from_str(line).ok()?;
        if record.v != VERSION {
            return None;
        }
        let text = canonical(&record.body).ok()?;
        if !self.key.verify(&text, &record.sig) {
            return None;
        }
        let valid = match &record.body {
            Body::Action {
                id, before, after, ..
            } => id.len() <= 64 && before.is_valid() && after.is_valid(),
            Body::Undo { target, .. } | Body::Redo { target, .. } => target.len() <= 64,
            Body::Barrier { .. } => true,
        };
        valid.then_some(record.body)
    }
}

fn canonical(body: &Body) -> Result<String> {
    serde_json::to_string(&Unsigned { v: VERSION, body })
        .map_err(|e| CoreError::Journal(e.to_string()))
}

fn read_tail(path: &Path) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > MAX_READ_BYTES {
        file.seek(SeekFrom::Start(len - MAX_READ_BYTES))?;
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
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
        mine.append(&action("내 줄")).unwrap();
        // 이름이 옵션처럼 보이는 줄은 서명이 맞아도 버린다.
        let mut bad = action("나쁜 줄");
        if let Body::Action { before, .. } = &mut bad {
            before.branch = Some("-x".into());
        }
        mine.append(&bad).unwrap();
        // 쓰다 끊긴 마지막 줄.
        let path = dir.path().join("higit/journal.jsonl");
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(b"{\"v\":1,\"type\":\"act").unwrap();
        let lines = mine.read();
        assert_eq!(lines, vec![action("내 줄")]);
    }
}
