import { describe, expect, it } from "vitest";
import { commitNode, type RouteNode } from "../model";
import { simplify } from "./simplify";

const n = (id: string, parents: string[] = [], over: Partial<RouteNode> = {}) => commitNode(id, parents, over);
const ids = (nodes: RouteNode[]) => nodes.map((x) => (x.kind === "fold" ? `[${x.count}]` : x.id));

/** 받은 버전의 모든 부모가 결과 안의 노드이거나 받은 범위 밖인지(선이 끊기지 않음). */
function connected(input: RouteNode[], out: RouteNode[]) {
  const inputIds = new Set(input.map((x) => x.id));
  const outIds = new Set(out.map((x) => x.id));
  for (const x of out) for (const p of x.parents) expect(outIds.has(p) || !inputIds.has(p), `${x.id} → ${p}`).toBe(true);
  // 위상 순서: 부모는 늘 아래.
  const row = new Map(out.map((x, i) => [x.id, i]));
  for (const x of out) for (const p of x.parents) if (row.has(p)) expect(row.get(p)!).toBeGreaterThan(row.get(x.id)!);
}

// main: h(지금, main) → m(합치기 a·s) → a → b → t(태그 v1) → c → r(첫 버전). 곁가지 s → s2 → b.
const history = () => [
  n("h", ["m"], { isHead: true, branches: ["main"] }),
  n("m", ["a", "s"]),
  n("s", ["s2"]),
  n("s2", ["b"]),
  n("a", ["b"]),
  n("b", ["t"]),
  n("t", ["c"], { tags: ["v1"] }),
  n("c", ["r"]),
  n("r"),
];

describe("확대·축소", () => {
  it("모든 버전은 그대로", () => {
    expect(ids(simplify(history(), "all"))).toEqual(["h", "m", "s", "s2", "a", "b", "t", "c", "r"]);
  });

  it("주요 버전: 합치기·갈라진 곳·이름표·지금 위치·첫 버전만, 나머지는 접힌 구간", () => {
    const out = simplify(history(), "key");
    expect(ids(out)).toEqual(["h", "m", "[2]", "[1]", "b", "t", "[1]", "r"]);
    connected(history(), out);
    // 접힌 곁가지는 합치기에서 갈라져 갈라진 곳에 합류한다.
    const side = out.find((x) => x.kind === "fold" && x.statOids.includes("s"))!;
    expect(side.parents).toEqual(["b"]);
    expect(out.find((x) => x.id === "m")!.parents).toContain(side.id);
  });

  it("태그만: 태그·브랜치 끝·지금 위치·첫 버전", () => {
    const out = simplify(history(), "tags");
    expect(out.filter((x) => x.kind === "commit").map((x) => x.id)).toEqual(["h", "t", "r"]);
    connected(history(), out);
  });

  it("태그 없는 기록의 태그만 단계", () => {
    const nodes = [n("h", ["b"], { isHead: true }), n("b", ["c"]), n("c", ["r"]), n("r")];
    const out = simplify(nodes, "tags");
    expect(ids(out)).toEqual(["h", "[2]", "r"]);
    expect(out[1]!.statOids).toEqual(["b", "c"]);
    connected(nodes, out);
  });

  it("펼친 구간만 펼친다", () => {
    const folded = simplify(history(), "key");
    const side = folded.find((x) => x.kind === "fold" && x.count === 2)!;
    const out = simplify(history(), "key", new Set([side.id]));
    expect(ids(out)).toEqual(["h", "m", "s", "s2", "[1]", "b", "t", "[1]", "r"]);
    connected(history(), out);
  });

  it("쪽 끝(부모가 받은 범위 밖)은 남긴다", () => {
    const nodes = [n("h", ["b"], { isHead: true }), n("b", ["c"]), n("c", ["밖"])];
    expect(ids(simplify(nodes, "tags"))).toEqual(["h", "[1]", "c"]);
  });

  it("1만 행 세 단계", () => {
    const nodes: RouteNode[] = [];
    for (let i = 0; i < 10_000; i++) {
      const over = i === 0 ? { isHead: true } : i % 500 === 0 ? { tags: [`v${i}`] } : {};
      const parents = i === 9_999 ? [] : i % 97 === 0 && i + 5 < 10_000 ? [`c${i + 1}`, `c${i + 5}`] : [`c${i + 1}`];
      nodes.push(n(`c${i}`, parents, over));
    }
    const start = performance.now();
    for (const zoom of ["all", "key", "tags"] as const) connected(nodes, simplify(nodes, zoom));
    expect(performance.now() - start).toBeLessThan(2000);
  });
});
