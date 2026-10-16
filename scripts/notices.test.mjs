// @vitest-environment node
// 서드파티 고지 파일의 모양(D-220). 목록이 지금 의존성과 같은지는 `node scripts/notices.mjs --check`(CI·pre-push)가 본다.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const text = readFileSync(`${ROOT}THIRD_PARTY_LICENSES.txt`, "utf8").replace(/\r\n/g, "\n");
const list = text.slice(text.indexOf("목록\n----\n") + 8, text.indexOf("\n\n원문\n")).split("\n");
const groups = [...text.matchAll(/^=== (.+) ===$/gm)].map((m) => m[1]);

describe("서드파티 고지", () => {
  it("앱에 들어가는 알려진 의존성과 라이선스를 담는다", () => {
    expect(list.some((l) => /^rust {2}tauri 2\.\S+ {2}Apache-2\.0 OR MIT$/.test(l))).toBe(true);
    expect(list.some((l) => /^npm {3}react \S+ {2}MIT$/.test(l))).toBe(true);
    expect(list.some((l) => /^rust {2}git2? /.test(l))).toBe(false);
    expect(list.length).toBeGreaterThan(100);
  });

  it("목록의 모든 패키지에 원문이 있다", () => {
    const covered = new Set(groups.flatMap((g) => g.split(", ")));
    const missing = list.map((l) => l.split(/\s+/).slice(1, 3).join(" ")).filter((p) => !covered.has(p));
    expect(missing).toEqual([]);
  });

  it("글꼴 고지가 이 파일을 가리킨다", () => {
    expect(readFileSync(`${ROOT}THIRD_PARTY_NOTICES.md`, "utf8")).toContain("THIRD_PARTY_LICENSES.txt");
  });
});
