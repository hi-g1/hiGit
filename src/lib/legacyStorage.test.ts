import { describe, expect, it } from "vitest";
import { clearLegacyStorage } from "./legacyStorage";

describe("옛 충돌 선택 지우기(D-211)", () => {
  it("higit.conflict.* 키만 지운다", () => {
    window.localStorage.setItem("higit.conflict./repo|abc", "{}");
    window.localStorage.setItem("higit.conflict./other|def", "{}");
    window.localStorage.setItem("다른.키", "남김");
    clearLegacyStorage();
    expect(window.localStorage.getItem("higit.conflict./repo|abc")).toBeNull();
    expect(window.localStorage.getItem("higit.conflict./other|def")).toBeNull();
    expect(window.localStorage.getItem("다른.키")).toBe("남김");
    window.localStorage.removeItem("다른.키");
  });
});
