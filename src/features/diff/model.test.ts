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
  it("줄을 그리기용 행으로 바꾸고, 1번 줄부터인 묶음 앞에는 접힌 줄이 없다", () => {
    const rows = toDiffRows(diff);
    expect(rows.map((r) => (r.type === "line" ? [r.sign, r.text, r.oldLine, r.newLine] : null))).toEqual([
      [" ", "가", 1, 1],
      ["−", "나", 2, null],
      ["+", "나나", null, 2],
      ["+", "다", null, 3],
    ]);
  });

  const ctx = (oldLine: number, newLine: number) => ({ kind: "context" as const, text: `줄 ${oldLine}`, oldLine, newLine, noNewlineAtEnd: false });
  const hunk = (oldStart: number, oldLen: number, newStart: number, newLen: number, section = "") => ({
    section,
    oldStart,
    oldLen,
    newStart,
    newLen,
    lines: Array.from({ length: oldLen }, (_, i) => ctx(oldStart + i, newStart + i)),
  });
  const spaced: FileDiffView = {
    binary: false,
    truncated: false,
    link: null,
    // 13~15줄, 24~26줄, 바로 붙은 27~29줄, 지운 줄만 있는 묶음(-40,2 +39,0)
    hunks: [hunk(13, 3, 13, 3, "fn a()"), hunk(24, 3, 24, 3), hunk(27, 3, 27, 3), { ...hunk(40, 2, 39, 0), lines: [] }],
  };

  it("첫 묶음 앞과 묶음 사이의 생략 줄 수를 묶음 범위로 센다(D-156)", () => {
    const folds = toDiffRows(spaced).filter((r) => r.type === "fold");
    expect(folds.map((f) => (f.type === "fold" ? [f.count, f.from, f.to, f.section] : null))).toEqual([
      [12, 1, 12, "fn a()"],
      [8, 16, 23, ""],
      [10, 30, 39, ""],
    ]);
  });

  it("펼친 접힌 줄은 전체 문맥의 그 구간 줄로 바뀐다", () => {
    const full: FileDiffView = { ...spaced, hunks: [hunk(1, 41, 1, 41)] };
    const rows = toDiffRows(spaced, full, new Set(["f1"]));
    const lines = rows.filter((r) => r.type === "line" && r.key.startsWith("f1"));
    expect(lines.map((r) => (r.type === "line" ? r.oldLine : null))).toEqual([16, 17, 18, 19, 20, 21, 22, 23]);
    expect(rows.some((r) => r.type === "fold" && r.key === "f1")).toBe(false);
    expect(rows.some((r) => r.type === "fold" && r.key === "f0")).toBe(true);
  });

  it("전체 문맥이 잘려 그 구간이 없으면 펼치지 못함으로 남는다", () => {
    const cut: FileDiffView = { ...spaced, truncated: true, hunks: [hunk(1, 20, 1, 20)] };
    const rows = toDiffRows(spaced, cut, new Set(["f0", "f1"]));
    expect(rows.filter((r) => r.type === "line" && r.key.startsWith("f0"))).toHaveLength(12);
    const f1 = rows.find((r) => r.type === "fold" && r.key === "f1");
    expect(f1 && f1.type === "fold" && f1.unavailable).toBe(true);
  });

  it("추가·삭제 줄 수를 센다", () => {
    expect(countChanges(diff)).toEqual({ added: 2, removed: 1 });
    expect(countChanges({ binary: true, truncated: false, hunks: [], link: null })).toEqual({ added: 0, removed: 0 });
  });
});
