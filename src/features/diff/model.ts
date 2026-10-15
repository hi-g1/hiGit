// 비교 보기 화면 모델. FileDiffView를 그리기용 행으로 바꾼다(순수 함수).
import type { FileDiffView, LineKindView } from "../../lib/api";

export type DiffRow =
  | { type: "hunk"; key: string; section: string }
  | {
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

const SIGN: Record<LineKindView, "+" | "−" | " "> = { added: "+", removed: "−", context: " " };

export function toDiffRows(diff: FileDiffView): DiffRow[] {
  return diff.hunks.flatMap((hunk, h) => [
    { type: "hunk" as const, key: `h${h}`, section: hunk.section },
    ...hunk.lines.map((line, i) => ({
      type: "line" as const,
      key: `h${h}l${i}`,
      kind: line.kind,
      oldLine: line.oldLine,
      newLine: line.newLine,
      sign: SIGN[line.kind],
      text: line.text,
      noNewlineAtEnd: line.noNewlineAtEnd,
    })),
  ]);
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
