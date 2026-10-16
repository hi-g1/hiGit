import { describe, expect, it } from "vitest";
import { choiceIndex, choiceKey } from "./choiceKeys";

const k = (key: string, mods: Partial<{ shiftKey: boolean; ctrlKey: boolean; altKey: boolean; metaKey: boolean; isComposing: boolean }> = {}) => ({
  key,
  shiftKey: false,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe("고르는 부품 키", () => {
  it("방향키·Home·End", () => {
    expect(choiceKey(k("ArrowRight"))).toBe("next");
    expect(choiceKey(k("ArrowDown"))).toBe("next");
    expect(choiceKey(k("ArrowLeft"))).toBe("prev");
    expect(choiceKey(k("ArrowUp"))).toBe("prev");
    expect(choiceKey(k("Home"))).toBe("first");
    expect(choiceKey(k("End"))).toBe("last");
    expect(choiceKey(k("Enter"))).toBeNull();
  });

  it("조합 중·수정키는 무시", () => {
    expect(choiceKey(k("ArrowRight", { isComposing: true }))).toBeNull();
    expect(choiceKey(k("ArrowRight", { ctrlKey: true }))).toBeNull();
    expect(choiceKey(k("ArrowRight", { shiftKey: true }))).toBeNull();
    expect(choiceKey(k("ArrowRight", { altKey: true }))).toBeNull();
  });

  it("끝에서는 돌아간다", () => {
    expect(choiceIndex("next", 3, 4)).toBe(0);
    expect(choiceIndex("prev", 0, 4)).toBe(3);
    expect(choiceIndex("last", 1, 4)).toBe(3);
  });
});
