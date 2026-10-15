// @vitest-environment node
// 화면 코드에서 HTML 주입과 eval을 린트가 막는지 확인한다(D-037).
import { ESLint } from "eslint";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const eslint = new ESLint({ cwd: ROOT });

async function errors(code) {
  const [result] = await eslint.lintText(code, { filePath: `${ROOT}src/features/x.tsx` });
  return result.messages.filter((m) => m.severity === 2).map((m) => m.ruleId);
}

describe("화면 코드 린트", () => {
  it.each([
    ["dangerouslySetInnerHTML", `export const A = (p: { h: string }) => <div dangerouslySetInnerHTML={{ __html: p.h }} />;`],
    ["innerHTML", `export function f(el: HTMLElement, s: string) { el.innerHTML = s; }`],
    ["outerHTML", `export function f(el: HTMLElement, s: string) { el.outerHTML = s; }`],
    ["insertAdjacentHTML", `export function f(el: HTMLElement, s: string) { el.insertAdjacentHTML("beforeend", s); }`],
    ["document.write", `export function f(s: string) { document.write(s); }`],
    ["eval", `export function f(s: string) { return eval(s); }`],
    ["new Function", `export function f(s: string) { return new Function(s); }`],
    ["setTimeout 문자열", `export function f() { setTimeout("alert(1)", 1); }`],
  ])("%s는 오류", async (_name, code) => {
    expect((await errors(code)).length).toBeGreaterThan(0);
  });

  it("평범한 JSX 글자는 통과", async () => {
    expect(await errors(`export const A = (p: { s: string }) => <p>{p.s}</p>;`)).toEqual([]);
  });
});
