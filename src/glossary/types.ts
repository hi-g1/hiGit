// 용어 id 목록. docs/spec.md 용어표의 id 열과 같은 순서다.
export const TERM_IDS = [
  "status",
  "diff",
  "stage",
  "unstage",
  "commit",
  "fetch",
  "pull",
  "push",
  "sync",
  "branch",
  "branch-delete",
  "branch-rename",
  "remote-branch-delete",
  "remote-branch-rename",
  "switch",
  "log",
  "merge",
  "conflict",
  "ours",
  "theirs",
  "base",
  "stash",
  "park",
  "discard",
  "pr",
  "pr-draft",
  "pr-merge",
  "pr-close",
  "checks",
  "review",
  "comment",
  "clone",
  "fork",
  "publish",
  "init",
  "identity",
  "amend",
  "revert",
  "reset",
  "rebase-i",
  "rebase",
  "cherry-pick",
  "tag",
  "tag-push",
  "remote-tag-delete",
  "blame",
  "reflog",
  "force-push",
  "remote",
  "ignore",
  "lock",
] as const;

export type TermId = (typeof TERM_IDS)[number];

export type TermEntry = {
  /** 화면 본문에 보이는 쉬운 이름 */
  friendly: string;
  /** 원래 Git 이름 */
  gitName: string;
  /** 대표 명령. 명령이 없는 용어(merge conflict 등)는 null */
  command: string | null;
  /** 툴팁의 한 줄 설명 */
  description: string;
};

export type Glossary = Record<TermId, TermEntry>;

export type Locale = "ko" | "en";
