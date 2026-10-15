import { describe, expect, it } from "vitest";
import { curveH, fitText, textWidth } from "./geometry";

describe("글자 폭", () => {
  it("한글은 1em, 영문·숫자는 0.55em", () => {
    expect(textWidth("가나", 12)).toBe(24);
    expect(textWidth("ab12", 10)).toBeCloseTo(22);
    expect(textWidth("가a", 10)).toBeCloseTo(15.5);
    expect(textWidth("", 12)).toBe(0);
  });

  it("넘치면 뒤를 …로 줄인다", () => {
    expect(fitText("짧음", 100, 12)).toBe("짧음");
    const cut = fitText("로그인 화면 다시 만들기", 60, 12);
    expect(cut.endsWith("…")).toBe(true);
    expect(textWidth(cut, 12)).toBeLessThanOrEqual(60);
    expect(fitText("가나다", 5, 12)).toBe("…");
  });

  it("곡선 조각", () => {
    expect(curveH(0, 10, 40, 50)).toBe("M 0 10 C 20 10, 20 50, 40 50");
  });
});
