import { describe, expect, it } from "vitest";
import type { DisplayHintsView } from "./api";
import { computeUiScale, FONT_STEPS, MAX_SCALE, MIN_SCALE, stepDown, stepUp } from "./fontScale";

const hints = (bodyPx: number, textScalePercent = 100): DisplayHintsView => ({ bodyPx, textScalePercent, dark: null });

describe("화면 배율(D-146)", () => {
  it("Windows 기본 보통은 1, Ubuntu 기본은 15/14", () => {
    expect(computeUiScale(hints(14), "normal")).toBe(1);
    expect(computeUiScale(hints(15), "normal")).toBeCloseTo(15 / 14);
  });

  it("OS 글자 배율을 곱한다", () => {
    expect(computeUiScale(hints(14, 125), "normal")).toBeCloseTo(1.25);
    expect(computeUiScale(hints(14, 150), "large")).toBeCloseTo(1.5 * (16 / 14));
  });

  it("하한·상한으로 자른다", () => {
    expect(computeUiScale(hints(14, 50), "small")).toBe(MIN_SCALE);
    expect(computeUiScale(hints(15, 300), "xlarge")).toBe(MAX_SCALE);
  });

  it("어떤 OS 값과 단계에서도 가장 작은 보조 글자가 12px 이상", () => {
    for (const body of [14, 15]) {
      for (let percent = 50; percent <= 300; percent += 5) {
        for (const step of FONT_STEPS) {
          const scale = computeUiScale(hints(body, percent), step);
          expect(16 * scale * 0.8125).toBeGreaterThanOrEqual(12 - 1e-9);
        }
      }
    }
  });

  it("단계는 한 칸씩, 끝에서는 그대로", () => {
    expect(stepUp("normal")).toBe("large");
    expect(stepUp("xlarge")).toBe("xlarge");
    expect(stepDown("normal")).toBe("small");
    expect(stepDown("small")).toBe("small");
  });
});
