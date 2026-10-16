// @vitest-environment node
// 화면 CSS의 길이에 px를 쓰면 린트가 막는지 확인한다. 선·테두리와 미디어 조건은 px 그대로(D-146).
import stylelint from "stylelint";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const ROOT = fileURLToPath(new URL("../", import.meta.url));

async function rules(code, file = "src/features/x.module.css") {
  const { results } = await stylelint.lint({ code, codeFilename: `${ROOT}${file}`, cwd: ROOT });
  return results[0].warnings.filter((w) => w.severity === "error").map((w) => w.rule);
}

describe("CSS 단위 린트", () => {
  it.each([
    ["width", ".a { width: 10px; }"],
    ["min() 안", ".a { width: min(440px, 100vw); }"],
    ["calc() 안", ".a { min-height: calc(1rem + 4px); }"],
    ["padding", ".a { padding: 2px 0; }"],
    ["transform", ".a { transform: translateY(-1px); }"],
    ["grid", ".a { grid-template-columns: 260px 1fr; }"],
    ["font-size", ".a { font-size: 12px; }"],
  ])("%s의 px는 오류", async (_name, code) => {
    expect(await rules(code)).toContain("declaration-property-unit-disallowed-list");
  });

  it.each([
    ["밑줄", ".a { text-decoration: underline; }", "declaration-property-value-disallowed-list"],
    ["밑줄 줄", ".a { text-decoration-line: underline; }", "declaration-property-value-disallowed-list"],
    ["점선 테두리", ".a { border-bottom: 1px dotted var(--text-muted); }", "declaration-property-value-disallowed-list"],
    ["점선 모양", ".a { border-bottom-style: dotted; }", "declaration-property-value-disallowed-list"],
  ])("%s는 오류", async (_name, code, rule) => {
    expect(await rules(code)).toContain(rule);
  });

  it("링크 모양 파일에서만 호버 밑줄을 쓴다", async () => {
    expect(await rules(".link:hover { text-decoration: underline; }", "src/ui/linkStyle.module.css")).toEqual([]);
  });

  it.each([
    ["테두리", ".a { border: 1px solid var(--border); }"],
    ["포커스", ".a {\n  outline: 2px solid var(--accent);\n  outline-offset: 2px;\n}"],
    ["안쪽 막대", ".a { box-shadow: inset 2px 0 0 var(--accent); }"],
    ["미디어 조건", "@media (width <= 900px) { .a { width: 1rem; } }"],
    ["rem", ".a { width: min(27.5rem, 100vw); }"],
  ])("%s는 통과", async (_name, code) => {
    expect(await rules(code)).toEqual([]);
  });
});
