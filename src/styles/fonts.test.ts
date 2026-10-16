// @vitest-environment node
// 코드 글꼴(D-173): 순서, 한글 범위, 크기 상한, 라이선스 원문.
import { readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = join(__dirname, "..", "..");
const read = (p: string) => readFileSync(join(root, p), "utf8");

describe("코드 글꼴 D2Coding(D-173)", () => {
  it("--font-mono는 JetBrains Mono → D2Coding → Pretendard 순서", () => {
    const tokens = read("src/styles/tokens.css");
    expect(tokens).toMatch(/--font-mono: "JetBrains Mono", "D2Coding", "Pretendard", ui-monospace, monospace;/);
  });

  it("공식 woff2 하나를 한글 범위에만, 라틴 두 자 폭으로", () => {
    const css = read("src/styles/fonts.css");
    const face = css.slice(css.indexOf(`font-family: "D2Coding"`));
    const block = face.slice(0, face.indexOf("}"));
    expect(block).toContain('url("../assets/fonts/D2Coding-Regular.woff2") format("woff2")');
    expect(block).toContain("font-weight: 400");
    expect(block).toContain("unicode-range: U+AC00-D7A3, U+1100-11FF, U+3130-318F");
    // JetBrains Mono 라틴 폭 0.6em × 2 = 1.2em = D2Coding 한글 1em × 120%
    expect(block).toContain("size-adjust: 120%");
    const font = readFileSync(join(root, "src/assets/fonts/D2Coding-Regular.woff2"));
    expect(font.subarray(0, 4).toString("latin1")).toBe("wOF2");
    expect(statSync(join(root, "src/assets/fonts/D2Coding-Regular.woff2")).size).toBeLessThan(2 * 1024 * 1024);
  });

  it("라이선스 원문이 글꼴 옆과 서드파티 고지에 있다", () => {
    const ofl = read("src/assets/fonts/D2Coding-OFL.txt");
    expect(ofl).toContain("with Reserved Font Name D2Coding.");
    expect(ofl).toContain("SIL Open Font License, Version 1.1");
    const notices = read("THIRD_PARTY_NOTICES.md");
    expect(notices).toContain("| D2Coding |");
    expect(notices).toContain(ofl.split("\n")[0]!);
  });
});
