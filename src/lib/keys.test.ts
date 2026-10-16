import { describe, expect, it } from "vitest";
import { composing, isBlockedEnter, isComposingEnter, isEnter, isEscape, isSubmitEnter } from "./keys";

const key = (k: string, more: Record<string, unknown> = {}) => ({ key: k, ...more });

describe("키 판정(D-137)", () => {
  it("조합 중 Enter·Esc는 명령이 아니다(isComposing, WebKitGTK keyCode 229)", () => {
    for (const extra of [{ isComposing: true }, { keyCode: 229 }, { nativeEvent: { isComposing: true } }, { nativeEvent: { keyCode: 229 } }]) {
      expect(isEnter(key("Enter", extra))).toBe(false);
      expect(isSubmitEnter(key("Enter", extra))).toBe(false);
      expect(isEscape(key("Escape", extra))).toBe(false);
      expect(isComposingEnter(key("Enter", extra))).toBe(true);
      expect(isBlockedEnter(key("a", extra))).toBe(true);
      expect(composing(extra)).toBe(true);
    }
  });

  it("평소의 Enter·Esc", () => {
    expect(isEnter(key("Enter"))).toBe(true);
    expect(isSubmitEnter(key("Enter"))).toBe(true);
    expect(isEscape(key("Escape"))).toBe(true);
    expect(isComposingEnter(key("Enter"))).toBe(false);
    expect(isBlockedEnter(key("Enter"))).toBe(true);
    expect(isBlockedEnter(key("a"))).toBe(false);
    expect(isEnter(key("a"))).toBe(false);
  });

  it("Shift+Enter는 글상자 보내기가 아니다", () => {
    expect(isSubmitEnter(key("Enter", { shiftKey: true }))).toBe(false);
    expect(isEnter(key("Enter", { shiftKey: true }))).toBe(true);
  });
});
