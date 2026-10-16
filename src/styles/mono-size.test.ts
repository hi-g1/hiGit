// @vitest-environment node
// 코드 글꼴의 크기는 --font-size-code(본문 코드) 또는 --font-size-sm(보조)만 쓴다. 코드 크기를 한 토큰에서 바꾼다(D-138, D-146).
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

/** 코드 글꼴을 쓰는 규칙: [파일:선택자, 글자 크기 값 또는 null]. */
function monoRules(): [string, string | null][] {
  return cssFiles(SRC).flatMap((file) => {
    const css = readFileSync(file, "utf8");
    return [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)]
      .filter((m) => /font-family:\s*var\(--font-mono\)/.test(m[2]!))
      .map((m): [string, string | null] => [
        `${relative(SRC, file).split(sep).join("/")}:${m[1]!.trim().split("\n").pop()}`,
        /font-size:\s*([^;]+);/.exec(m[2]!)?.[1]?.trim() ?? null,
      ]);
  });
}

// GitHub 연결 코드는 크게 보여야 브라우저에 옮겨 적는다(ConnectPanel).
const EXCEPT = new Set(["features/settings/SettingsDialog.module.css:.code"]);

describe("코드 글꼴 크기", () => {
  it("코드 글꼴 규칙은 --font-size-code나 --font-size-sm만 쓴다", () => {
    const rules = monoRules();
    expect(rules.length).toBeGreaterThan(10);
    const bad = rules.filter(
      ([name, size]) =>
        !EXCEPT.has(name) && size !== null && size !== "var(--font-size-code)" && size !== "var(--font-size-sm)",
    );
    expect(bad).toEqual([]);
  });

  it("비교 보기·충돌 코드·파일 목록은 본문 코드 크기", () => {
    const sizes = Object.fromEntries(monoRules());
    for (const name of [
      "features/diff/DiffView.module.css:.table",
      "features/conflict/HunkPanel.module.css:.code",
      "features/conflict/HunkPanel.module.css:.editor",
      "ui/ListRow.module.css:.mono",
    ]) {
      expect(sizes[name], name).toBe("var(--font-size-code)");
    }
  });
});
