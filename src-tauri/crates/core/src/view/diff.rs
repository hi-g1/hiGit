//! 비교 보기 화면 타입.

use serde::{Deserialize, Serialize};
use specta::Type;

/// 비교할 파일. 바뀐 내용 목록의 항목에서 만든다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffRequest {
    pub path: String,
    pub orig_path: Option<String>,
    /// 아직 추적하지 않는 새 파일이면 true.
    pub untracked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileDiffView {
    pub binary: bool,
    pub truncated: bool,
    pub hunks: Vec<HunkView>,
    /// 심볼릭 링크면 가리키는 곳. 이때는 내용을 읽지 않고 줄도 없다(D-052).
    pub link: Option<String>,
}

impl FileDiffView {
    pub(crate) fn link(target: String) -> Self {
        Self {
            binary: false,
            truncated: false,
            hunks: Vec::new(),
            link: Some(target),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HunkView {
    pub section: String,
    pub lines: Vec<DiffLineView>,
}

/// 줄 하나. 줄 번호는 화면에 바로 쓰도록 core가 계산한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineView {
    pub kind: LineKindView,
    pub text: String,
    /// 마지막 저장 쪽 줄 번호. 추가된 줄은 null.
    pub old_line: Option<u32>,
    /// 지금 쪽 줄 번호. 지운 줄은 null.
    pub new_line: Option<u32>,
    pub no_newline_at_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LineKindView {
    Context,
    Added,
    Removed,
}

impl From<&DiffRequest> for git::DiffTarget {
    fn from(r: &DiffRequest) -> Self {
        git::DiffTarget {
            path: r.path.clone(),
            orig_path: r.orig_path.clone(),
            kind: if r.untracked {
                git::DiffKind::Untracked
            } else {
                git::DiffKind::Tracked
            },
        }
    }
}

impl From<git::FileDiff> for FileDiffView {
    fn from(d: git::FileDiff) -> Self {
        Self {
            binary: d.binary,
            truncated: d.truncated,
            hunks: d.hunks.into_iter().map(HunkView::from).collect(),
            link: None,
        }
    }
}

impl From<git::Hunk> for HunkView {
    fn from(h: git::Hunk) -> Self {
        let mut old = h.old_start;
        let mut new = h.new_start;
        let lines = h
            .lines
            .into_iter()
            .map(|l| {
                let (kind, old_line, new_line) = match l.kind {
                    git::LineKind::Context => {
                        let n = (Some(old), Some(new));
                        old += 1;
                        new += 1;
                        (LineKindView::Context, n.0, n.1)
                    }
                    git::LineKind::Removed => {
                        old += 1;
                        (LineKindView::Removed, Some(old - 1), None)
                    }
                    git::LineKind::Added => {
                        new += 1;
                        (LineKindView::Added, None, Some(new - 1))
                    }
                };
                DiffLineView {
                    kind,
                    text: l.text,
                    old_line,
                    new_line,
                    no_newline_at_end: l.no_newline_at_end,
                }
            })
            .collect();
        Self {
            section: h.section,
            lines,
        }
    }
}
