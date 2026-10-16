// @vitest-environment node
// 누르는 곳은 24×24px 이상(WCAG 2.5.8, D-191). 끄는 선(Splitter)은 끌기 전용이라 빼고 키보드 ←→가 대신한다.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (path: string) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const rem = (value: string) => Number(/([\d.]+)rem/.exec(value)?.[1] ?? NaN);
const tokens = read("styles/tokens.css");
const token = (name: string) => rem(new RegExp(`${name}:\\s*([^;]+);`).exec(tokens)![1]!);

describe("누르는 곳의 크기(D-191)", () => {
  it("토큰이 24px(1.5rem)이다", () => {
    expect(token("--target-min")).toBe(1.5);
  });

  it("버튼은 줄 높이(2rem)로 그 이상이다", () => {
    expect(read("ui/Button.module.css")).toMatch(/\.button \{[^}]*height: var\(--row-height\);/);
    expect(token("--row-height")).toBeGreaterThanOrEqual(token("--target-min"));
  });

  it("체크 상자는 보이는 16px에 눌림 영역만 넓힌다", () => {
    const css = read("ui/Checkbox.module.css");
    expect(css).toMatch(/\.box \{[^}]*position: relative;[^}]*width: 1rem;/);
    expect(css).toMatch(/\.box::after \{[^}]*inset: calc\(\(1rem - var\(--target-min\)\) \/ 2\);/);
  });

  it("고르는 부품과 줄 메뉴 단추", () => {
    expect(read("ui/Choice.module.css")).toMatch(/\.option \{[^}]*min-height: var\(--target-min\);/);
    const rowMenu = /\.rowMenu \{[^}]*width: ([\d.]+rem);[^}]*height: ([\d.]+rem);/.exec(read("features/branches/BranchList.module.css"))!;
    expect(rem(rowMenu[1]!)).toBeGreaterThanOrEqual(1.5);
    expect(rem(rowMenu[2]!)).toBeGreaterThanOrEqual(1.5);
  });

  it("칸 나눔선은 끌기 대신 키보드로도 된다(예외의 까닭)", () => {
    expect(read("ui/Splitter.tsx")).toMatch(/onKeyDown[\s\S]*paneKey\(/);
  });
});
