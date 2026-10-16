// @vitest-environment node
// 스크롤 막대는 base.css의 전역 규칙 하나(C14). 비교 칸·목록의 overflow: auto가 모두 같은 얇은 막대를 받도록
// 부품 CSS는 막대 모양을 따로 정하지 않고, 늘 보이는 overflow: scroll도 쓰지 않는다.
import { readFileSync, readdirSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../", import.meta.url));

function cssFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const path = join(dir, e.name);
    if (e.isDirectory()) return cssFiles(path);
    return e.name.endsWith(".module.css") ? [path] : [];
  });
}

const name = (f: string) => relative(SRC, f).split(sep).join("/");

describe("스크롤 막대", () => {
  it("전역 규칙이 토큰으로 얇게 그린다", () => {
    const base = readFileSync(join(SRC, "styles/base.css"), "utf8");
    expect(base).toMatch(/scrollbar-width: thin;/);
    expect(base).toMatch(/scrollbar-color: var\(--scrollbar-thumb\) transparent;/);
    expect(base).toMatch(/::-webkit-scrollbar \{\s*width: var\(--scrollbar-size\);\s*height: var\(--scrollbar-size\);/);
  });

  it("라이트·시스템 다크·고른 다크 모두 막대 색이 있다", () => {
    const tokens = readFileSync(join(SRC, "styles/tokens.css"), "utf8");
    expect(tokens.match(/--scrollbar-thumb:/g)).toHaveLength(3);
  });

  it("부품 CSS는 막대 모양을 따로 정하지 않고 overflow: scroll을 쓰지 않는다", () => {
    const found = cssFiles(SRC).flatMap((f) => {
      const css = readFileSync(f, "utf8");
      return /scrollbar|overflow(-[xy])?:\s*scroll/.test(css) ? [name(f)] : [];
    });
    expect(found).toEqual([]);
  });
});
