// 충돌 해결 화면의 계산(D-028, D-086). DOM 없이 시험한다.
import type { ConflictHunkView, FileResolution } from "../../lib/api";

export type HunkPick = { kind: "ours" } | { kind: "theirs" } | { kind: "both" } | { kind: "custom"; text: string };

/** 파일 하나의 선택. 텍스트는 곳마다, 그 밖(`whole`)은 한쪽 그대로. */
export type FilePick =
  | { kind: "hunks"; picks: (HunkPick | null)[] }
  | { kind: "ours" }
  | { kind: "theirs" };

export type Picks = Record<string, FilePick | undefined>;

/** 화면이 아는 충돌 파일 요약(조회 결과의 일부). */
export type ConflictFile = { path: string; kind: "text" | "whole"; hunks: number };

export type Place = { path: string; hunk: number };

function remainingIn(file: ConflictFile, pick: FilePick | undefined): number {
  if (file.kind === "whole") return pick && pick.kind !== "hunks" ? 0 : 1;
  if (!pick || pick.kind !== "hunks") return file.hunks;
  let left = 0;
  for (let i = 0; i < file.hunks; i++) if (!pick.picks[i]) left++;
  return left;
}

/** 전체 곳 수, 남은 곳 수, 파일별 남은 곳 수. */
export function progress(files: ConflictFile[], picks: Picks) {
  let total = 0;
  let remaining = 0;
  const byFile: Record<string, number> = {};
  for (const f of files) {
    const left = remainingIn(f, picks[f.path]);
    total += f.kind === "whole" ? 1 : f.hunks;
    remaining += left;
    byFile[f.path] = left;
  }
  return { total, remaining, byFile };
}

/** 모두 골랐는지. 충돌 파일이 없으면(저장만 남음) 참. */
export function canFinish(files: ConflictFile[], picks: Picks): boolean {
  return progress(files, picks).remaining === 0;
}

/** 텍스트 파일의 곳 하나에 고른 것을 넣는다. */
export function pickHunk(picks: Picks, file: ConflictFile, hunk: number, pick: HunkPick): Picks {
  const current = picks[file.path];
  const list =
    current && current.kind === "hunks"
      ? [...current.picks]
      : Array.from<HunkPick | null>({ length: file.hunks }).fill(null);
  list[hunk] = pick;
  return { ...picks, [file.path]: { kind: "hunks", picks: list } };
}

/** 고른 곳 하나. */
export function hunkPick(picks: Picks, path: string, hunk: number): HunkPick | null {
  const p = picks[path];
  return p && p.kind === "hunks" ? (p.picks[hunk] ?? null) : null;
}

export type PreviewLine = { text: string; from: "context" | "ours" | "theirs" | "custom" };

/** "저장될 모습": 앞뒤 맥락과 고른 내용. 줄마다 어느 쪽 것인지 함께. */
export function savedPreview(hunk: ConflictHunkView, pick: HunkPick | null): PreviewLine[] {
  const context = (lines: string[]) => lines.map((text) => ({ text, from: "context" as const }));
  let chosen: PreviewLine[] = [];
  if (pick?.kind === "ours") chosen = hunk.ours.map((text) => ({ text, from: "ours" }));
  if (pick?.kind === "theirs") chosen = hunk.theirs.map((text) => ({ text, from: "theirs" }));
  if (pick?.kind === "both")
    chosen = [
      ...hunk.ours.map((text) => ({ text, from: "ours" as const })),
      ...hunk.theirs.map((text) => ({ text, from: "theirs" as const })),
    ];
  if (pick?.kind === "custom")
    chosen = pick.text === "" ? [] : pick.text.split("\n").map((text) => ({ text, from: "custom" }));
  return [...context(hunk.before), ...chosen, ...context(hunk.after)];
}

/** 직접 고치기 글상자를 시작할 글: 고른 내용, 아직 안 골랐으면 내 쪽. */
export function customStart(hunk: ConflictHunkView, pick: HunkPick | null): string {
  if (pick?.kind === "custom") return pick.text;
  if (pick?.kind === "theirs") return hunk.theirs.join("\n");
  if (pick?.kind === "both") return [...hunk.ours, ...hunk.theirs].join("\n");
  return hunk.ours.join("\n");
}

/** `from` 다음의 아직 안 고른 곳. 끝까지 없으면 처음부터 찾는다. 다 골랐으면 `null`. */
export function nextUnresolved(files: ConflictFile[], picks: Picks, from: Place | null): Place | null {
  const places: Place[] = [];
  for (const f of files) {
    const count = f.kind === "whole" ? 1 : f.hunks;
    for (let h = 0; h < count; h++) {
      const done =
        f.kind === "whole"
          ? remainingIn(f, picks[f.path]) === 0
          : hunkPick(picks, f.path, h) !== null;
      if (!done) places.push({ path: f.path, hunk: h });
    }
  }
  if (places.length === 0) return null;
  if (!from) return places[0] ?? null;
  const order = (p: Place) => {
    const i = files.findIndex((f) => f.path === p.path);
    return i * 100000 + p.hunk;
  };
  const after = places.find((p) => order(p) > order(from));
  return after ?? places[0] ?? null;
}

/** 모든 곳(이동용): 이전·다음. */
export function step(files: ConflictFile[], from: Place, delta: 1 | -1): Place {
  const places: Place[] = files.flatMap((f) =>
    Array.from({ length: f.kind === "whole" ? 1 : f.hunks }, (_, hunk) => ({ path: f.path, hunk })),
  );
  const i = places.findIndex((p) => p.path === from.path && p.hunk === from.hunk);
  const next = places[Math.min(Math.max(i + delta, 0), places.length - 1)];
  return next ?? from;
}

/** core에 보낼 입력. 모두 골랐을 때만 부른다. */
export function toResolutions(files: ConflictFile[], picks: Picks): FileResolution[] {
  return files.map((f) => {
    const p = picks[f.path];
    if (f.kind === "whole" || !p || p.kind !== "hunks") {
      return { path: f.path, choice: { kind: p?.kind === "theirs" ? "theirs" : "ours" } };
    }
    return {
      path: f.path,
      choice: {
        kind: "hunks",
        choices: p.picks.map((h) =>
          h?.kind === "custom" ? { kind: "custom", text: h.text } : { kind: h?.kind ?? "ours" },
        ),
      },
    };
  });
}
