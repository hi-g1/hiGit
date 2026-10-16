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

  it.each([
    ["Intl.DateTimeFormat", `export const f = (d: Date) => new Intl.DateTimeFormat("ko").format(d);`],
    ["Intl.RelativeTimeFormat", `export const f = () => new Intl.RelativeTimeFormat("ko").format(1, "day");`],
    ["toLocaleDateString", `export const f = (d: Date) => d.toLocaleDateString();`],
    ["navigator.language", `export const f = () => navigator.language;`],
    ["navigator.languages", `export const f = () => navigator.languages[0];`],
  ])("화면 코드의 %s는 오류(D-149, D-150)", async (_name, code) => {
    expect((await errors(code)).length).toBeGreaterThan(0);
  });

  it("시각·언어 파일에서는 통과", async () => {
    const [time] = await eslint.lintText(`export const f = (d: Date) => new Intl.DateTimeFormat("ko").format(d);`, {
      filePath: `${ROOT}src/lib/time.ts`,
    });
    expect(time.messages.filter((m) => m.severity === 2)).toEqual([]);
    const [locale] = await eslint.lintText(`export const f = () => navigator.languages;`, { filePath: `${ROOT}src/lib/locale.ts` });
    expect(locale.messages.filter((m) => m.severity === 2)).toEqual([]);
  });

  it("평범한 JSX 글자는 통과", async () => {
    expect(await errors(`export const A = (p: { s: string }) => <p>{p.s}</p>;`)).toEqual([]);
  });
});
