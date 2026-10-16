// @vitest-environment node
// jsdom은 레이아웃을 재지 못한다. 상단 바가 넘치지 않게 하는 규칙이 CSS에 있는지만 확인하고, 800px·"아주 크게"의
// 실제 모양은 Ubuntu 스크린샷과 손 확인으로 본다(D-138, 계획 011 R5).
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = (rel: string) => readFileSync(new URL(rel, import.meta.url), "utf8");
const block = (text: string, selector: string) => {
  const start = text.indexOf(`${selector} {`);
  expect(start, selector).toBeGreaterThanOrEqual(0);
  return text.slice(start, text.indexOf("}", start));
};

describe("상단 바 CSS", () => {
  it("상단 바와 오른쪽 묶음은 줄을 바꾼다", () => {
    const top = css("./TopBar.module.css");
    expect(block(top, ".bar")).toContain("flex-wrap: wrap");
    expect(block(top, ".right")).toContain("flex-wrap: wrap");
    expect(block(top, ".right")).toContain("margin-left: auto");
  });

  it("브랜치 단추는 프로젝트 이름처럼 테두리 없이 호버 때 바탕", () => {
    const branch = css("../../features/branches/BranchList.module.css");
    const trigger = block(branch, ".trigger");
    expect(trigger).toContain("border: none");
    expect(trigger).toContain("background: none");
    expect(branch).toMatch(/\.trigger:hover,\s*\.trigger\[data-state="open"\] \{\s*background: var\(--surface-hover\)/);
    const project = css("./ProjectMenu.module.css");
    expect(block(project, ".trigger")).toContain("border: none");
  });
});
