//! 제외할 파일에 추가(`ignore`, D-040, D-077). 저장소 최상위 `.gitignore`에 `/경로` 줄을 덧붙인다.

use std::path::Path;

use git::{Change, GitOp};

use super::{paths, repo_of};
use crate::view::ActionInput;
use crate::{
    Action, ActionMeta, Availability, CoreError, ExecCtx, Outcome, RepoState, Result, Risk,
};

/// 한 번에 제외하는 파일 수 상한. 되돌리기가 덧붙인 줄을 모두 알아야 해서 journal 사본 상한과 같다.
pub(crate) const MAX_IGNORE: usize = 200;
const FILE: &str = ".gitignore";

pub struct IgnoreAction {
    meta: ActionMeta,
}

impl IgnoreAction {
    pub fn new() -> Self {
        Self {
            meta: ActionMeta {
                id: "ignore",
                glossary_id: "ignore",
                risk: Risk::Safe,
                undo: crate::undo::UndoKind::Inverse,
            },
        }
    }
}

impl Default for IgnoreAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for IgnoreAction {
    fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    fn available(&self, state: &RepoState) -> Availability {
        super::needs_repo(state)
    }

    fn moves_head(&self) -> bool {
        false
    }

    fn execute(&self, ctx: &ExecCtx<'_>, input: &ActionInput) -> Result<Outcome> {
        let ActionInput::Ignore(input) = input else {
            return Err(CoreError::InputMismatch("ignore"));
        };
        paths::validate(&input.paths)?;
        if input.paths.len() > MAX_IGNORE {
            return Err(CoreError::InvalidInput("한 번에 고른 파일이 너무 많아요"));
        }
        let repo = repo_of(ctx)?;
        // 새 파일(추적 안 함)이나 이번에 새로 넣은 파일만 받는다. 이미 버전에 든 파일은 규칙으로 빠지지 않는다.
        let status = ctx.git.status(repo)?;
        let mut staged = Vec::new();
        for path in &input.paths {
            let entry = status
                .entries
                .iter()
                .find(|e| &e.path == path)
                .ok_or(CoreError::InvalidInput("새 파일만 제외할 수 있어요"))?;
            if entry.kind == git::EntryKind::Untracked {
                continue;
            }
            if entry.index == Change::Added && entry.kind == git::EntryKind::Ordinary {
                staged.push(path.clone());
                continue;
            }
            return Err(CoreError::InvalidInput("새 파일만 제외할 수 있어요"));
        }
        let lines: Vec<String> = input
            .paths
            .iter()
            .map(|p| rule_line(p))
            .collect::<Result<_>>()?;
        append_rules(repo.as_path(), &lines)?;
        if !staged.is_empty() {
            ctx.git.run(repo, &GitOp::Unstage { paths: staged })?;
        }
        Ok(Outcome::Ignored {
            count: u32::try_from(input.paths.len()).unwrap_or(u32::MAX),
        })
    }
}

/// `/경로` 규칙 한 줄. 무시 규칙에서 뜻이 있는 글자는 `\`로 적는다. 줄바꿈이 든 이름은 받지 않는다.
pub(crate) fn rule_line(path: &str) -> Result<String> {
    if path.contains(['\n', '\r']) {
        return Err(CoreError::InvalidInput(
            "이 이름은 제외할 파일에 넣을 수 없어요",
        ));
    }
    let mut line = String::from("/");
    for c in path.chars() {
        if matches!(c, '\\' | '*' | '?' | '[' | '!' | '#') {
            line.push('\\');
        }
        line.push(c);
    }
    // 끝 공백은 git이 지우므로 하나씩 `\`로 적는다.
    let trailing = line.len() - line.trim_end_matches(' ').len();
    if trailing > 0 {
        line.truncate(line.len() - trailing);
        line.push_str(&"\\ ".repeat(trailing));
    }
    Ok(line)
}

/// `.gitignore` 내용. 없으면 빈 값. 링크이거나 파일이 아니면 읽지도 쓰지도 않는다(D-037).
pub(crate) fn read_rules(root: &Path) -> Result<String> {
    let path = root.join(FILE);
    match std::fs::symlink_metadata(&path) {
        Ok(m) if !m.is_file() => Err(CoreError::InvalidInput("제외할 파일 목록을 바꿀 수 없어요")),
        Ok(_) => std::fs::read_to_string(&path)
            .map_err(|_| CoreError::InvalidInput("제외할 파일 목록을 바꿀 수 없어요")),
        Err(_) => Ok(String::new()),
    }
}

/// 파일의 줄바꿈 방식(CRLF가 있으면 CRLF).
pub(crate) fn newline_of(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

/// 덧붙일 글자: 각 줄 뒤에 그 파일의 줄바꿈.
pub(crate) fn block(lines: &[String], newline: &str) -> String {
    lines.iter().map(|l| format!("{l}{newline}")).collect()
}

fn append_rules(root: &Path, lines: &[String]) -> Result<()> {
    let mut text = read_rules(root)?;
    let newline = newline_of(&text);
    if !text.is_empty() && !text.ends_with('\n') {
        text.push_str(newline);
    }
    text.push_str(&block(lines, newline));
    write_rules(root, &text)
}

/// 임시 파일에 쓰고 바꿔 넣는다(쓰다 끊겨도 반쯤 쓴 `.gitignore`가 남지 않게).
pub(crate) fn write_rules(root: &Path, text: &str) -> Result<()> {
    let fail = |_| CoreError::InvalidInput("제외할 파일 목록을 바꿀 수 없어요");
    let tmp = root.join(".gitignore.higit-tmp");
    std::fs::write(&tmp, text).map_err(fail)?;
    std::fs::rename(&tmp, root.join(FILE)).map_err(fail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_characters_are_escaped() {
        assert_eq!(rule_line("a.txt").unwrap(), "/a.txt");
        assert_eq!(
            rule_line("폴더/비밀 파일.env").unwrap(),
            "/폴더/비밀 파일.env"
        );
        assert_eq!(rule_line("#메모").unwrap(), "/\\#메모");
        assert_eq!(rule_line("!중요").unwrap(), "/\\!중요");
        assert_eq!(rule_line("a*b?[c]\\d").unwrap(), "/a\\*b\\?\\[c]\\\\d");
        assert_eq!(rule_line("끝 공백  ").unwrap(), "/끝 공백\\ \\ ");
        assert!(rule_line("줄\n바꿈").is_err());
    }
}
