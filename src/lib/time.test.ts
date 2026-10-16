import { describe, expect, it } from "vitest";
import { formatClock, formatDate, formatExact, formatTime } from "./time";

// 시간대와 무관하게 로컬 시각으로 만든다.
const at = (y: number, mo: number, d: number, h = 0, mi = 0, s = 0) => new Date(y, mo - 1, d, h, mi, s).getTime();
const sec = (ms: number) => Math.floor(ms / 1000);
const now = at(2026, 10, 16, 15, 30);

describe("시각 표시 규칙(D-150)", () => {
  it("1분 안과 미래(시계 차이)는 지금", () => {
    expect(formatTime(sec(now) - 30, now, "ko")).toBe("지금");
    expect(formatTime(sec(now) + 120, now, "ko")).toBe("지금");
    expect(formatTime(sec(now) - 30, now, "en")).toBe("now");
  });

  it("1시간 안은 N분 전", () => {
    expect(formatTime(sec(now) - 5 * 60, now, "ko")).toBe("5분 전");
    expect(formatTime(sec(now) - 59 * 60, now, "en")).toBe("59 minutes ago");
  });

  it("같은 날은 시각", () => {
    expect(formatTime(sec(at(2026, 10, 16, 15, 12)) - 3600, now, "ko")).toBe("오후 2:12");
    expect(formatTime(sec(at(2026, 10, 16, 9, 5)), now, "ko")).toBe("오전 9:05");
    expect(formatTime(sec(at(2026, 10, 16, 9, 5)), now, "en")).toBe("9:05 AM");
  });

  it("자정을 넘기면 날짜", () => {
    const justAfterMidnight = at(2026, 10, 17, 0, 30);
    expect(formatTime(sec(at(2026, 10, 16, 23, 0)), justAfterMidnight, "ko")).toBe("10월 16일");
  });

  it("같은 해는 월·일, 그 전은 연도까지", () => {
    expect(formatTime(sec(at(2026, 10, 7, 10)), now, "ko")).toBe("10월 7일");
    expect(formatTime(sec(at(2025, 10, 7, 10)), now, "ko")).toBe("2025년 10월 7일");
    expect(formatTime(sec(at(2026, 10, 7, 10)), now, "en")).toBe("October 7");
    expect(formatTime(sec(at(2025, 10, 7, 10)), now, "en")).toBe("October 7, 2025");
  });

  it("정확한 시각과 시·분", () => {
    expect(formatExact(sec(at(2026, 10, 16, 15, 12, 5)), "ko")).toBe("2026년 10월 16일 오후 3:12:05");
    expect(formatClock(sec(at(2026, 10, 16, 15, 12)), "ko")).toMatch(/03:12|3:12/);
    expect(formatDate(sec(at(2025, 10, 7)), now, "ko")).toBe("2025년 10월 7일");
  });
});
