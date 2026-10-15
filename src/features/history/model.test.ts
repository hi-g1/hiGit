import { describe, expect, it } from "vitest";
import { logPage } from "../../test/logPage";
import { toRouteNodes } from "./model";

const commit = (oid: string, parents: string[]) => ({
  oid,
  shortOid: oid.slice(0, 7),
  parents,
  summary: `저장 ${oid}`,
  authorName: "지원",
  authorEmail: "j@example.com",
  authorTime: 1_700_000_000,
});

describe("toRouteNodes", () => {
  it("쪽을 잇고 같은 해시는 한 번만, 이름표·지금 위치·안 올림을 붙인다", () => {
    const first = logPage({
      commits: [commit("ccc", ["bbb"]), commit("bbb", ["aaa"])],
      labels: [
        { oid: "ccc", kind: "branch", name: "기능" },
        { oid: "ccc", kind: "branch", name: "다른" },
        { oid: "bbb", kind: "tag", name: "v1" },
      ],
      head: "ccc",
      unpushed: ["ccc"],
    });
    const second = logPage({ commits: [commit("bbb", ["aaa"]), commit("aaa", [])], head: null });
    const nodes = toRouteNodes([first, second]);
    expect(nodes.map((x) => x.id)).toEqual(["ccc", "bbb", "aaa"]);
    expect(nodes[0]).toMatchObject({ branches: ["기능", "다른"], isHead: true, unpushed: true, kind: "commit" });
    expect(nodes[1]).toMatchObject({ tags: ["v1"], isHead: false, unpushed: false });
  });
});
