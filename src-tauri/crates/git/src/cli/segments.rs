//! 세 쪽을 합친 구간 나누기(충돌 해결 화면, D-086).
//!
//! `merge-file --diff3`의 표시 줄을 읽는다. 표시는 길이 40으로 정해 두고, 입력 어디에든 그 길이의
//! 표시 같은 줄이 있으면 구간을 믿을 수 없으니 오류로 돌려 화면이 파일 단위 선택으로 떨어지게 한다.

use std::ffi::OsStr;

use super::CliBackend;
use super::merge_check::Scratch;
use crate::{GitError, MergeSegment, RepoPath, Result};

const MARKER: usize = 40;

impl CliBackend {
    pub(super) fn segments(
        &self,
        repo: &RepoPath,
        base: &[u8],
        ours: &[u8],
        theirs: &[u8],
    ) -> Result<Vec<MergeSegment>> {
        if [base, ours, theirs].iter().any(|b| has_marker_line(b)) {
            return Err(GitError::Parse("충돌 표시와 같은 줄이 있음".into()));
        }
        let scratch = Scratch::new()?;
        let mut files = Vec::new();
        for (name, bytes) in [("ours", ours), ("base", base), ("theirs", theirs)] {
            let path = scratch.path(name);
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .and_then(|mut f| std::io::Write::write_all(&mut f, bytes))
                .map_err(GitError::Spawn)?;
            files.push(path);
        }
        let paths: Vec<&OsStr> = files.iter().map(|p| p.as_os_str()).collect();
        let size = format!("--marker-size={MARKER}");
        let (code, out) = self.runner.merge_file_print(
            Some(repo.as_path()),
            &["--diff3", &size, "-L", "O", "-L", "B", "-L", "T"],
            &paths,
        )?;
        if !(0..=127).contains(&code) {
            return Err(GitError::Parse("merge-file: 이진 파일이거나 실패".into()));
        }
        parse_segments(&out)
    }
}

/// 줄 시작이 같은 글자 `MARKER`개 이상인 표시 모양(`<`, `|`, `=`, `>`)인지.
fn is_marker(line: &[u8], c: u8) -> bool {
    line.len() >= MARKER && line[..MARKER].iter().all(|b| *b == c)
}

fn has_marker_line(bytes: &[u8]) -> bool {
    bytes
        .split(|b| *b == b'\n')
        .any(|l| b"<|=>".iter().any(|c| is_marker(l, *c)))
}

/// 표시 줄 뒤에 붙은 이름표만 확인한다(`<<<…<< O`). 줄 끝의 `\r`은 무시한다.
fn marker_with(line: &[u8], c: u8, label: Option<&str>) -> bool {
    if !is_marker(line, c) {
        return false;
    }
    let rest = &line[MARKER..];
    let rest = rest.strip_suffix(b"\r").unwrap_or(rest);
    match label {
        Some(l) => rest.strip_prefix(b" ") == Some(l.as_bytes()),
        None => rest.is_empty(),
    }
}

pub(super) fn parse_segments(out: &[u8]) -> Result<Vec<MergeSegment>> {
    enum At {
        Same,
        Ours,
        Base,
        Theirs,
    }
    let bad = || GitError::Parse("merge-file 출력".into());
    let mut segments = Vec::new();
    let mut same = Vec::new();
    let (mut ours, mut base, mut theirs) = (Vec::new(), Vec::new(), Vec::new());
    let mut at = At::Same;
    // 줄 끝까지 담아 나눈다(마지막 줄에 줄바꿈이 없을 수 있다).
    for line in out.split_inclusive(|b| *b == b'\n') {
        let body = line.strip_suffix(b"\n").unwrap_or(line);
        match at {
            At::Same if marker_with(body, b'<', Some("O")) => {
                if !same.is_empty() {
                    segments.push(MergeSegment::Same(std::mem::take(&mut same)));
                }
                at = At::Ours;
            }
            At::Same => same.extend_from_slice(line),
            At::Ours if marker_with(body, b'|', Some("B")) => at = At::Base,
            At::Ours => ours.extend_from_slice(line),
            At::Base if marker_with(body, b'=', None) => at = At::Theirs,
            At::Base => base.extend_from_slice(line),
            At::Theirs if marker_with(body, b'>', Some("T")) => {
                segments.push(MergeSegment::Conflict {
                    ours: std::mem::take(&mut ours),
                    base: std::mem::take(&mut base),
                    theirs: std::mem::take(&mut theirs),
                });
                at = At::Same;
            }
            At::Theirs => theirs.extend_from_slice(line),
        }
    }
    if !matches!(at, At::Same) {
        return Err(bad());
    }
    if !same.is_empty() {
        segments.push(MergeSegment::Same(same));
    }
    Ok(segments)
}
