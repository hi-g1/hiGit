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

  /**
   * `outline: none`을 쓴 규칙마다 대신하는 포커스 표시가 있다(D-191). 규칙 자체가 `:focus-visible`이면 같은 규칙에
   * 테두리·바탕·그림자가, 아니면 같은 파일에 그 선택자의 `:focus-visible`이나 메뉴 항목의 `[data-highlighted]`가 있다.
   */
  it("outline: none 자리마다 대신하는 표시가 있다", () => {
    /** 코드가 포커스를 주는 칸이라 표시가 필요 없는 곳과 까닭. */
    const EXEMPT: Record<string, string> = {
      "ui/Toast.module.css .viewport": "토스트 묶음 칸, 안의 단추가 각자 포커스를 그린다",
    };
    const visual = /(border[\w-]*|background|box-shadow|outline):/;
    const missing: string[] = [];
    for (const file of cssFiles(SRC)) {
      const css = readFileSync(file, "utf8");
      const name = relative(SRC, file).split(sep).join("/");
      const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => ({
        selector: m[1]!.replace(/\/\*[\s\S]*?\*\//g, "").trim(),
        body: m[2]!,
      }));
      for (const rule of rules.filter((r) => /outline:\s*none/.test(r.body))) {
        for (const sel of rule.selector.split(",").map((s) => s.trim())) {
          if (EXEMPT[`${name} ${sel}`]) continue;
          if (sel.includes(":focus-visible")) {
            if (!visual.test(rule.body.replace(/outline:\s*none;?/, ""))) missing.push(`${name} ${sel}`);
            continue;
          }
          const base = sel.replace(/:focus$/, "");
          const ok = rules.some(
            (r) =>
              r.selector.split(",").some((s) => s.trim() === `${base}:focus-visible` || s.trim() === `${base}[data-highlighted]`) &&
              visual.test(r.body),
          );
          if (!ok) missing.push(`${name} ${sel}`);
        }
      }
    }
    expect(missing).toEqual([]);
  });
});
