import { describe, expect, it } from "vitest";
import { formatRelative } from "./time";

const now = 1_700_000_000_000;

describe("formatRelative", () => {
  it("한국어 상대 시각", () => {
    expect(formatRelative(now / 1000 - 3 * 3600, now, "ko")).toBe("3시간 전");
    expect(formatRelative(now / 1000 - 2 * 24 * 3600, now, "ko")).toBe("그저께");
    expect(formatRelative(now / 1000 - 10, now, "ko")).toBe("지금");
  });

  it("영어 상대 시각", () => {
    expect(formatRelative(now / 1000 - 5 * 60, now, "en")).toBe("5 minutes ago");
  });
});
