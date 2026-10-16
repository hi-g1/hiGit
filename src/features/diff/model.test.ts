import { describe, expect, it } from "vitest";
import type { FileDiffView } from "../../lib/api";
import { countChanges, toDiffRows } from "./model";

const diff: FileDiffView = {
  binary: false,
  truncated: false,
  link: null,
  hunks: [
    {
      section: "함수",
      oldStart: 1,
      oldLen: 3,
      newStart: 1,
      newLen: 3,
      lines: [
        { kind: "context", text: "가", oldLine: 1, newLine: 1, noNewlineAtEnd: false },
        { kind: "removed", text: "나", oldLine: 2, newLine: null, noNewlineAtEnd: false },
        { kind: "added", text: "나나", oldLine: null, newLine: 2, noNewlineAtEnd: false },
        { kind: "added", text: "다", oldLine: null, newLine: 3, noNewlineAtEnd: true },
      ],
    },
  ],
};

describe("비교 보기 모델", () => {
  it("hunk 머리와 줄을 그리기용 행으로 바꾼다", () => {
    const rows = toDiffRows(diff);
    expect(rows[0]).toEqual({ type: "hunk", key: "h0", section: "함수" });
    expect(rows.slice(1).map((r) => (r.type === "line" ? [r.sign, r.text, r.oldLine, r.newLine] : null))).toEqual([
      [" ", "가", 1, 1],
      ["−", "나", 2, null],
      ["+", "나나", null, 2],
      ["+", "다", null, 3],
    ]);
  });

  it("추가·삭제 줄 수를 센다", () => {
    expect(countChanges(diff)).toEqual({ added: 2, removed: 1 });
    expect(countChanges({ binary: true, truncated: false, hunks: [], link: null })).toEqual({ added: 0, removed: 0 });
  });
});
