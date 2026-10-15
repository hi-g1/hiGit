// @vitest-environment node
// tokens.css의 색 대비가 WCAG AA를 넘는지 확인한다(D-026). 글자 4.5:1, 테두리·포커스·버튼 경계 3:1.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");

/** `selector {` 다음의 중괄호 블록 안 색 변수들. */
function colors(selector: string): Record<string, string> {
  const start = css.indexOf(selector);
  if (start < 0) throw new Error(`tokens.css에 ${selector}가 없습니다`);
  const open = css.indexOf("{", start);
  const close = css.indexOf("}", open);
  const block = css.slice(open + 1, close);
  const out: Record<string, string> = {};
  for (const m of block.matchAll(/(--[\w-]+):\s*(#[0-9a-fA-F]{3,6})\s*;/g)) {
    out[m[1]!] = m[2]!;
  }
  return out;
}

function luminance(hex: string): number {
  let h = hex.slice(1);
  if (h.length === 3) h = [...h].map((c) => c + c).join("");
  const [r, g, b] = [0, 2, 4].map((i) => {
    const c = parseInt(h.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r! + 0.7152 * g! + 0.0722 * b!;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi! + 0.05) / (lo! + 0.05);
}

/** [글자 또는 앞쪽, 바탕, 기준] */
const PAIRS: [string, string, number][] = [
  ...["--bg", "--surface", "--surface-sunken", "--surface-hover"].flatMap(
    (bg): [string, string, number][] => [
      ["--text", bg, 4.5],
      ["--text-muted", bg, 4.5],
    ],
  ),
  ...["--bg", "--surface", "--surface-hover"].flatMap((bg): [string, string, number][] => [
    ["--added", bg, 4.5],
    ["--modified", bg, 4.5],
    ["--deleted", bg, 4.5],
  ]),
  ["--accent", "--bg", 4.5],
  ["--accent", "--surface", 4.5],
  ["--accent", "--surface-sunken", 4.5],
  ["--primary-text", "--primary", 4.5],
  ["--primary-text", "--primary-hover", 4.5],
  ["--added", "--diff-add-bg", 4.5],
  ["--deleted", "--diff-del-bg", 4.5],
  ["--text", "--diff-add-bg", 4.5],
  ["--text", "--diff-del-bg", 4.5],
  // 경계: 주 버튼 모양, 입력칸·체크 상자 테두리
  ["--primary", "--bg", 3],
  ["--primary", "--surface", 3],
  ["--border-strong", "--surface", 3],
  ["--border-strong", "--bg", 3],
];

const themes = {
  라이트: colors(":root {"),
  다크: colors(':root[data-theme="dark"] {'),
};

describe("색 대비", () => {
  for (const [name, theme] of Object.entries(themes)) {
    it(`${name}: 모든 쌍이 기준을 넘는다`, () => {
      for (const [fg, bg, min] of PAIRS) {
        expect(theme[fg], `${name} ${fg}`).toBeDefined();
        expect(theme[bg], `${name} ${bg}`).toBeDefined();
        const ratio = contrast(theme[fg]!, theme[bg]!);
        expect(ratio, `${name} ${fg} / ${bg} = ${ratio.toFixed(2)}`).toBeGreaterThanOrEqual(min);
      }
    });
  }

  it("다크 값 두 곳(시스템 설정, 직접 선택)이 같다", () => {
    expect(colors(':root:not([data-theme="light"]) {')).toEqual(themes.다크);
  });

  it("주 버튼은 파랑이 아니다", () => {
    expect(themes.라이트["--primary"]).not.toBe(themes.라이트["--accent"]);
    expect(themes.라이트["--accent-text"]).toBeUndefined();
  });
});
