// @vitest-environment node
// 로컬 브랜치 줄은 줄 메뉴와 나란히 놓이는 flex 묶음이다. 묶음 제목이 그 옆으로 끼지 않고 줄 위에 오는지 본다.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./BranchList.module.css", import.meta.url), "utf8");

describe("브랜치 목록 CSS", () => {
  it("묶음 제목은 줄 위에 한 줄로", () => {
    expect(css).toMatch(/\.rowWrap \{[^}]*flex-wrap: wrap/);
    expect(css).toMatch(/\.rowWrap > \.section \{[^}]*flex: 1 0 100%/);
  });
});
