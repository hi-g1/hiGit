import { describe, expect, it } from "vitest";
import type { RecoveryItem } from "../../lib/api";
import { groupByDay, restoreInput } from "./model";

const item = (over: Partial<RecoveryItem>): RecoveryItem => ({
  id: "x",
  at: 0,
  source: "journal",
  action: "commit",
  label: "main",
  summary: "버전",
  oid: "a".repeat(40),
  refname: null,
  restore: "version",
  undone: false,
  ...over,
});

describe("작업 내역 모델", () => {
  it("같은 날끼리 묶는다", () => {
    const items = [item({ id: "1", at: 200 }), item({ id: "2", at: 150 }), item({ id: "3", at: 50 })];
    const groups = groupByDay(items, (at) => (at >= 100 ? "오늘" : "어제"));
    expect(groups.map((g) => [g.day, g.items.length])).toEqual([
      ["오늘", 2],
      ["어제", 1],
    ]);
  });

  it("단추가 부를 입력은 조회가 준 값 그대로", () => {
    expect(restoreInput(item({}))).toEqual({ id: "restore", input: { oid: "a".repeat(40) } });
    expect(restoreInput(item({ source: "stash", restore: "changes" }))).toEqual({
      id: "take-out",
      input: { source: { kind: "stash", oid: "a".repeat(40) } },
    });
    expect(restoreInput(item({ source: "backup", restore: "changes", refname: "refs/higit/backup/x/y" }))).toEqual({
      id: "take-out",
      input: { source: { kind: "ref", refname: "refs/higit/backup/x/y" } },
    });
    expect(restoreInput(item({ restore: "none" }))).toBeNull();
  });
});
