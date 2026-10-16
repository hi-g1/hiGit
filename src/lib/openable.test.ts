import { describe, expect, it } from "vitest";
import { OPEN_BLOCKED_EXTENSIONS } from "./bindings";
import { blockedExtension, openBlock } from "./openable";

describe("기본 앱으로 열기 차단표(D-216)", () => {
  it("core 상수에서 온다", () => {
    expect(OPEN_BLOCKED_EXTENSIONS).toContain("exe");
    expect(OPEN_BLOCKED_EXTENSIONS).toContain("sh");
  });

  it("마지막 확장자만 대소문자 없이 본다", () => {
    expect(blockedExtension("a/B.EXE")).toBe(true);
    expect(blockedExtension("a.pdf.exe")).toBe(true);
    expect(blockedExtension("a.exe.pdf")).toBe(false);
    expect(blockedExtension("install.sh")).toBe(true);
    expect(blockedExtension("README")).toBe(false);
    expect(blockedExtension("dir.exe/readme.md")).toBe(false);
  });

  it("지운 파일과 실행 파일은 까닭이 다르다", () => {
    expect(openBlock("a.txt", true)).toBe("deleted");
    expect(openBlock("a.bat", false)).toBe("runnable");
    expect(openBlock("a.png", false)).toBeNull();
  });
});
