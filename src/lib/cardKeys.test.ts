import { describe, expect, it } from "vitest";
import { cardKey } from "./cardKeys";

const k = (key: string, mods: Partial<{ shiftKey: boolean; ctrlKey: boolean; altKey: boolean; metaKey: boolean; isComposing: boolean; keyCode: number }> = {}) => ({
  key,
  shiftKey: false,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe("카드 조작 키", () => {
  it("Alt+↑↓는 옮기기, ↑↓는 고르기, Delete·Backspace는 빼기, Enter는 메시지 고치기", () => {
    expect(cardKey(k("ArrowUp", { altKey: true }))).toBe("moveUp");
    expect(cardKey(k("ArrowDown", { altKey: true }))).toBe("moveDown");
    expect(cardKey(k("ArrowUp"))).toBe("prev");
    expect(cardKey(k("ArrowDown"))).toBe("next");
    expect(cardKey(k("Delete"))).toBe("toggleDrop");
    expect(cardKey(k("Backspace"))).toBe("toggleDrop");
    expect(cardKey(k("Enter"))).toBe("edit");
  });

  it("조합 중·다른 수정키·다른 키는 아니다", () => {
    expect(cardKey(k("Enter", { isComposing: true }))).toBeNull();
    expect(cardKey(k("Enter", { keyCode: 229 }))).toBeNull();
    expect(cardKey(k("ArrowUp", { ctrlKey: true }))).toBeNull();
    expect(cardKey(k("ArrowUp", { shiftKey: true }))).toBeNull();
    expect(cardKey(k("x", { altKey: true }))).toBeNull();
    expect(cardKey(k("a"))).toBeNull();
  });
});
