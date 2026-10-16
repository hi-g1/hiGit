// @vitest-environment node
// 포커스 테두리는 base.css의 전역 :focus-visible 한 곳(D-159). 부품은 같은 선을 다시 그리지 않고, 안쪽으로 그릴 때만
// --focus-offset-inset을 준다. 예외는 --primary 바탕 위 토스트 단추(--primary-text 선).
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

describe("포커스 테두리", () => {
  it("전역 규칙이 토큰으로 그린다", () => {
    const base = readFileSync(join(SRC, "styles/base.css"), "utf8");
    expect(base).toMatch(/:focus-visible \{\s*outline: var\(--focus-width\) solid var\(--accent\);\s*outline-offset: var\(--focus-offset\);/);
  });

  it("부품 CSS는 --accent 포커스 선을 따로 그리지 않는다", () => {
    const found = cssFiles(SRC).flatMap((f) =>
      /outline:\s*(2px|var\(--focus-width\))\s+solid\s+var\(--accent\)/.test(readFileSync(f, "utf8"))
        ? [relative(SRC, f).split(sep).join("/")]
        : [],
    );
    expect(found).toEqual([]);
  });
});
