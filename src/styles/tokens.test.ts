// @vitest-environment node
// 크기 토큰이 rem이라 루트 글자 크기 하나로 화면 전체가 커지는지, 가장 작은 글자가 13px인지 확인한다(D-138, D-146).
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");
const root = css.slice(css.indexOf(":root {"), css.indexOf("}", css.indexOf(":root {")));

function tokens(pattern: RegExp): [string, string][] {
  return [...root.matchAll(/(--[\w-]+):\s*([^;]+);/g)]
    .filter((m) => pattern.test(m[1]!))
    .map((m) => [m[1]!, m[2]!.trim()]);
}

describe("크기 토큰", () => {
  it("글자 크기는 모두 rem이고 D-138 값이다", () => {
    expect(Object.fromEntries(tokens(/^--font-size-/))).toEqual({
      "--font-size-sm": "0.8125rem",
      "--font-size-md": "0.875rem",
      "--font-size-code": "0.875rem",
      "--font-size-title": "1.0625rem",
      "--font-size-name": "1.375rem",
      "--font-size-display": "1.75rem",
    });
  });

  it("간격·모서리·행 높이·스크롤 막대·움직임 거리는 rem", () => {
    const sizes = tokens(/^--(space-|radius-|row-height|scrollbar-size$|motion-shift$)/);
    expect(sizes.length).toBeGreaterThanOrEqual(12);
    for (const [name, value] of sizes) expect(`${name}: ${value}`).toMatch(/: [\d.]+rem$/);
  });

  it("px는 선·그림자 토큰에만 남는다", () => {
    const px = tokens(/^--/).filter(([, v]) => /\dpx/.test(v)).map(([n]) => n);
    expect(px.sort()).toEqual(["--border-width", "--focus-offset", "--focus-offset-inset", "--focus-width", "--selected-bar", "--shadow-float"]);
  });
});
