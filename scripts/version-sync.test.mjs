// @vitest-environment node
// 앱 버전은 package.json·tauri.conf.json·Cargo 작업 공간이 같아야 한다(D-220). 설치 파일 이름과 앱 정보가 어긋나지 않게.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const read = (rel) => readFileSync(`${ROOT}${rel}`, "utf8");

describe("버전 맞춤", () => {
  it("세 곳의 버전이 같다", () => {
    const pkg = JSON.parse(read("package.json")).version;
    const tauri = JSON.parse(read("src-tauri/tauri.conf.json")).version;
    const cargo = /\[workspace\.package\][^[]*?\nversion = "([^"]+)"/.exec(read("src-tauri/Cargo.toml"))?.[1];
    expect(pkg).toMatch(/^\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?$/);
    expect(tauri).toBe(pkg);
    expect(cargo).toBe(pkg);
  });
});
