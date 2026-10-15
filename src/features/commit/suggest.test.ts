import { describe, expect, it } from "vitest";
import type { ChangedFile } from "../../lib/api";
import { en } from "../../messages/en";
import { ko } from "../../messages/ko";
import { filesToSave, suggestName } from "./suggest";

const f = (path: string, change: ChangedFile["change"], staged = false, origPath: string | null = null): ChangedFile => ({
  path,
  origPath,
  change,
  staged,
});

describe("버전 이름 제안", () => {
  it.each([
    [[f("src/login/화면.tsx", "modified")], "화면.tsx 고침", "Update 화면.tsx"],
    [[f("src/새.ts", "untracked")], "새.ts 추가", "Add 새.ts"],
    [[f("old.txt", "deleted")], "old.txt 삭제", "Delete old.txt"],
    [[f("b/new.txt", "renamed", false, "a/old.txt")], "old.txt 이름 바꿈", "Rename old.txt"],
    [[f("화면.tsx", "modified"), f("비밀번호.ts", "untracked"), f("배경.png", "modified")], "화면.tsx 외 2개 고침", "Update 화면.tsx and 2 more"],
    [[f("a.ts", "untracked"), f("b.ts", "added")], "a.ts 외 1개 추가", "Add a.ts and 1 more"],
    [[f("a.ts", "deleted"), f("b.ts", "deleted")], "a.ts 외 1개 삭제", "Delete a.ts and 1 more"],
  ])("%#", (files, wantKo, wantEn) => {
    expect(suggestName(files, ko)).toBe(wantKo);
    expect(suggestName(files, en)).toBe(wantEn);
  });

  it("넣은 파일이 있으면 그것만, 없으면 모두", () => {
    const files = [f("a.ts", "modified"), f("b.ts", "modified", true)];
    expect(filesToSave(files).map((x) => x.path)).toEqual(["b.ts"]);
    expect(suggestName(files, ko)).toBe("b.ts 고침");
    expect(filesToSave([f("a.ts", "modified")]).length).toBe(1);
    expect(suggestName([], ko)).toBe("");
  });
});
