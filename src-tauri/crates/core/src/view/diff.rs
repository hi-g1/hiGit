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
    /// 이 버전에서 바뀐 것(첫 부모와 비교)을 본다(D-105). 없으면 작업 폴더의 변경.
    #[serde(default)]
    pub commit: Option<String>,
    /// 파일 전체를 문맥으로 읽는다(접힌 줄 펼치기, D-156). 없으면 거짓.
    #[serde(default)]
    pub full_context: bool,
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
    /// 묶음의 시작 줄과 길이(마지막 저장 쪽·지금 쪽). 묶음 사이에 접힌 줄 수를 센다(D-156).
    pub old_start: u32,
    pub old_len: u32,
    pub new_start: u32,
    pub new_len: u32,
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
            // 버전 안의 비교(`commit`)는 해시를 다시 확인하는 `Queries::diff`가 따로 만든다.
            kind: if r.untracked {
                git::DiffKind::Untracked
            } else {
                git::DiffKind::Tracked
            },
            full_context: r.full_context,
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
            old_start: h.old_start,
            old_len: h.old_len,
            new_start: h.new_start,
            new_len: h.new_len,
            lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_requests_read_full_context_as_false() {
        let r: DiffRequest =
            serde_json::from_str(r#"{"path":"a.txt","origPath":null,"untracked":false}"#).unwrap();
        assert!(!r.full_context);
        assert!(git::DiffTarget::from(&r).full_context == r.full_context);
        let full: DiffRequest = serde_json::from_str(
            r#"{"path":"a.txt","origPath":null,"untracked":false,"fullContext":true}"#,
        )
        .unwrap();
        assert!(git::DiffTarget::from(&full).full_context);
    }

    #[test]
    fn hunk_view_keeps_ranges() {
        let view = HunkView::from(git::Hunk {
            old_start: 10,
            old_len: 3,
            new_start: 12,
            new_len: 4,
            section: "fn a()".into(),
            lines: Vec::new(),
        });
        assert_eq!(
            (view.old_start, view.old_len, view.new_start, view.new_len),
            (10, 3, 12, 4)
        );
    }
}
