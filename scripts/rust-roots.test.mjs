// @vitest-environment node
// 모든 Rust crate 루트에 #![forbid(unsafe_code)]가 있는지 확인한다(D-038).
// 새 crate나 실행 파일을 만들고 빠뜨리면 실패한다.
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const TAURI = fileURLToPath(new URL("../src-tauri/", import.meta.url));

function roots() {
  const found = ["src/lib.rs", "src/main.rs"];
  for (const name of readdirSync(join(TAURI, "src/bin"))) {
    if (name.endsWith(".rs")) found.push(`src/bin/${name}`);
  }
  for (const crate of readdirSync(join(TAURI, "crates"))) {
    const lib = `crates/${crate}/src/lib.rs`;
    if (existsSync(join(TAURI, lib))) found.push(lib);
    const main = `crates/${crate}/src/main.rs`;
    if (existsSync(join(TAURI, main))) found.push(main);
  }
  return found;
}

describe("Rust crate 루트", () => {
  it("모두 unsafe 코드를 금지한다", () => {
    const missing = roots().filter(
      (rel) => !readFileSync(join(TAURI, rel), "utf8").includes("#![forbid(unsafe_code)]"),
    );
    expect(missing).toEqual([]);
  });

  it("crate가 하나 이상 잡힌다", () => {
    expect(roots().length).toBeGreaterThanOrEqual(8);
  });
});
