import { describe, expect, it } from "vitest";
import { clampPane, paneKey, paneMaxRem } from "./paneWidth";

const k = (key: string, mods: Partial<{ shiftKey: boolean; ctrlKey: boolean; altKey: boolean; metaKey: boolean; isComposing: boolean }> = {}) => ({
  key,
  shiftKey: false,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe("칸 폭 계산(D-164)", () => {
  it("최소·최대 안으로, 최대가 최소보다 작으면 최소", () => {
    expect(clampPane(10, 15, 30)).toBe(15);
    expect(clampPane(40, 15, 30)).toBe(30);
    expect(clampPane(20, 15, 30)).toBe(20);
    expect(clampPane(20, 15, 12)).toBe(15);
  });

  it("최대는 칸들 폭의 50%, 폭을 모르면 끝이 없다", () => {
    expect(paneMaxRem(960, 16)).toBe(30);
    expect(paneMaxRem(960, 20)).toBe(24);
    expect(paneMaxRem(0, 16)).toBe(Number.POSITIVE_INFINITY);
  });

  it("←·→만, 조합 중·수정키는 무시", () => {
    expect(paneKey(k("ArrowLeft"))).toBe(-1);
    expect(paneKey(k("ArrowRight"))).toBe(1);
    expect(paneKey(k("ArrowUp"))).toBe(0);
    expect(paneKey(k("ArrowRight", { isComposing: true }))).toBe(0);
    expect(paneKey(k("ArrowRight", { ctrlKey: true }))).toBe(0);
    expect(paneKey(k("ArrowLeft", { shiftKey: true }))).toBe(0);
  });
});
