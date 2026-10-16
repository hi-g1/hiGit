import { describe, expect, it } from "vitest";
import type { PrRowView } from "../../lib/api";
import { ko } from "../../messages/ko";
import { branchLine, stateLabel } from "./model";

const row = (over: Partial<PrRowView>): PrRowView => ({
  number: 1,
  title: "제목",
  state: "open",
  draft: false,
  head: "feature",
  headOwner: null,
  base: "main",
  author: "mona",
  updatedAt: 0,
  isCurrentBranch: false,
  ...over,
});

describe("pr model", () => {
  it("상태 글자", () => {
    expect(stateLabel(row({}), ko)).toBe("열림");
    expect(stateLabel(row({ draft: true }), ko)).toBeNull();
    expect(stateLabel(row({ state: "merged", draft: true }), ko)).toBe("합쳐짐");
    expect(stateLabel(row({ state: "closed" }), ko)).toBe("닫힘");
  });

  it("브랜치 줄", () => {
    expect(branchLine(row({}))).toBe("feature → main");
    expect(branchLine(row({ headOwner: "forker" }))).toBe("forker:feature → main");
  });
});
