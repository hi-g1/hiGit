import { describe, expect, it } from "vitest";
import { isExtendClick, listKey } from "./listKeys";

const k = (key: string, mods: Partial<{ shiftKey: boolean; ctrlKey: boolean; altKey: boolean; metaKey: boolean; isComposing: boolean }> = {}) => ({
  key,
  shiftKey: false,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe("목록 조작 키", () => {
  it("방향·쪽·처음·끝·열기", () => {
    expect(["ArrowDown", "ArrowUp", "PageDown", "PageUp", "Home", "End", "Enter", " ", "ContextMenu"].map((x) => listKey(k(x)))).toEqual([
      "next",
      "prev",
      "pageNext",
      "pagePrev",
      "first",
      "last",
      "open",
      "open",
      "open",
    ]);
    expect(listKey(k("F10", { shiftKey: true }))).toBe("open");
  });

  it("수정키·조합 중·다른 키는 아니다", () => {
    expect(listKey(k("ArrowDown", { ctrlKey: true }))).toBeNull();
    expect(listKey(k("ArrowDown", { shiftKey: true, ctrlKey: true }))).toBeNull();
    expect(listKey(k("ArrowDown", { isComposing: true }))).toBeNull();
    expect(listKey(k("a"))).toBeNull();
  });

  it("Shift+↑↓는 구간 넓히기, 한글 조합 중에는 아니다", () => {
    expect(listKey(k("ArrowDown", { shiftKey: true }))).toBe("extendNext");
    expect(listKey(k("ArrowUp", { shiftKey: true }))).toBe("extendPrev");
    expect(listKey(k("ArrowUp", { shiftKey: true, isComposing: true }))).toBeNull();
    expect(listKey(k("Home", { shiftKey: true }))).toBeNull();
  });

  it("구간을 넓히는 누름은 Shift만 누른 클릭", () => {
    expect(isExtendClick(k("", { shiftKey: true }))).toBe(true);
    expect(isExtendClick(k("", { shiftKey: true, ctrlKey: true }))).toBe(false);
    expect(isExtendClick(k(""))).toBe(false);
  });
});
