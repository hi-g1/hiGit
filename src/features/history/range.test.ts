import { describe, expect, it } from "vitest";
import { commitNode } from "./model";
import { eligible, extend, headLine, MAX_RANGE, rangeIds } from "./range";

// 화면 순서: h(지금) s(곁가지) m(합침: a·s) a b(첫 버전).
const nodes = [
  commitNode("h", ["m"], { isHead: true }),
  commitNode("s", ["b"]),
  commitNode("m", ["a", "s"]),
  commitNode("a", ["b"]),
  commitNode("b", []),
];

describe("여러 역 선택", () => {
  it("지금 브랜치 선은 첫 부모만 따라간다", () => {
    expect(headLine(nodes)).toEqual(["h", "m", "a", "b"]);
    expect(headLine([commitNode("x", [])])).toEqual([]);
  });

  it("범위 끝은 선 위의 이웃으로(곁가지는 건너뜀), 끝에서 멈춤", () => {
    const line = headLine(nodes);
    expect(extend(line, "h", "next")).toBe("m");
    expect(extend(line, "m", "prev")).toBe("h");
    expect(extend(line, "b", "next")).toBe("b");
    expect(extend(line, "h", "prev")).toBe("h");
    expect(extend(line, "s", "next")).toBe("s");
  });

  it("두 끝 사이(새 것부터)", () => {
    const line = headLine(nodes);
    expect(rangeIds(line, "a", "h")).toEqual(["h", "m", "a"]);
    expect(rangeIds(line, "h", "h")).toEqual(["h"]);
    expect(rangeIds(line, "s", "h")).toEqual([]);
  });

  it("정리할 수 있는 구간", () => {
    const linear = [commitNode("c", ["b"], { isHead: true }), commitNode("b", ["a"]), commitNode("a", [])];
    expect(eligible(["c", "b"], linear, "all")).toBe(true);
    expect(eligible(["c"], linear, "all")).toBe(false);
    expect(eligible(["c", "b"], linear, "key")).toBe(false);
    expect(eligible(["h", "m"], nodes, "all")).toBe(false);
    expect(eligible(["a", "b"], nodes, "all")).toBe(false);
    const folded = [{ ...linear[0]! }, { ...linear[1]!, kind: "fold" as const }];
    expect(eligible(["c", "b"], folded, "all")).toBe(false);
    const many = Array.from({ length: MAX_RANGE + 1 }, (_, i) => commitNode(`n${i}`, [`n${i + 1}`]));
    expect(eligible(many.slice(0, MAX_RANGE).map((n) => n.id), many, "all")).toBe(true);
    expect(eligible(many.map((n) => n.id), many, "all")).toBe(false);
  });
});
