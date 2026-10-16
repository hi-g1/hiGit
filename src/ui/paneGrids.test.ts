// @vitest-environment node
// 나눔선을 쓰는 세 화면의 칸 폭 규칙(D-164): 15rem~칸들 폭의 50%, 폭은 --pane-width(rem)에서 온다.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (rel: string) => readFileSync(new URL(rel, import.meta.url), "utf8");

describe("나눔선 칸 규칙", () => {
  it.each([
    ["홈", "../screens/Workspace/Workspace.module.css", "21.25rem"],
    ["충돌 화면", "../screens/Conflict/Conflict.module.css", "15rem"],
    ["이때 파일 보기", "../features/history/menu/dialogs/FilesDialog.module.css", "16.25rem"],
  ])("%s", (_name, file, fallback) => {
    expect(read(file)).toContain(`grid-template-columns: clamp(15rem, var(--pane-width, ${fallback}), 50%) auto 1fr;`);
  });

  it("세 화면 모두 나눔선을 그린다", () => {
    for (const file of [
      "../screens/Workspace/Workspace.tsx",
      "../screens/Conflict/Conflict.tsx",
      "../features/history/menu/dialogs/FilesDialog.tsx",
    ]) {
      expect(read(file), file).toMatch(/<Splitter label=\{m\.paneWidth\} \{\.\.\.pane\.splitter\}/);
    }
  });
});
