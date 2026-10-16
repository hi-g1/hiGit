import { describe, expect, it } from "vitest";
import { useUiStore } from "../state/uiStore";
import { applyDisplay, DEFAULT_SETTINGS } from "./display";

describe("화면 설정 적용", () => {
  it("루트 글자 크기를 16px × 배율로, 배율을 스토어에", () => {
    applyDisplay({ ...DEFAULT_SETTINGS, fontSize: "xlarge" }, { bodyPx: 14, textScalePercent: 100, dark: null });
    expect(document.documentElement.style.fontSize).toBe(`${16 * (18 / 14)}px`);
    expect(useUiStore.getState().uiScale).toBeCloseTo(18 / 14);
  });

  it("읽지 못한 값은 기본값(배율 1)", () => {
    applyDisplay(undefined, undefined);
    expect(document.documentElement.style.fontSize).toBe("16px");
    expect(useUiStore.getState().uiScale).toBe(1);
  });
});
