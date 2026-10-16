//! 앱이 시작한 작업의 표식(D-102). `<git 폴더>/higit/<이름>.json`에 둔다.
//!
//! 파일은 `{ "v": 1, "dir": "<git 폴더 실제 경로>", "body": "<내용 JSON 글자>", "sig": … }`이다.
//! 서명은 도메인 접두사(`higit-marker`)와 `v`·이름·`dir`·`body` 글자에 건다(journal 키를 쓰지만
//! journal 줄과 섞이지 않는다). `body`를 글자로 담는 까닭은 다시 직렬화해도 서명한 글자가 그대로이게
//! 하려는 것이다. 서명이 없거나 틀리거나 `dir`이 지금 실제 경로와 다르면 읽지 않는다: 남이 만든
//! 파일이나 다른 저장소에서 복사해 온 표식이 `--continue` 같은 문을 열지 못한다.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::undo::JournalKey;
use crate::{CoreError, Result};

const VERSION: u32 = 1;
const DOMAIN: &str = "higit-marker";
/// 표식 파일 크기 상한(기록 정리의 고친 메시지 64KB를 담고도 남는다).
const MAX_BYTES: u64 = 256 * 1024;

/// 표식 이름. 파일 이름이 되므로 상수만 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Name {
    /// 맡겨 둔 파일 꺼내기의 겹침(D-083).
    Unpark,
    /// 복사·되돌리기·기록 정리(D-101).
    Operation,
}

impl Name {
    fn as_str(self) -> &'static str {
        match self {
            Name::Unpark => "unpark",
            Name::Operation => "operation",
        }
    }
}

#[derive(Serialize, Deserialize)]
struct File {
    v: u32,
    dir: String,
    body: String,
    sig: String,
}

fn path(git_dir: &Path, name: Name) -> PathBuf {
    git_dir
        .join("higit")
        .join(format!("{}.json", name.as_str()))
}

/// `higit` 폴더와 표식 파일이 링크가 아닐 때만 쓰고 읽는다(journal과 같은 까닭).
fn place_ok(git_dir: &Path, name: Name) -> bool {
    let not_link = |p: &Path, want_dir: bool| match std::fs::symlink_metadata(p) {
        Ok(m) => !m.file_type().is_symlink() && m.is_dir() == want_dir,
        Err(_) => true,
    };
    not_link(&git_dir.join("higit"), true) && not_link(&path(git_dir, name), false)
}

