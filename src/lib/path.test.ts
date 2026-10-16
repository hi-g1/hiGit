import { describe, expect, it } from "vitest";
import { splitPath } from "./path";

describe("경로 나누기", () => {
  it("이름과 폴더", () => {
    expect(splitPath("README.md")).toEqual({ name: "README.md", dir: "" });
    expect(splitPath("docs/plans/010-settings-github-more.md")).toEqual({ name: "010-settings-github-more.md", dir: "docs/plans" });
    expect(splitPath("화면/새 폴더/한글 파일.txt")).toEqual({ name: "한글 파일.txt", dir: "화면/새 폴더" });
  });
});
