// 테스트 전용: docs/spec.md의 "기능 목록과 용어표"를 읽는다. node 환경 테스트에서만 쓴다.
import { readFileSync } from "node:fs";
import type { TermEntry } from "../glossary";

export type SpecRow = { id: string; risk: string } & TermEntry;

export function readSpecTable(): SpecRow[] {
  const spec = readFileSync(new URL("../../docs/spec.md", import.meta.url), "utf8");
  const start = spec.indexOf("## 기능 목록과 용어표");
  if (start < 0) throw new Error("spec.md에 용어표 절이 없습니다");

  const rows: SpecRow[] = [];
  for (const line of spec.slice(start).split(/\r?\n/)) {
    if (!line.startsWith("|")) {
      if (rows.length > 0) break;
      continue;
    }
    const cells = line
      .trim()
      .replace(/^\||\|$/g, "")
      .split("|")
      .map((c) => c.trim());
    const [id, friendly, original, description, risk] = cells;
    if (!id || id === "id" || id.startsWith("---")) continue;
    if (
      friendly === undefined ||
      original === undefined ||
      description === undefined ||
      risk === undefined
    ) {
      throw new Error(`용어표 행 형식이 잘못되었습니다: ${line}`);
    }
    const [gitName, command] = original.split(" · ");
    rows.push({
      id,
      friendly,
      gitName: gitName ?? original,
      command: command ? command.replace(/^`|`$/g, "") : null,
      description,
      risk,
    });
  }
  return rows;
}
