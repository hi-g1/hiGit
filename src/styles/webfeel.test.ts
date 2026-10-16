// @vitest-environment node
// 데스크톱 프로그램처럼 굴게 하는 CSS(D-171, D-189): 손가락 커서는 링크 모양에만, 끌림·튕김·한국어 줄바꿈 규칙.
import { readFileSync, readdirSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../", import.meta.url));
const base = readFileSync(new URL("./base.css", import.meta.url), "utf8");

function cssFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const path = join(dir, e.name);
    if (e.isDirectory()) return cssFiles(path);
    return e.name.endsWith(".css") ? [path] : [];
  });
}

describe("웹 느낌 끄기(D-189)", () => {
  it("손가락 커서는 링크 모양 파일에만 있다", () => {
    const users = cssFiles(SRC)
      .filter((file) => /cursor:\s*pointer/.test(readFileSync(file, "utf8")))
      .map((file) => relative(SRC, file).split(sep).join("/"));
    expect(users).toEqual(["ui/linkStyle.module.css"]);
  });

  it("버튼·목록 줄은 화살표 커서이고 글자가 끌리지 않으며, 선택할 곳은 따로 연다", () => {
    expect(base).toMatch(/:where\(button,[^)]*\[role="option"\][^)]*summary, label, kbd\)\s*\{\s*cursor: default;\s*user-select: none;/);
    expect(base).toMatch(/:where\(\[data-selectable\], pre, input, textarea\)\s*\{\s*user-select: text;/);
  });

  it("튕김·이미지 끌기·한국어 줄바꿈·같은 폭 숫자", () => {
    expect(base).toMatch(/html,\s*body\s*\{\s*overscroll-behavior: none;/);
    expect(base).toMatch(/img,\s*svg,\s*a\s*\{\s*-webkit-user-drag: none;/);
    expect(base).toMatch(/word-break: keep-all;/);
    expect(base).toMatch(/font-variant-numeric: tabular-nums;/);
    expect(base).toMatch(/\[data-selectable\]\s*\{\s*word-break: normal;\s*overflow-wrap: anywhere;/);
  });
});
