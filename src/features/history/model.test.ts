import { describe, expect, it } from "vitest";
import { logPage } from "../../test/logPage";
import { toHistoryRows } from "./model";

const commit = (oid: string, parents: string[]) => ({
  oid,
  shortOid: oid.slice(0, 7),
  parents,
  summary: `저장 ${oid}`,
  authorName: "지원",
  authorEmail: "j@example.com",
  authorTime: 1_700_000_000,
});

describe("toHistoryRows", () => {
  it("저장을 화면 모델로 옮긴다", () => {
    const rows = toHistoryRows(
      logPage({
        commits: [commit("ccc", ["aaa", "bbb"]), commit("aaa", [])],
      }),
    );
    expect(rows).toEqual([
      {
        oid: "ccc",
        shortOid: "ccc",
        summary: "저장 ccc",
        author: "지원",
        time: 1_700_000_000,
        parents: ["aaa", "bbb"],
        isMerge: true,
      },
      expect.objectContaining({ oid: "aaa", isMerge: false }),
    ]);
  });
});
