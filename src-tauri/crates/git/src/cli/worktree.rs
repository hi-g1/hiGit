//! 저장소 설정 `core.worktree`가 작업 폴더를 다른 곳으로 옮겼는지 판정한다(D-212).
//!
//! 받은 저장소의 `.git/config`에 `core.worktree = <다른 폴더>`가 있으면 git은 그 폴더를 작업 폴더로 쓴다.
//! 앱이 그 폴더의 파일을 지우거나 덮어쓰지 않게, 작업 폴더의 `.git`이 git 폴더를 가리킬 때만 믿는다.
//! 서브모듈과 연결된 worktree는 작업 폴더의 `.git` 파일이 git 폴더를 가리키므로 그대로 열린다.

use std::io::Read;
use std::path::{Path, PathBuf};

/// `.git` 파일(`gitdir: <경로>`)을 읽는 상한.
const GITFILE_LIMIT: u64 = 4096;

/// `top/.git`이 `git_dir`인지(폴더), 또는 `git_dir`을 가리키는 `.git` 파일인지.
pub(super) fn points_to(top: &Path, git_dir: &Path) -> bool {
    let dot = top.join(".git");
    let Ok(meta) = std::fs::symlink_metadata(&dot) else {
        return false;
    };
    if meta.is_dir() {
        return same_place(&dot, git_dir);
    }
    if !meta.is_file() {
        return false;
    }
    let mut text = String::new();
    let Ok(file) = std::fs::File::open(&dot) else {
        return false;
    };
    if file.take(GITFILE_LIMIT).read_to_string(&mut text).is_err() {
        return false;
    }
    let Some(target) = text.lines().next().and_then(|l| l.strip_prefix("gitdir:")) else {
        return false;
    };
    same_place(&top.join(target.trim()), git_dir)
}

/// 작업 폴더를 믿을 수 없을 때 대신 열 폴더: git 폴더가 `.git`이라는 이름의 폴더이고 연결된 worktree가
/// 아니면 그 부모. 아니면 `None`(맨 저장소, `--separate-git-dir`, worktree 설정으로 옮긴 경우).
pub(super) fn owner_of(git_dir: &Path, common_dir: &Path) -> Option<PathBuf> {
    if git_dir.file_name()? != ".git" || !same_place(git_dir, common_dir) {
        return None;
    }
    git_dir.parent().map(Path::to_path_buf)
}

fn same_place(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
