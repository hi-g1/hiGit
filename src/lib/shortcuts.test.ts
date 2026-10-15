import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PLANNED, RESERVED, SHORTCUTS, matches, type Key, type ShortcutId } from "./shortcuts";
import { useShortcut } from "./useShortcut";

const key = (code: string, mods: Partial<KeyboardEventInit> = {}, extra: Partial<KeyboardEventInit> = {}) =>
  new KeyboardEvent("keydown", { code, key: "ㅐ", ...mods, ...extra });

describe("단축키 표", () => {
  const combo = (k: Key) => `${k.code}|${k.ctrl}|${k.shift}|${k.alt}`;
  const used = Object.values(SHORTCUTS).flatMap((s) => s.keys.map(combo));

  it("같은 키 조합이 두 번 나오지 않는다(계획된 키 포함)", () => {
    const all = [...used, ...PLANNED.map(combo)];
    expect(new Set(all).size).toBe(all.length);
  });

  it("웹뷰·OS가 쓰는 키와 겹치지 않는다", () => {
    const reserved = new Set(RESERVED.map(combo));
    expect(used.filter((c) => reserved.has(c))).toEqual([]);
  });

  it("다시 하기는 Ctrl+Shift+Z와 Ctrl+Y 둘 다", () => {
    expect(matches("redo", key("KeyZ", { ctrlKey: true, shiftKey: true }))).toBe(true);
    expect(matches("redo", key("KeyY", { ctrlKey: true }))).toBe(true);
    expect(matches("undo", key("KeyZ", { ctrlKey: true, shiftKey: true }))).toBe(false);
  });

  it("한글 상태(key가 ㅠ)에서 Ctrl+B를 물리 키로 판정한다", () => {
    expect(matches("branches", new KeyboardEvent("keydown", { code: "KeyB", key: "ㅠ", ctrlKey: true }))).toBe(true);
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

  it("되돌리기는 입력칸 포커스에서 글자 되돌리기에 맡긴다", () => {
    const handler = vi.fn();
    const { unmount } = renderHook(() => useShortcut("undo", handler));
    const input = document.createElement("input");
    document.body.appendChild(input);
    const inField = key("KeyZ", { ctrlKey: true, cancelable: true, bubbles: true });
    input.dispatchEvent(inField);
    expect(handler).not.toHaveBeenCalled();
    expect(inField.defaultPrevented).toBe(false);
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    document.body.appendChild(checkbox);
    checkbox.dispatchEvent(key("KeyZ", { ctrlKey: true, cancelable: true, bubbles: true }));
    expect(handler).toHaveBeenCalledTimes(1);
    window.dispatchEvent(key("KeyZ", { ctrlKey: true, cancelable: true }));
    expect(handler).toHaveBeenCalledTimes(2);
    input.remove();
    checkbox.remove();
    unmount();
  });
});
