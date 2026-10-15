// @vitest-environment node
import { describe, expect, it } from "vitest";
import { isTermId } from "../glossary";
import { readSpecTable } from "../test/specTable";
import actionMeta from "./action-meta.json";

describe("등록된 액션과 용어표", () => {
  const specRisk = new Map(readSpecTable().map((row) => [row.id, row.risk]));

  it("액션이 하나 이상 등록되어 있다", () => {
    expect(actionMeta.length).toBeGreaterThan(0);
  });

  it("모든 glossaryId가 용어표에 있다", () => {
    for (const meta of actionMeta) {
      expect(isTermId(meta.glossaryId), meta.id).toBe(true);
    }
  });

  it("위험도가 spec.md 표와 같다", () => {
    for (const meta of actionMeta) {
      expect(meta.risk, meta.id).toBe(specRisk.get(meta.glossaryId));
    }
  });
});
