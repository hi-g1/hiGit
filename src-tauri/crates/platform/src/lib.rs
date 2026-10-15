//! OS별 코드는 이 크레이트에만 둔다(D-002).
//!
//! 다른 내부 크레이트에 의존하지 않는다. `git` 크레이트는 이 크레이트를 모르므로
//! core가 여기서 git 경로와 [`configure_child`]를 받아 `CliBackend`에 넘긴다.

#![forbid(unsafe_code)]

mod app_dir;
mod git_exe;
mod link;
mod path;
mod process;

pub use app_dir::{DATA_DIR_ENV, app_data_dir, home_dir, open_private_append, write_private_file};
pub use git_exe::{GitBuild, find_git, git_build, git_download_url};
pub use link::{symlink_dir, symlink_file};
pub use path::{git_path_string, normalize_repo_path, same_path};
pub use process::{configure_child, kill_tree, prepare_killable, terminate_tree};