/// git 폴더의 실제 경로 글자. 쓸 때와 읽을 때 같은 함수로 만든다.
fn real_dir(git_dir: &Path) -> Option<String> {
    std::fs::canonicalize(git_dir)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

fn signed_text(name: Name, dir: &str, body: &str) -> String {
    format!("{DOMAIN}\n{VERSION}\n{}\n{dir}\n{body}", name.as_str())
}

pub(crate) fn write<T: Serialize>(
    git_dir: &Path,
    name: Name,
    key: &JournalKey,
    body: &T,
) -> Result<()> {
    let fail = || CoreError::InvalidInput("작업 표식을 남기지 못했어요");
    if !place_ok(git_dir, name) {
        return Err(fail());
    }
    std::fs::create_dir_all(git_dir.join("higit")).map_err(|_| fail())?;
    // 만든 뒤 다시 본다(그사이 링크로 바뀌지 않았는지).
    if !place_ok(git_dir, name) {
        return Err(fail());
    }
    let dir = real_dir(git_dir).ok_or_else(fail)?;
    let body = serde_json::to_string(body).map_err(|_| fail())?;
    let sig = key.sign(&signed_text(name, &dir, &body));
    let text = serde_json::to_string(&File {
        v: VERSION,
        dir,
        body,
        sig,
    })
    .map_err(|_| fail())?;
    std::fs::write(path(git_dir, name), text).map_err(|_| fail())
}

/// 서명과 저장소 위치가 맞는 표식의 내용. 아니면 `None`(내용 형식 검사는 부르는 쪽이 한다).
pub(crate) fn read<T: DeserializeOwned>(git_dir: &Path, name: Name, key: &JournalKey) -> Option<T> {
    if !place_ok(git_dir, name) {
        return None;
    }
    let file = path(git_dir, name);
    if std::fs::metadata(&file).ok()?.len() > MAX_BYTES {
        return None;
    }
    let text = std::fs::read_to_string(file).ok()?;
    let parsed: File = serde_json::from_str(&text).ok()?;
    if parsed.v != VERSION || Some(&parsed.dir) != real_dir(git_dir).as_ref() {
        return None;
    }
    if !key.verify(&signed_text(name, &parsed.dir, &parsed.body), &parsed.sig) {
        return None;
    }
    serde_json::from_str(&parsed.body).ok()
}

/// 표식이 있는지(서명은 보지 않는다). 남은 표식을 정리할지 정할 때 쓴다.
pub(crate) fn exists(git_dir: &Path, name: Name) -> bool {
    place_ok(git_dir, name) && path(git_dir, name).is_file()
}

pub(crate) fn clear(git_dir: &Path, name: Name) {
    if place_ok(git_dir, name) {
        let _ = std::fs::remove_file(path(git_dir, name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Body {
        branch: String,
        n: u32,
    }

    fn body() -> Body {
        Body {
            branch: "기능/한글".into(),
            n: 3,
        }
    }

    #[test]
    fn round_trip_and_clear() {
        let dir = tempfile::tempdir().unwrap();
        let key = JournalKey::ephemeral();
        assert_eq!(read::<Body>(dir.path(), Name::Unpark, &key), None);
        write(dir.path(), Name::Unpark, &key, &body()).unwrap();
        assert_eq!(read(dir.path(), Name::Unpark, &key), Some(body()));
        // 이름이 다르면 다른 파일이다.
        assert_eq!(read::<Body>(dir.path(), Name::Operation, &key), None);
        assert!(exists(dir.path(), Name::Unpark));
        clear(dir.path(), Name::Unpark);
        assert!(!exists(dir.path(), Name::Unpark));
        assert_eq!(read::<Body>(dir.path(), Name::Unpark, &key), None);
    }

    #[test]
    fn refuses_tampered_foreign_or_unsigned_files() {
        let dir = tempfile::tempdir().unwrap();
        let key = JournalKey::ephemeral();
        write(dir.path(), Name::Unpark, &key, &body()).unwrap();
        let file = dir.path().join("higit/unpark.json");
        let text = std::fs::read_to_string(&file).unwrap();
        // 다른 키.
        assert_eq!(
            read::<Body>(dir.path(), Name::Unpark, &JournalKey::ephemeral()),
            None
        );
        // 내용 한 글자 바꿈.
        std::fs::write(&file, text.replace("\\\"n\\\":3", "\\\"n\\\":4")).unwrap();
        assert_ne!(std::fs::read_to_string(&file).unwrap(), text);
        assert_eq!(read::<Body>(dir.path(), Name::Unpark, &key), None);
        // 다른 저장소 폴더로 복사.
        let other = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(other.path().join("higit")).unwrap();
        std::fs::write(other.path().join("higit/unpark.json"), &text).unwrap();
        assert_eq!(read::<Body>(other.path(), Name::Unpark, &key), None);
        // 서명 없는 옛 형식.
        std::fs::write(
            &file,
            r#"{"branch":"main","oid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        )
        .unwrap();
        assert_eq!(read::<Body>(dir.path(), Name::Unpark, &key), None);
        // 너무 큼.
        std::fs::write(&file, " ".repeat(MAX_BYTES as usize + 1)).unwrap();
        assert_eq!(read::<Body>(dir.path(), Name::Unpark, &key), None);
    }

    #[test]
    fn fixed_key_signature_is_stable() {
        // 같은 키·내용이면 같은 서명(형식을 바꾸면 이 테스트가 알려 준다).
        let key = JournalKey::fixed(7);
        assert_eq!(
            key.sign(&signed_text(Name::Unpark, "/r/.git", "{}")),
            key.sign("higit-marker\n1\nunpark\n/r/.git\n{}")
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_linked_folder() {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let key = JournalKey::ephemeral();
        std::os::unix::fs::symlink(elsewhere.path(), dir.path().join("higit")).unwrap();
        assert!(write(dir.path(), Name::Unpark, &key, &body()).is_err());
        assert_eq!(read::<Body>(dir.path(), Name::Unpark, &key), None);
    }
}
