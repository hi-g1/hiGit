//! 충돌 파일 읽기와 구간 계산(D-086). 블롭만 읽으므로 작업 폴더의 링크를 따라가지 않는다.

use git::{GitBackend, IndexEntry, MergeSegment, RepoPath};

use crate::Result;
use crate::view::{ConflictFileKind, ConflictFileView, ConflictHunkView, SideState};

/// 이보다 큰 쪽이 있으면 파일 단위로 고른다.
const MAX_TEXT: u64 = 1024 * 1024;
const CONTEXT: usize = 3;
const PREVIEW: usize = 40;

/// 경로 하나의 충돌 항목(stage 1: 원래, 2: 내 쪽, 3: 합칠 쪽).
#[derive(Debug, Clone)]
pub(crate) struct ConflictPath {
    pub path: String,
    pub base: Option<IndexEntry>,
    pub ours: Option<IndexEntry>,
    pub theirs: Option<IndexEntry>,
}

/// index의 충돌 항목을 경로별로 묶는다. 순서는 index 순서(경로 순).
pub(crate) fn conflict_paths(entries: Vec<IndexEntry>) -> Vec<ConflictPath> {
    let mut found: Vec<ConflictPath> = Vec::new();
    for e in entries.into_iter().filter(|e| e.stage > 0) {
        let at = match found.iter().position(|c| c.path == e.path) {
            Some(i) => i,
            None => {
                found.push(ConflictPath {
                    path: e.path.clone(),
                    base: None,
                    ours: None,
                    theirs: None,
                });
                found.len() - 1
            }
        };
        match e.stage {
            1 => found[at].base = Some(e),
            2 => found[at].ours = Some(e),
            _ => found[at].theirs = Some(e),
        }
    }
    found
}

/// 파일 하나를 읽어 구간을 나눈다. 텍스트로 다룰 수 없으면 `Whole`.
pub(crate) fn read_file(
    git: &dyn GitBackend,
    repo: &RepoPath,
    c: &ConflictPath,
) -> Result<ConflictFileView> {
    let oids: Vec<git::Oid> = [&c.base, &c.ours, &c.theirs]
        .iter()
        .filter_map(|e| e.as_ref().map(|e| e.oid.clone()))
        .collect();
    let blobs = git.read_blobs(repo, &oids, MAX_TEXT as usize)?;
    let side = |e: &Option<IndexEntry>| -> (SideState, Option<Vec<u8>>) {
        let Some(e) = e else {
            return (SideState::Deleted, None);
        };
        let Some(blob) = blobs.iter().find(|b| b.oid == e.oid) else {
            return (SideState::Binary, None);
        };
        match e.mode.as_str() {
            "120000" => {
                let target = String::from_utf8_lossy(&blob.head).into_owned();
                return (SideState::Link { target }, None);
            }
            "160000" => return (SideState::Submodule, None),
            _ => {}
        }
        if blob.size > MAX_TEXT {
            return (SideState::Large, None);
        }
        if blob.head.contains(&0) || std::str::from_utf8(&blob.head).is_err() {
            return (SideState::Binary, None);
        }
        (SideState::Text, Some(blob.head.clone()))
    };
    let (ours_side, ours) = side(&c.ours);
    let (theirs_side, theirs) = side(&c.theirs);
    let (_, base) = side(&c.base);
    let preview = |b: &Option<Vec<u8>>| b.as_deref().map(|b| lines(b, PREVIEW)).unwrap_or_default();
    let mut view = ConflictFileView {
        path: c.path.clone(),
        kind: ConflictFileKind::Whole,
        hunks: Vec::new(),
        ours_preview: preview(&ours),
        theirs_preview: preview(&theirs),
        ours_side,
        theirs_side,
    };
    // 모드가 다르거나 이름이 UTF-8이 아니면 파일 단위로만 고른다.
    let same_mode = matches!((&c.ours, &c.theirs), (Some(o), Some(t)) if o.mode == t.mode);
    let odd_name = c.path.contains('\u{FFFD}');
    if let (Some(ours), Some(theirs)) = (&ours, &theirs)
        && same_mode
        && !odd_name
    {
        let base = base.unwrap_or_default();
        if let Ok(segments) = git.merge_segments(repo, &base, ours, theirs) {
            let hunks = hunks(&segments);
            if !hunks.is_empty() {
                view.kind = ConflictFileKind::Text;
                view.hunks = hunks;
                view.ours_preview.clear();
                view.theirs_preview.clear();
            }
        }
    }
    Ok(view)
}

/// 앞에서 `max`줄, 줄 끝 없이.
fn lines(bytes: &[u8], max: usize) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .take(max)
        .map(str::to_string)
        .collect()
}

fn all_lines(bytes: &[u8]) -> Vec<String> {
    lines(bytes, usize::MAX)
}

/// 구간들에서 고를 곳과 앞뒤 맥락, 내 쪽 시작 줄 번호를 만든다.
pub(crate) fn hunks(segments: &[MergeSegment]) -> Vec<ConflictHunkView> {
    let mut out = Vec::new();
    let mut line = 1usize;
    for (i, seg) in segments.iter().enumerate() {
        match seg {
            MergeSegment::Same(bytes) => line += all_lines(bytes).len(),
            MergeSegment::Conflict { ours, base, theirs } => {
                let before = match i.checked_sub(1).and_then(|p| segments.get(p)) {
                    Some(MergeSegment::Same(b)) => {
                        let l = all_lines(b);
                        l[l.len().saturating_sub(CONTEXT)..].to_vec()
                    }
                    _ => Vec::new(),
                };
                let after = match segments.get(i + 1) {
                    Some(MergeSegment::Same(b)) => lines(b, CONTEXT),
                    _ => Vec::new(),
                };
                let ours_lines = all_lines(ours);
                out.push(ConflictHunkView {
                    index: u32::try_from(out.len()).unwrap_or(u32::MAX),
                    line: u32::try_from(line).unwrap_or(u32::MAX),
                    base: all_lines(base),
                    theirs: all_lines(theirs),
                    before,
                    after,
                    ours: ours_lines.clone(),
                });
                line += ours_lines.len();
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(s: &str) -> MergeSegment {
        MergeSegment::Same(s.as_bytes().to_vec())
    }

    #[test]
    fn hunk_lines_and_context() {
        let segments = [
            same("1\n2\n3\n4\n5\n"),
            MergeSegment::Conflict {
                ours: b"o1\r\no2\r\n".to_vec(),
                base: b"b\n".to_vec(),
                theirs: b"t\n".to_vec(),
            },
            same("6\n7\n8\n9\n"),
            MergeSegment::Conflict {
                ours: Vec::new(),
                base: b"x\n".to_vec(),
                theirs: b"y\n".to_vec(),
            },
        ];
        let h = hunks(&segments);
        assert_eq!(h.len(), 2);
        assert_eq!((h[0].index, h[0].line), (0, 6));
        assert_eq!(h[0].ours, ["o1", "o2"]);
        assert_eq!(h[0].before, ["3", "4", "5"]);
        assert_eq!(h[0].after, ["6", "7", "8"]);
        assert_eq!((h[1].index, h[1].line), (1, 12));
        assert!(h[1].ours.is_empty() && h[1].after.is_empty());
    }
}
