import { describe, expect, it } from "vitest";
import { resolveLocale } from "./locale";

describe("화면 언어(D-139, D-149)", () => {
  it("직접 고르면 그 언어", () => {
    expect(resolveLocale("ko", ["en-US"])).toBe("ko");
    expect(resolveLocale("en", ["ko-KR"])).toBe("en");
  });

  it("시스템 따라가기는 첫 언어가 한국어일 때만 한국어", () => {
    expect(resolveLocale("system", ["ko-KR", "en-US"])).toBe("ko");
    expect(resolveLocale("system", ["ko"])).toBe("ko");
    expect(resolveLocale("system", ["en-US", "ko"])).toBe("en");
    expect(resolveLocale("system", ["ja-JP"])).toBe("en");
    expect(resolveLocale("system", [])).toBe("en");
  });
});
