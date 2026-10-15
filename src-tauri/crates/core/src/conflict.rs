//! 충돌 파일 읽기와 구간 계산(D-086). 블롭만 읽으므로 작업 폴더의 링크를 따라가지 않는다.

use git::{GitBackend, IndexEntry, MergeSegment, RepoPath};

use crate::Result;
use crate::view::{ConflictFileKind, ConflictFileView, ConflictHunkView, SideState};

/// 이보다 큰 쪽이 있으면 파일 단위로 고른다.
const MAX_TEXT: u64 = 1024 * 1024;
const CONTEXT: usize = 3;
const PREVIEW: usize = 40;

/// 지금 충돌을 남긴 작업.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Context {
    /// 합치기(`MERGE_HEAD`).
    Merge,
    /// 앱이 시작한 맡긴 파일 꺼내기(D-083). 표식과 맡긴 ref가 서로 맞을 때만.
    Unpark(crate::parked::UnparkMarker),
}

/// 다른 도구가 시작한 리베이스·cherry-pick·revert. 앱이 끝내지 않는다.
pub(crate) fn other_operation(git_dir: &std::path::Path) -> Option<crate::view::OtherOperation> {
    use crate::view::OtherOperation as O;
    [
        ("rebase-merge", O::Rebase),
        ("rebase-apply", O::Rebase),
        ("CHERRY_PICK_HEAD", O::CherryPick),
        ("REVERT_HEAD", O::Revert),
    ]
    .into_iter()
    .find(|(f, _)| git_dir.join(f).exists())
    .map(|(_, op)| op)
}

/// 앱이 다룰 수 있는 충돌 맥락. 합치는 중이면 `Merge`, 앱이 시작한 꺼내기가 충돌을 남겼으면
/// `Unpark`, 아니면 `None`(다른 도구의 작업이거나 충돌 없음).
pub(crate) fn context(git: &dyn GitBackend, repo: &RepoPath) -> Result<Option<Context>> {
    if git.resolve_ref(repo, "MERGE_HEAD")?.is_some() {
        return Ok(Some(Context::Merge));
    }
    let Some(marker) = crate::parked::read_marker(&git.git_dir(repo)?) else {
        return Ok(None);
    };
    let on_branch = git.current_branch(repo)?.as_deref() == Some(marker.branch.as_str());
    let parked = git.resolve_ref(repo, &crate::parked::ref_name(&marker.branch))?;
    let same = parked.as_ref().map(|o| o.as_str()) == Some(marker.oid.as_str());
    let conflicted = git.index_entries(repo)?.iter().any(|e| e.stage > 0);
    Ok((on_branch && same && conflicted).then_some(Context::Unpark(marker)))
}

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
    load_file(git, repo, c).map(|(view, _)| view)
}

/// `read_file`과 같고, 텍스트로 다루면 조립에 쓸 구간도 돌려준다.
pub(crate) fn load_file(
    git: &dyn GitBackend,
    repo: &RepoPath,
    c: &ConflictPath,
) -> Result<(ConflictFileView, Option<Vec<MergeSegment>>)> {
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
    let mut kept = None;
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
                kept = Some(segments);
            }
        }
    }
    Ok((view, kept))
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

/// 고를 곳 하나의 선택(조립용). 화면 입력에서 옮긴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pick<'a> {
    Ours,
    Theirs,
    /// 내 쪽 다음 합칠 쪽.
    Both,
    /// 직접 고친 글(줄 끝은 `\n`).
    Custom(&'a str),
}

/// 구간과 선택으로 해결한 파일 내용을 만든다. 선택 수가 고를 곳 수와 다르면 `None`.
/// 직접 고친 글은 내 쪽이 CRLF면 CRLF로 바꾸고, 그 곳의 원래 내용이 줄바꿈으로 끝났으면
/// 줄바꿈으로 끝나게 맞춘다.
pub(crate) fn assemble(segments: &[MergeSegment], picks: &[Pick<'_>]) -> Option<Vec<u8>> {
    let conflicts = segments
        .iter()
        .filter(|s| matches!(s, MergeSegment::Conflict { .. }))
        .count();
    if conflicts != picks.len() {
        return None;
    }
    let mut out = Vec::new();
    let mut picks = picks.iter();
    for seg in segments {
        match seg {
            MergeSegment::Same(bytes) => out.extend_from_slice(bytes),
            MergeSegment::Conflict { ours, theirs, .. } => match picks.next()? {
                Pick::Ours => out.extend_from_slice(ours),
                Pick::Theirs => out.extend_from_slice(theirs),
                Pick::Both => {
                    out.extend_from_slice(ours);
                    out.extend_from_slice(theirs);
                }
                Pick::Custom(text) => {
                    let crlf = ours.windows(2).any(|w| w == b"\r\n")
                        || (ours.is_empty() && theirs.windows(2).any(|w| w == b"\r\n"));
                    let ends_with_newline = ours.ends_with(b"\n") || theirs.ends_with(b"\n");
                    let mut text = text.replace("\r\n", "\n");
                    if ends_with_newline && !text.is_empty() && !text.ends_with('\n') {
                        text.push('\n');
                    }
                    if crlf {
                        text = text.replace('\n', "\r\n");
                    }
                    out.extend_from_slice(text.as_bytes());
                }
            },
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conflict(ours: &str, theirs: &str) -> MergeSegment {
        MergeSegment::Conflict {
            ours: ours.as_bytes().to_vec(),
            base: Vec::new(),
            theirs: theirs.as_bytes().to_vec(),
        }
    }

    #[test]
    fn assemble_each_pick() {
        let segs = [
            same("앞\n"),
            conflict("내1\n", "남1\n"),
            same("가운데\n"),
            conflict("내2\n", "남2\n"),
            conflict("내3\n", "남3\n"),
            conflict("내4\n", "남4\n"),
        ];
        let out = assemble(
            &segs,
            &[
                Pick::Ours,
                Pick::Theirs,
                Pick::Both,
                Pick::Custom("직접\n고침"),
            ],
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "앞\n내1\n가운데\n남2\n내3\n남3\n직접\n고침\n"
        );
        assert!(assemble(&segs, &[Pick::Ours]).is_none());
    }

    #[test]
    fn custom_text_follows_crlf_and_can_be_empty() {
        let segs = [same("a\r\n"), conflict("x\r\n", "y\r\n"), same("b\r\n")];
        let out = assemble(&segs, &[Pick::Custom("하나\n둘")]).unwrap();
        assert_eq!(out, "a\r\n하나\r\n둘\r\nb\r\n".as_bytes());
        // 비우면 그 곳을 지운다.
        let out = assemble(&segs, &[Pick::Custom("")]).unwrap();
        assert_eq!(out, b"a\r\nb\r\n");
    }

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
