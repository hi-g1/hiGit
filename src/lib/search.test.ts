import { describe, expect, it } from "vitest";
import { matchesSearch } from "./search";

describe("목록 찾기(D-185)", () => {
  it("대소문자를 무시한 부분 일치", () => {
    expect(matchesSearch("Octo/Hello-World", "hello")).toBe(true);
    expect(matchesSearch("octo/hello", "OCTO/H")).toBe(true);
    expect(matchesSearch("octo/hello", "bye")).toBe(false);
  });

  it("빈 찾기와 앞뒤 공백", () => {
    expect(matchesSearch("octo/hello", "")).toBe(true);
    expect(matchesSearch("octo/hello", "  hel  ")).toBe(true);
  });

  it("한글은 조합형과 완성형을 같게 본다", () => {
    const composed = "팀/블로그";
    const decomposed = composed.normalize("NFD");
    expect(matchesSearch(composed, decomposed)).toBe(true);
    expect(matchesSearch(decomposed, "블로")).toBe(true);
  });
});
