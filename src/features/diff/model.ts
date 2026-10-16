// 비교 보기 화면 모델. FileDiffView를 그리기용 행으로 바꾼다(순수 함수).
import type { DiffLineView, FileDiffView, HunkView, LineKindView } from "../../lib/api";

export type LineRow = {
  type: "line";
  key: string;
  kind: LineKindView;
  oldLine: number | null;
  newLine: number | null;
  /** 색만으로 알리지 않도록 기호를 함께 쓴다(design.md). */
  sign: "+" | "−" | " ";
  text: string;
  noNewlineAtEnd: boolean;
};

/** 묶음 사이에 생략된 변경 없는 줄(D-156). `from`~`to`는 마지막 저장 쪽 줄 번호. */
export type FoldRow = {
  type: "fold";
  key: string;
  count: number;
  from: number;
  to: number;
  /** 다음 묶음 머리의 함수 이름. */
  section: string;
  /** 펼치려 했지만 받은 전체 문맥에 그 줄이 없을 때(비교 상한). */
  unavailable: boolean;
};

export type DiffRow = LineRow | FoldRow;

const SIGN: Record<LineKindView, "+" | "−" | " "> = { added: "+", removed: "−", context: " " };

function lineRow(key: string, line: DiffLineView): LineRow {
  return {
    type: "line",
    key,
    kind: line.kind,
    oldLine: line.oldLine,
    newLine: line.newLine,
    sign: SIGN[line.kind],
    text: line.text,
    noNewlineAtEnd: line.noNewlineAtEnd,
  };
}

/** 묶음이 덮는 마지막 저장 쪽 첫 줄과, 그 뒤 변경 없는 첫 줄(길이 0이면 시작이 앞 줄이다). */
const firstCovered = (h: HunkView) => (h.oldLen === 0 ? h.oldStart + 1 : h.oldStart);
const nextUnchanged = (h: HunkView) => (h.oldLen === 0 ? h.oldStart + 1 : h.oldStart + h.oldLen);

/** 묶음이 덮는 지금 쪽 마지막 줄(길이 0이면 시작 줄 뒤에 끼운다). */
const lastNewCovered = (h: HunkView) => (h.newLen === 0 ? h.newStart : h.newStart + h.newLen - 1);

/**
 * 행으로 바꾼다. 첫 묶음 앞, 묶음 사이, 마지막 묶음 뒤(파일 줄 수 `newLineCount`를 알 때, D-219)의 생략은 접힌 줄
 * 하나("변경 없는 N줄 · 펼치기"). `expanded`에 든 접힌 줄은 `full`(전체 문맥 비교)에서 그 구간의 줄로 바꾼다.
 */
export function toDiffRows(diff: FileDiffView, full?: FileDiffView, expanded: ReadonlySet<string> = new Set()): DiffRow[] {
  const fullLines = full ? full.hunks.flatMap((h) => h.lines) : [];
  const rows: DiffRow[] = [];
  const fold = (key: string, from: number, to: number, section: string) => {
    const count = to - from + 1;
    if (count <= 0) return;
    const lines = expanded.has(key) && full ? fullLines.filter((l) => l.oldLine !== null && l.oldLine >= from && l.oldLine <= to) : [];
    if (lines.length === count) {
      lines.forEach((line, i) => rows.push(lineRow(`${key}l${i}`, line)));
    } else {
      rows.push({ type: "fold", key, count, from, to, section, unavailable: expanded.has(key) && !!full });
    }
  };
  diff.hunks.forEach((hunk, h) => {
    fold(`f${h}`, h === 0 ? 1 : nextUnchanged(diff.hunks[h - 1]!), firstCovered(hunk) - 1, hunk.section);
    hunk.lines.forEach((line, i) => rows.push(lineRow(`h${h}l${i}`, line)));
  });
  // 마지막 묶음 뒤의 변경 없는 줄은 양쪽이 같아 지금 쪽 줄 수로 센다. 접힌 줄의 번호는 마지막 저장 쪽.
  const last = diff.hunks.at(-1);
  if (last && diff.newLineCount !== null && !diff.truncated) {
    const count = diff.newLineCount - lastNewCovered(last);
    const from = nextUnchanged(last);
    fold("fEnd", from, from + count - 1, "");
  }
  return rows;
}

export function countChanges(diff: FileDiffView): { added: number; removed: number } {
  let added = 0;
  let removed = 0;
  for (const hunk of diff.hunks) {
    for (const line of hunk.lines) {
      if (line.kind === "added") added += 1;
      else if (line.kind === "removed") removed += 1;
    }
  }
  return { added, removed };
}
