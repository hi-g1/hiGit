import { describe, expect, it } from "vitest";
import { useUiStore } from "../state/uiStore";
import { applyDisplay, DEFAULT_SETTINGS, themeAttribute } from "./display";

describe("화면 설정 적용", () => {
  it("루트 글자 크기를 16px × 배율로, 배율을 스토어에", () => {
    applyDisplay({ ...DEFAULT_SETTINGS, fontSize: "xlarge" }, { bodyPx: 14, textScalePercent: 100, dark: null });
    expect(document.documentElement.style.fontSize).toBe(`${16 * (18 / 14)}px`);
    expect(useUiStore.getState().uiScale).toBeCloseTo(18 / 14);
  });

  it("언어를 정해 스토어와 문서 lang에", () => {
    applyDisplay({ ...DEFAULT_SETTINGS, language: "en" }, undefined);
    expect(useUiStore.getState().locale).toBe("en");
    expect(document.documentElement.lang).toBe("en");
    applyDisplay({ ...DEFAULT_SETTINGS, language: "ko" }, undefined);
    expect(useUiStore.getState().locale).toBe("ko");
  });

  it("화면 모드: 라이트·다크는 그대로, 시스템은 OS 다크를 알면 그 값", () => {
    expect(themeAttribute("light", true)).toBe("light");
    expect(themeAttribute("dark", false)).toBe("dark");
    expect(themeAttribute("system", true)).toBe("dark");
    expect(themeAttribute("system", false)).toBe("light");
    expect(themeAttribute("system", null)).toBeNull();
    const hints = { bodyPx: 14, textScalePercent: 100, dark: null };
    applyDisplay({ ...DEFAULT_SETTINGS, theme: "dark" }, hints);
    expect(document.documentElement.dataset.theme).toBe("dark");
    applyDisplay({ ...DEFAULT_SETTINGS, theme: "system" }, hints);
    expect(document.documentElement.dataset.theme).toBeUndefined();
    applyDisplay({ ...DEFAULT_SETTINGS, theme: "system" }, { ...hints, dark: true });
    expect(document.documentElement.dataset.theme).toBe("dark");
    applyDisplay({ ...DEFAULT_SETTINGS, theme: "light" }, { ...hints, dark: true });
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("용어 설명 보기 설정을 스토어에", () => {
    applyDisplay({ ...DEFAULT_SETTINGS, termHelp: false }, undefined);
    expect(useUiStore.getState().termHelp).toBe(false);
  });

  it("읽지 못한 값은 기본값(배율 1)", () => {
    applyDisplay(undefined, undefined);
    expect(document.documentElement.style.fontSize).toBe("16px");
    expect(useUiStore.getState().uiScale).toBe(1);
  });
});
