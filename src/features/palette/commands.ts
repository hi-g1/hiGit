// 명령 팔레트의 목록(D-170, D-199). 화면 없이 시험하는 순수 함수다. 화면은 액션 id와 입력만 보내고 core가 다시 검사한다.
import { getTerm, type Locale, type TermId } from "../../glossary";
import type { ActionInput, ActionMetaView, RecentProject } from "../../lib/api";
import { matchQuery } from "../../lib/hangul";
import type { ShortcutId } from "../../lib/shortcuts";
import type { Messages } from "../../messages";

/** 열기 항목이 여는 곳. 연결은 팔레트 화면이 한다. */
export type OpenTarget =
  | "commitBox"
  | "branchList"
  | "clone"
  | "publish"
  | "prCreate"
  | "settings"
  | "forcePush"
  | "openFolder"
  | "shortcuts"
  | "history"
  | "activity"
  /** 흐린 항목(실행하지 않는다). */
  | "none";

type Base = {
  /** 목록 안에서만 쓰는 구분 값. */
  key: string;
  label: string;
  /** 이름 옆 옅은 글자(원래 이름). */
  detail: string;
  /** 찾기에만 쓰는 글자(설명 등). */
  extra: string[];
  shortcut: ShortcutId | null;
  /** 지금 할 수 없으면 까닭(흐린 항목). */
  disabledReason: string | null;
};

export type Command =
  | (Base & { kind: "run"; input: ActionInput })
  | (Base & { kind: "open"; open: OpenTarget })
  | (Base & { kind: "switch"; branch: string; remote: string | null })
  | (Base & { kind: "recent"; path: string });

type Ctx = { locale: Locale; changedFiles: number; hasRemote: boolean; hasUpstream: boolean; merging: boolean; m: Messages };

/** 입력 없이 바로 하는 액션과 그 입력, 할 수 없는 까닭. */
export const RUN_DIRECT: Record<string, { input: (c: Ctx) => ActionInput; reason: (c: Ctx) => string | null; shortcut?: ShortcutId }> = {
  sync: {
    input: (c) => ({ id: "sync", input: { locale: c.locale } }),
    reason: (c) => (c.hasRemote ? null : c.m.palette.noRemote),
    shortcut: "sync",
  },
  fetch: { input: () => ({ id: "fetch", input: { background: false } }), reason: (c) => (c.hasRemote ? null : c.m.palette.noRemote) },
  pull: { input: () => ({ id: "pull" }), reason: (c) => (c.hasUpstream ? null : c.m.palette.noUpstream) },
  push: { input: () => ({ id: "push" }), reason: (c) => (c.hasRemote ? null : c.m.palette.noRemote) },
  stash: { input: () => ({ id: "stash" }), reason: (c) => (c.changedFiles > 0 ? null : c.m.disabledReasons.noChanges) },
};

/** 입력이 필요하거나 확인 창이 있는 안내 흐름을 여는 액션. */
export const OPEN_UI: Record<string, { open: OpenTarget; shortcut?: ShortcutId }> = {
  commit: { open: "commitBox", shortcut: "commit" },
  "branch-create": { open: "branchList", shortcut: "branches" },
  switch: { open: "branchList", shortcut: "branches" },
  clone: { open: "clone", shortcut: "clone" },
  publish: { open: "publish" },
  "pr-create": { open: "prCreate" },
  "identity-set": { open: "settings" },
  // 확인 창이 그 대화상자 안에 있다(D-128).
  "force-push": { open: "forcePush" },
};

/** 확인이 필요해 팔레트에서는 흐리게 보이는 액션과 할 곳(D-199). */
export const DIMMED: Record<string, keyof Messages["palette"]["where"]> = {
  "clear-lock": "notice",
  "pr-merge": "pr",
  "remote-branch-delete": "branches",
  "remote-branch-rename": "branches",
  "remote-tag-delete": "tags",
};

/** 팔레트에 두지 않는 액션과 그 까닭(버전·파일·충돌 곳을 골라야 하거나 그 화면의 흐름 안에서만 뜻이 있다). */
export const EXCLUDED: Record<string, string> = {
  init: "폴더를 열 때 묻는다",
  stage: "파일을 고른다",
  unstage: "파일을 고른다",
  ignore: "파일을 고른다",
  discard: "파일을 고른다",
  "take-out": "보관 항목을 고른다",
  restore: "작업 내역의 버전을 고른다",
  merge: "홈의 합치기 제안에서",
  "merge-into": "브랜치 목록의 제안에서",
  "conflict-finish": "충돌 해결 화면 안에서",
  "conflict-abort": "충돌 해결 화면 안에서",
  "recover-parked": "알림 줄에서",
  amend: "버전 기록의 역을 고른다",
  revert: "버전을 고른다",
  "cherry-pick": "버전을 고른다",
  reset: "버전을 고른다",
  "rebase-i": "버전을 고른다",
  tag: "버전을 고른다",
  "tag-delete": "태그를 고른다",
  "tag-push": "태그를 고른다",
  "branch-delete": "브랜치를 고른다",
  "branch-rename": "브랜치를 고른다",
  "pr-comment": "검토 요청을 고른다",
  "pr-close": "검토 요청을 고른다",
  "pr-reopen": "검토 요청을 고른다",
  fork: "검토 요청 흐름 안에서",
};

