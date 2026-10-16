import { describe, expect, it } from "vitest";
import { joinPath, suggestName } from "./name";

describe("suggestName", () => {
  it.each([
    ["https://github.com/octo/hello.git", "hello"],
    ["https://github.com/octo/hello", "hello"],
    ["https://github.com/octo/hello/", "hello"],
    ["https://github.com/octo/hello.git/", "hello"],
    ["  https://github.com/octo/my.repo  ", "my.repo"],
    ["git@github.com:octo/hello.git", "hello"],
    ["ssh://git@github.com:22/octo/hello.git", "hello"],
    ["https://github.com/octo/한글-저장소", "한글-저장소"],
    ["https://github.com/octo/a*b?c", "a-b-c"],
    ["https://github.com/octo/.hidden", "hidden"],
    ["https://github.com/octo/..", ""],
    ["https://github.com/", "github.com"],
    ["", ""],
  ])("%s → %s", (url, want) => {
    expect(suggestName(url)).toBe(want);
  });
});

describe("joinPath", () => {
  it("구분자를 따른다", () => {
    expect(joinPath("/home/me", "hello")).toBe("/home/me/hello");
    expect(joinPath("/home/me/", "hello")).toBe("/home/me/hello");
    expect(joinPath("C:\\Users\\me", "hello")).toBe("C:\\Users\\me\\hello");
    expect(joinPath("", "hello")).toBe("hello");
  });
});
