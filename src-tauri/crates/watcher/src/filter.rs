//! 어떤 경로 변경을 알릴지 고른다. notify와 떨어져 있어 단위 테스트가 쉽다.

use std::path::{Component, Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// 작업 폴더의 파일이 바뀌었다.
    Worktree,
    /// `.gitignore`가 바뀌었다. 규칙을 다시 읽고 작업 폴더 변경으로도 알린다.
    GitignoreChanged,
    /// `.git` 안에서 저장·갈래·index가 바뀌었다.
    Git,
    /// 알리지 않는다.
    Skip,
}

/// `.git` 안에서 화면에 영향을 주는 파일. 잠금 파일, objects, logs는 뺀다.
fn is_git_relevant(rel: &Path) -> bool {
    let name = rel.to_string_lossy();
    if name.ends_with(".lock") {
        return false;
    }
    match rel.components().next() {
        Some(Component::Normal(first)) if first == "refs" => true,
        _ => matches!(
            name.as_ref(),
            "HEAD" | "index" | "packed-refs" | "FETCH_HEAD" | "ORIG_HEAD" | "MERGE_HEAD"
        ),
    }
}

/// 저장소의 무시 규칙. 루트 `.gitignore`, `.git/info/exclude`, 전역 제외 파일,
/// 하위 폴더의 `.gitignore`(그 폴더 기준)를 함께 쓴다. 깊은 규칙이 얕은 규칙을 이긴다.
pub(crate) struct IgnoreRules {
    /// (규칙이 적용되는 폴더, 규칙). 얕은 것부터.
    matchers: Vec<(PathBuf, Gitignore)>,
}

impl IgnoreRules {
    pub(crate) fn load(root: &Path, extra_excludes: &[PathBuf]) -> Self {
        let mut builder = GitignoreBuilder::new(root);
        for file in [root.join(".gitignore"), root.join(".git/info/exclude")]
            .iter()
            .chain(extra_excludes)
        {
            // 없는 파일은 무시한다.
            let _ = builder.add(file);
        }
        let mut matchers = vec![(
            root.to_path_buf(),
            builder.build().unwrap_or_else(|_| Gitignore::empty()),
        )];

        // 하위 폴더의 .gitignore. 무시된 폴더 안은 볼 필요가 없으므로 ignore의 걷기 규칙을 그대로 쓴다.
        let walker = ignore::WalkBuilder::new(root)
            .hidden(false)
            .parents(false)
            .filter_entry(|e| e.file_name() != ".git")
            .build();
        let mut nested: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == ".gitignore" && e.depth() > 1)
            .map(|e| e.into_path())
            .collect();
        nested.sort_by_key(|p| p.components().count());
        for file in nested {
            let (gi, _) = Gitignore::new(&file);
            if let Some(dir) = file.parent() {
                matchers.push((dir.to_path_buf(), gi));
            }
        }
        Self { matchers }
    }

    pub(crate) fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let mut ignored = false;
        for (dir, gi) in &self.matchers {
            if !path.starts_with(dir) || path == dir {
                continue;
            }
            let m = gi.matched_path_or_any_parents(path, is_dir);
            if m.is_ignore() {
                ignored = true;
            } else if m.is_whitelist() {
                ignored = false;
            }
        }
        ignored
    }
}

pub(crate) fn classify(root: &Path, rules: &IgnoreRules, path: &Path) -> Kind {
    let Ok(rel) = path.strip_prefix(root) else {
        return Kind::Skip;
    };
    if let Ok(git_rel) = rel.strip_prefix(".git") {
        return if is_git_relevant(git_rel) {
            Kind::Git
        } else {
            Kind::Skip
        };
    }
    if rel.as_os_str().is_empty() {
        return Kind::Skip;
    }
    if path.file_name().is_some_and(|n| n == ".gitignore") {
        return Kind::GitignoreChanged;
    }
    if rules.is_ignored(path, path.is_dir()) {
        Kind::Skip
    } else {
        Kind::Worktree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(".git/info")).unwrap();
        fs::write(root.join(".gitignore"), "node_modules/\n*.log\n!keep.log\n").unwrap();
        fs::write(root.join(".git/info/exclude"), "secret.txt\n").unwrap();
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/.gitignore"), "build/\n").unwrap();
        dir
    }

    #[test]
    fn git_internal_files() {
        let dir = setup();
        let root = dir.path();
        let rules = IgnoreRules::load(root, &[]);
        let k = |p: &str| classify(root, &rules, &root.join(p));
        assert_eq!(k(".git/HEAD"), Kind::Git);
        assert_eq!(k(".git/index"), Kind::Git);
        assert_eq!(k(".git/refs/heads/main"), Kind::Git);
        assert_eq!(k(".git/packed-refs"), Kind::Git);
        assert_eq!(k(".git/index.lock"), Kind::Skip);
        assert_eq!(k(".git/refs/heads/main.lock"), Kind::Skip);
        assert_eq!(k(".git/objects/ab/cdef"), Kind::Skip);
        assert_eq!(k(".git/logs/HEAD"), Kind::Skip);
    }

    #[test]
    fn ignored_worktree_files() {
        let dir = setup();
        let root = dir.path();
        let extra = root.join("global-ignore");
        fs::write(&extra, "*.tmp\n").unwrap();
        let rules = IgnoreRules::load(root, &[extra]);
        let k = |p: &str| classify(root, &rules, &root.join(p));
        assert_eq!(k("src/한글 파일.txt"), Kind::Worktree);
        assert_eq!(k("node_modules/react/index.js"), Kind::Skip);
        assert_eq!(k("debug.log"), Kind::Skip);
        assert_eq!(k("keep.log"), Kind::Worktree);
        assert_eq!(k("secret.txt"), Kind::Skip);
        assert_eq!(k("a.tmp"), Kind::Skip);
        assert_eq!(k("sub/build/out.js"), Kind::Skip);
        assert_eq!(
            k("build/out.js"),
            Kind::Worktree,
            "하위 .gitignore는 그 폴더에만 적용"
        );
        assert_eq!(k("sub/.gitignore"), Kind::GitignoreChanged);
        assert_eq!(k(".gitignore"), Kind::GitignoreChanged);
    }

    #[test]
    fn outside_root_is_skipped() {
        let dir = setup();
        let rules = IgnoreRules::load(dir.path(), &[]);
        assert_eq!(
            classify(dir.path(), &rules, Path::new("/다른/곳")),
            Kind::Skip
        );
        assert_eq!(classify(dir.path(), &rules, dir.path()), Kind::Skip);
    }
}
