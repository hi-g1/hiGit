import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SHORTCUTS, matches, type ShortcutId } from "./shortcuts";
import { useShortcut } from "./useShortcut";

const key = (code: string, mods: Partial<KeyboardEventInit> = {}, extra: Partial<KeyboardEventInit> = {}) =>
  new KeyboardEvent("keydown", { code, key: "ㅐ", ...mods, ...extra });

describe("단축키 표", () => {
  it("같은 키 조합이 두 번 나오지 않는다", () => {
    const combos = Object.values(SHORTCUTS).map((s) => `${s.code}|${s.ctrl}|${s.shift}|${s.alt}`);
    expect(new Set(combos).size).toBe(combos.length);
  });

  it("한글 입력 상태(key가 ㅐ)에서도 물리 키로 판정한다", () => {
    expect(matches("openFolder", key("KeyO", { ctrlKey: true }))).toBe(true);
  });

  it("조합 중이면 무시한다", () => {
    expect(matches("openFolder", key("KeyO", { ctrlKey: true }, { isComposing: true }))).toBe(false);
    expect(matches("openFolder", key("KeyO", { ctrlKey: true }, { keyCode: 229 }))).toBe(false);
  });

  it("Shift가 더해지면 다른 단축키다", () => {
    expect(matches("openFolder", key("KeyO", { ctrlKey: true, shiftKey: true }))).toBe(false);
    expect(matches("openFolder", key("KeyO"))).toBe(false);
  });

  it("useShortcut은 창에서 단축키를 받고 기본 동작을 막는다", () => {
    const handler = vi.fn();
    const { unmount } = renderHook(() => useShortcut("openFolder" as ShortcutId, handler));
    const e = key("KeyO", { ctrlKey: true, cancelable: true });
    window.dispatchEvent(e);
    expect(handler).toHaveBeenCalledTimes(1);
    expect(e.defaultPrevented).toBe(true);
    unmount();
    window.dispatchEvent(key("KeyO", { ctrlKey: true }));
    expect(handler).toHaveBeenCalledTimes(1);
  });
});
