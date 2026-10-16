// @vitest-environment node
import { describe, expect, it } from "vitest";
import { choseong, matchQuery, normalizeJamo } from "./hangul";

describe("한글 찾기(D-199)", () => {
  it("초성으로 바꾼다", () => {
    expect(choseong("브랜치")).toBe("ㅂㄹㅊ");
    expect(choseong("버전 저장")).toBe("ㅂㅈ ㅈㅈ");
    expect(choseong("Git 동기화")).toBe("Git ㄷㄱㅎ");
    expect(choseong("까치")).toBe("ㄲㅊ");
  });

  it("조합 자모와 풀어 쓴 음절을 호환 꼴로 맞춘다", () => {
    expect(normalizeJamo("ᄇᄅᄎ")).toBe("ㅂㄹㅊ");
    expect(normalizeJamo("브랜치")).toBe("브랜치");
  });

  it("초성·섞인 찾기·조합 중인 마지막 글자", () => {
    expect(matchQuery("ㅂㄹㅊ", "브랜치")).toBe(2);
    expect(matchQuery("ㅂㅈㅈㅈ", "버전 저장")).toBe(2);
    expect(matchQuery("버ㅈ", "버전 저장")).toBe(2);
    expect(matchQuery("버저", "버전 저장")).toBe(2);
    expect(matchQuery("ᄇᄅᄎ", "브랜치 목록")).toBe(2);
    expect(matchQuery("ㅈㅂ", "브랜치")).toBe(0);
  });

  it("그대로 들어 있으면 가장 높고, 원래 이름·설명은 낮다", () => {
    expect(matchQuery("동기", "동기화")).toBe(3);
    expect(matchQuery("SYNC", "Sync")).toBe(3);
    expect(matchQuery("push", "올리기", ["push", "git push"])).toBe(1);
    expect(matchQuery("ㄱㅅ", "올리기", ["원격에 내 버전 보내기"])).toBe(0);
    expect(matchQuery("", "아무거나")).toBe(1);
    expect(matchQuery("   ", "아무거나")).toBe(1);
  });
});
