import { describe, expect, it } from "vitest";
import { formatRelative } from "./time";

const now = 1_700_000_000_000;

describe("formatDate", () => {
  it("올해면 월·일, 아니면 연도까지", async () => {
    const { formatDate } = await import("./time");
    const now = Date.UTC(2026, 9, 15, 3);
    expect(formatDate(Date.UTC(2026, 9, 7, 3) / 1000, now, "ko")).toBe("10월 7일");
    expect(formatDate(Date.UTC(2025, 9, 7, 3) / 1000, now, "ko")).toBe("2025년 10월 7일");
  });
});

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
