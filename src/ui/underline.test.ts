// @vitest-environment node
// 밑줄과 점선은 링크 모양 파일에서만 쓴다(D-140, D-154). 린트가 막지만 화면 CSS 전체를 한 번 더 본다.
import { readFileSync, readdirSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../", import.meta.url));

function cssFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const path = join(dir, e.name);
    if (e.isDirectory()) return cssFiles(path);
    return e.name.endsWith(".css") ? [path] : [];
  });
}

describe("밑줄·점선", () => {
  it("underline·dotted는 linkStyle.module.css에만 있다", () => {
    const found = cssFiles(SRC)
      .filter((f) => /underline|dotted/.test(readFileSync(f, "utf8").replace(/\/\*[\s\S]*?\*\//g, "")))
      .map((f) => relative(SRC, f).split(sep).join("/"));
    expect(found).toEqual(["ui/linkStyle.module.css"]);
  });
});
