// @vitest-environment node
// 프로젝트 열기는 한 곳(D-032): 화면 코드에서 api.openProject를 부르는 곳은 useOpenProject뿐이다.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../../", import.meta.url));
const ALLOWED = new Set(["features/open-project/useOpenProject.ts", "lib/api.ts"]);

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : [path];
  });
}

describe("프로젝트 열기 입구", () => {
  it("api.openProject는 useOpenProject에서만 부른다", () => {
    const offenders = files(SRC)
      .filter((p) => /\.(ts|tsx)$/.test(p) && !/\.test\.tsx?$/.test(p))
      .map((p) => relative(SRC, p).replaceAll("\\", "/"))
      .filter((rel) => !ALLOWED.has(rel))
      .filter((rel) => /\bopenProject\(|commands\.projectOpen\(/.test(readFileSync(join(SRC, rel), "utf8")));
    expect(offenders).toEqual([]);
  });
});
