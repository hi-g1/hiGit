//! 충돌 해결 화면에서 고른 것(D-211). `<git 폴더>/higit/conflict-draft.json`에 서명해 둔다(`marker`).
//!
//! 웹뷰 저장소에 두지 않는 까닭: 직접 고친 코드 글이 저장소 밖에 남지 않게. 서명은 받은 zip에 심어 둔 선택이
//! "마치기" 한 번에 쓰이는 것을 막고, 합칠 쪽 전체 해시를 함께 묶어 다른 합치기에 이어 쓰지 않는다.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::marker::{self, Name};
use crate::undo::JournalKey;
use crate::view::{
    ConflictDraftView, ConflictFileKind, ConflictFileSummary, DraftPick, HunkChoice,
};
use crate::{CoreError, Result};

/// 직접 고친 글 하나의 상한(`ConflictFinishInput`과 같음).
const MAX_TEXT: usize = 1024 * 1024;
/// 선택 전체의 상한.
const MAX_TOTAL: usize = 8 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Stored {
    /// 합칠 쪽 버전 전체 해시.
    theirs: String,
    draft: ConflictDraftView,
}

/// 지금 충돌 경로·곳 수에 맞는지. 경로가 겹치거나 없는 경로·넘치는 곳 번호·너무 긴 글이면 거짓.
fn fits(draft: &ConflictDraftView, files: &[ConflictFileSummary]) -> bool {
    let mut seen = std::collections::HashSet::new();
    draft.files.iter().all(|f| {
        let Some(file) = files.iter().find(|c| c.path == f.path) else {
            return false;
        };
        if !seen.insert(&f.path) {
            return false;
        }
        match &f.pick {
            DraftPick::Hunks { picks } => {
                file.kind == ConflictFileKind::Text
                    && picks.len() <= file.hunks as usize
                    && picks.iter().flatten().all(|p| match p {
                        HunkChoice::Custom { text } => text.len() <= MAX_TEXT,
                        _ => true,
                    })
            }
            DraftPick::Ours | DraftPick::Theirs => true,
        }
    })
}

/// 고른 것을 남긴다. 화면이 보낸 경로는 지금 충돌 경로와 다시 맞춘다.
pub(crate) fn save(
    git_dir: &Path,
    key: &JournalKey,
    theirs: &str,
    draft: &ConflictDraftView,
    files: &[ConflictFileSummary],
) -> Result<()> {
    if !fits(draft, files) {
        return Err(CoreError::InvalidInput("지금 충돌과 맞지 않는 선택이에요"));
    }
    let size = serde_json::to_string(draft)
        .map(|t| t.len())
        .unwrap_or(usize::MAX);
    if size > MAX_TOTAL {
        return Err(CoreError::InvalidInput("고른 내용이 너무 커요"));
    }
    marker::write(
        git_dir,
        Name::ConflictDraft,
        key,
        &Stored {
            theirs: theirs.to_string(),
            draft: draft.clone(),
        },
    )
}

/// 같은 합치기의 서명이 맞는 선택. 합칠 쪽이 바뀌었으면 지운다. 경로·곳 수가 맞지 않으면 쓰지 않는다.
pub(crate) fn load(
    git_dir: &Path,
    key: &JournalKey,
    theirs: &str,
    files: &[ConflictFileSummary],
) -> Option<ConflictDraftView> {
    let stored: Stored = marker::read(git_dir, Name::ConflictDraft, key)?;
    if stored.theirs != theirs {
        clear(git_dir);
        return None;
    }
    fits(&stored.draft, files).then_some(stored.draft)
}

pub(crate) fn clear(git_dir: &Path) {
    marker::clear(git_dir, Name::ConflictDraft);
}
