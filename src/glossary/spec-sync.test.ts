// @vitest-environment node
import { describe, expect, it } from "vitest";
import { readSpecTable } from "../test/specTable";
import { en } from "./en";
import { ko } from "./ko";
import { TERM_IDS } from "./types";

describe("glossary와 docs/spec.md 용어표", () => {
  const rows = readSpecTable();

  it("표를 읽었다", () => {
    expect(rows.length).toBeGreaterThan(0);
  });

  it("id 목록과 순서가 같다", () => {
    expect([...TERM_IDS]).toEqual(rows.map((r) => r.id));
  });

  it("ko.ts의 각 항목이 표와 같다", () => {
    for (const { id, friendly, gitName, command, description } of rows) {
      expect(ko[id as keyof typeof ko], id).toEqual({ friendly, gitName, command, description });
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
