import { describe, expect, it } from "vitest";
import { commitNode, type RouteNode } from "../model";
import { assignLanes, type LaneLayout } from "./lanes";

const n = (id: string, parents: string[] = [], over: Partial<RouteNode> = {}) => commitNode(id, parents, over);
const lanes = (l: LaneLayout) => Object.fromEntries(l.nodes.map((x) => [x.id, x.lane]));
const edge = (l: LaneLayout, from: string, to: string) => l.edges.find((e) => e.from === from && e.to === to);

describe("레인 배치", () => {
  it("일직선은 한 레인", () => {
    const l = assignLanes([n("c", ["b"]), n("b", ["a"]), n("a")]);
    expect(lanes(l)).toEqual({ c: 0, b: 0, a: 0 });
    expect(l.edges).toHaveLength(2);
    expect(l.laneCount).toBe(1);
  });

  it("갈라졌다 합침: 합친 곁가지는 합치기 아래에서 갈라지고 갈라진 곳에서 합류한다", () => {
    const l = assignLanes([n("m", ["c", "s"]), n("s", ["b"]), n("c", ["b"]), n("b", ["a"]), n("a")]);
    expect(lanes(l)).toEqual({ m: 0, s: 1, c: 0, b: 0, a: 0 });
    expect(edge(l, "m", "s")).toMatchObject({ fromLane: 0, travel: 1, toLane: 1 });
    expect(edge(l, "s", "b")).toMatchObject({ fromLane: 1, travel: 1, toLane: 0, toRow: 3 });
    expect(l.nodes.find((x) => x.id === "m")!.isMerge).toBe(true);
  });

  it("끝점 둘: 기본 브랜치가 0번, 곁가지는 갈라진 곳 바로 위에서 꺾인다", () => {
    const l = assignLanes([n("f", ["b"]), n("t", ["b"]), n("b")], { trunkId: "t", trunkName: "main" });
    expect(lanes(l)).toEqual({ f: 1, t: 0, b: 0 });
    expect(edge(l, "f", "b")).toMatchObject({ travel: 1, toLane: 0 });
    expect(l.nodes.find((x) => x.id === "t")!.color).toBe("b:main");
  });

  it("세 부모 합치기", () => {
    const l = assignLanes([n("m", ["a", "b", "c"]), n("c", ["r"]), n("b", ["r"]), n("a", ["r"]), n("r")]);
    expect(new Set(Object.values(lanes(l)))).toEqual(new Set([0, 1, 2]));
    expect(lanes(l).r).toBe(0);
    expect(l.laneCount).toBe(3);
  });

  it("기본 브랜치의 조상인 끝점은 같은 레인에 놓인다", () => {
    const l = assignLanes([n("t", ["x"], { branches: ["main"] }), n("x", ["a"], { branches: ["옛 기능"] }), n("a")], {
      trunkId: "t",
      trunkName: "main",
    });
    expect(lanes(l)).toEqual({ t: 0, x: 0, a: 0 });
    expect(l.nodes.every((x) => x.color === "b:main")).toBe(true);
  });

  it("레인 색: 지금 브랜치 > 기본 브랜치 > 이름순, 이름이 없으면 레인을 여는 버전", () => {
    const named = assignLanes([n("f", ["a"], { branches: ["나", "가", "지금"] }), n("a")], { headBranch: "지금" });
    expect(named.nodes[0]!.color).toBe("b:지금");
    const alpha = assignLanes([n("f", ["a"], { branches: ["나", "가"] }), n("a")]);
    expect(alpha.nodes[0]!.color).toBe("b:가");
    const unnamed = assignLanes([n("f", ["a"]), n("a")]);
    expect(unnamed.nodes[0]!.color).toBe("u:f");
    // 합친 곁가지(지운 브랜치)는 그 레인에 처음 놓인 버전으로 정한다.
    const merged = assignLanes([n("m", ["c", "s"]), n("s", ["b"]), n("c", ["b"]), n("b")]);
    expect(merged.nodes.find((x) => x.id === "s")!.color).toBe("u:s");
    expect(edge(merged, "m", "s")!.color).toBe("u:s");
  });

  it("받은 범위 밖의 부모는 아래 끝까지 이어진다", () => {
    const l = assignLanes([n("b", ["a"]), n("a", ["밖"])]);
    expect(edge(l, "a", "밖")).toMatchObject({ toRow: null, travel: 0 });
  });

  it("부모가 먼저 나온 입력(어긋남)에도 깨지지 않는다", () => {
    const l = assignLanes([n("a"), n("b", ["a"])]);
    expect(edge(l, "b", "a")).toMatchObject({ fromRow: 1, toRow: 0 });
  });

  it("닫힌 칸은 다음 끝점이 다시 쓴다", () => {
    const l = assignLanes([n("m", ["c", "s"]), n("s", ["c"]), n("c", ["b"]), n("t2", ["b"]), n("b")]);
    // s의 레인(1)은 c에서 닫히고 t2가 그 칸을 다시 연다.
    expect(lanes(l).t2).toBe(1);
    expect(l.laneCount).toBe(2);
  });

  it("0번 끝점이 받은 범위에 없으면 첫 노드가 0번", () => {
    const l = assignLanes([n("f", ["a"]), n("a")], { trunkId: "없음", trunkName: "main" });
    expect(lanes(l)).toEqual({ f: 0, a: 0 });
  });

  it("레인 상한을 넘으면 마지막 레인에 겹쳐 그린다", () => {
    const l = assignLanes([n("x", ["r"]), n("y", ["r"]), n("z", ["r"]), n("r")], { maxLanes: 2 });
    expect(Math.max(...Object.values(lanes(l)))).toBe(1);
    expect(l.laneCount).toBe(2);
  });

  it("지금 브랜치 선은 지금 위치에서 첫 부모를 따라간다", () => {
    const l = assignLanes([n("f", ["b"]), n("h", ["b"], { isHead: true }), n("b", ["a"]), n("a")]);
    expect([...l.onHeadLine].sort()).toEqual(["a", "b", "h"]);
    const merged = assignLanes([n("h", ["a", "s"], { isHead: true }), n("s", ["a"]), n("a")]);
    expect([...merged.onHeadLine].sort()).toEqual(["a", "h"]);
  });

  it("1만 행도 빨리 배치한다", () => {
    const nodes: RouteNode[] = [];
    for (let i = 0; i < 10_000; i++) {
      const parents = i === 9_999 ? [] : i % 50 === 0 && i + 7 < 10_000 ? [`c${i + 1}`, `c${i + 7}`] : [`c${i + 1}`];
      nodes.push(n(`c${i}`, parents));
    }
    const start = performance.now();
    const l = assignLanes(nodes);
    const ms = performance.now() - start;
    expect(l.nodes).toHaveLength(10_000);
    expect(ms).toBeLessThan(500);
  });
});