export type PaletteInput = {
  actions: ActionMetaView[];
  locale: Locale;
  m: Messages;
  /** 저장소가 열렸는지. 시작 화면이면 폴더 열기·복제·최근 프로젝트·설정·단축키 표만. */
  hasRepo: boolean;
  changedFiles: number;
  hasRemote: boolean;
  hasUpstream: boolean;
  merging: boolean;
  /** 지금 브랜치(바꾸기 항목에서 뺀다). */
  current: string | null;
  branches: { name: string; remote: string | null }[];
  recent: RecentProject[];
  /** 열린 저장소의 경로(최근 프로젝트에서 뺀다). */
  repoPath: string | null;
};

function term(id: TermId, locale: Locale) {
  const t = getTerm(id, locale);
  return { label: t.friendly, detail: t.gitName, extra: [t.description] };
}

/** 지금 상태로 만든 팔레트 목록(찾기 전). 순서: 할 일 → 화면 → 브랜치 → 최근 프로젝트. */
export function buildCommands(p: PaletteInput): Command[] {
  const c: Ctx = { locale: p.locale, changedFiles: p.changedFiles, hasRemote: p.hasRemote, hasUpstream: p.hasUpstream, merging: p.merging, m: p.m };
  const out: Command[] = [];
  const open = (key: string, label: string, target: OpenTarget, shortcut: ShortcutId | null, detail = ""): Command => ({
    key,
    kind: "open",
    label,
    detail,
    extra: [],
    shortcut,
    disabledReason: null,
    open: target,
  });
  if (p.hasRepo) {
    for (const meta of p.actions) {
      const id = meta.id;
      const shown = term(meta.glossaryId as TermId, p.locale);
      const direct = RUN_DIRECT[id];
      const ui = OPEN_UI[id];
      const dimmed = DIMMED[id];
      // 합치는 중에는 충돌 해결 화면이 먼저다(바꾸기·올리기 등을 하지 않는다).
      const busy = p.merging ? p.m.palette.merging : null;
      if (direct) {
        out.push({ key: `run:${id}`, kind: "run", ...shown, shortcut: direct.shortcut ?? null, disabledReason: busy ?? direct.reason(c), input: direct.input(c) });
      } else if (ui) {
        // 같은 곳을 여는 액션은 하나만(브랜치 만들기·바꾸기는 브랜치 목록 하나).
        if (id === "switch") continue;
        out.push({ ...open(`open:${id}`, shown.label, ui.open, ui.shortcut ?? null, shown.detail), extra: shown.extra, disabledReason: busy });
      } else if (dimmed) {
        out.push({
          ...open(`dim:${id}`, shown.label, "none", null, shown.detail),
          extra: shown.extra,
          disabledReason: p.m.palette.where[dimmed],
        });
      }
    }
    out.push(open("view:history", term("log", p.locale).label, "history", "history"));
    out.push(open("view:activity", term("reflog", p.locale).label, "activity", "activity"));
  }
  out.push(open("view:openFolder", p.m.shortcutName.openFolder, "openFolder", "openFolder"));
  if (!p.hasRepo) out.push(open("view:clone", term("clone", p.locale).label, "clone", "clone"));
  out.push(open("view:settings", p.m.settings.title, "settings", null));
  out.push(open("view:shortcuts", p.m.shortcutName.shortcutHelp, "shortcuts", "shortcutHelp"));
  if (p.hasRepo) {
    for (const b of p.branches) {
      if (b.remote === null && b.name === p.current) continue;
      out.push({
        key: `switch:${b.remote ?? ""}/${b.name}`,
        kind: "switch",
        label: b.remote ? `${b.remote}/${b.name}` : b.name,
        detail: p.m.palette.switchTo,
        extra: [],
        shortcut: null,
        disabledReason: p.merging ? p.m.palette.merging : null,
        branch: b.name,
        remote: b.remote,
      });
    }
  }
  for (const r of p.recent) {
    if (r.path === p.repoPath) continue;
    out.push({ key: `recent:${r.path}`, kind: "recent", label: r.name, detail: r.path, extra: [], shortcut: null, disabledReason: null, path: r.path });
  }
  return out;
}

/** 찾기: 점수 높은 것 먼저, 같은 점수면 원래 순서. 빈 질의면 그대로. */
export function searchCommands(commands: Command[], query: string): Command[] {
  return commands
    .map((cmd, i) => ({ cmd, i, score: matchQuery(query, cmd.label, [cmd.detail, ...cmd.extra]) }))
    .filter((x) => x.score > 0)
    .sort((a, b) => b.score - a.score || a.i - b.i)
    .map((x) => x.cmd);
}
