// @vitest-environment node
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { en } from "./en";
import { ko } from "./ko";
import { TERM_IDS, type TermEntry } from "./types";

type SpecRow = { id: string } & TermEntry;

/** docs/spec.md의 "기능 목록과 용어표"를 읽는다. */
function readSpecTable(): SpecRow[] {
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
    const [id, friendly, original, description] = cells;
    if (!id || id === "id" || id.startsWith("---")) continue;
    if (friendly === undefined || original === undefined || description === undefined) {
      throw new Error(`용어표 행 형식이 잘못되었습니다: ${line}`);
    }
    const [gitName, command] = original.split(" · ");
    rows.push({
      id,
      friendly,
      gitName: gitName ?? original,
      command: command ? command.replace(/^`|`$/g, "") : null,
      description,
    });
  }
  return rows;
}

describe("glossary와 docs/spec.md 용어표", () => {
  const rows = readSpecTable();

  it("표를 읽었다", () => {
    expect(rows.length).toBeGreaterThan(0);
  });

  it("id 목록과 순서가 같다", () => {
    expect([...TERM_IDS]).toEqual(rows.map((r) => r.id));
  });

  it("ko.ts의 각 항목이 표와 같다", () => {
    for (const { id, ...entry } of rows) {
      expect(ko[id as keyof typeof ko], id).toEqual(entry);
    }
  });
});

describe("en.ts", () => {
  it("ko.ts와 id 집합이 같다", () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(ko).sort());
  });

  it("원래 이름과 명령은 ko.ts와 같다", () => {
    for (const id of TERM_IDS) {
      expect(en[id].gitName, id).toBe(ko[id].gitName);
      expect(en[id].command, id).toBe(ko[id].command);
    }
  });

  it("빈 문구가 없다", () => {
    for (const id of TERM_IDS) {
      expect(en[id].friendly.trim(), id).not.toBe("");
      expect(en[id].description.trim(), id).not.toBe("");
    }
  });
});
