// @vitest-environment node
// 한국어·영어 문구가 같은 열쇠와 모양을 갖는지(D-139). 형은 tsc가 맞추고, 여기서는 실행해서 빈 글자가 없는지 본다.
import { describe, expect, it } from "vitest";
import { en } from "./en";
import { ko } from "./ko";
import { messageStrings } from "./samples";

/** 열쇠 경로와 값 종류(문자열·함수·객체). */
function shape(value: unknown, path = ""): string[] {
  if (value && typeof value === "object") {
    return Object.entries(value).flatMap(([k, v]) => shape(v, path ? `${path}.${k}` : k));
  }
  return [`${path}:${typeof value}`];
}

describe("ko·en 문구", () => {
  it("열쇠와 값 종류가 같다", () => {
    expect(shape(en).sort()).toEqual(shape(ko).sort());
  });

  it("모든 문구가 빈 글자가 아니다", () => {
    for (const [path, text] of [...messageStrings(ko), ...messageStrings(en)]) {
      expect(text.trim().length, path).toBeGreaterThan(0);
    }
  });
});
