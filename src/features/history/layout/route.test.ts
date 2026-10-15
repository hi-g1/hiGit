import { describe, expect, it } from "vitest";
import { commitNode, type RouteNode } from "../model";
import { branchPalette, monoPalette } from "../render/palette";
import { hitTest, rowAt } from "./hit";
import { buildRoute, edgeWidth, FIRST_LANE_X, graphWidth, MAX_GRAPH_W, ROW_H, verticalScene } from "./route";

const n = (id: string, parents: string[] = [], over: Partial<RouteNode> = {}) => commitNode(id, parents, over);

// r-log 모양: main(h) 위에 곁가지 f, 합치기 m이 곁가지 s를 합침.
const sample = () => [
  n("f", ["h"], { branches: ["기능"] }),
  n("h", ["m"], { branches: ["main"], isHead: true }),
  n("m", ["a", "s"]),
  n("s", ["b"]),
  n("a", ["b"]),
  n("b"),
];

describe("세로 노선도 장면", () => {
  it("레인 x = 40 + 24n, 행 가운데 y = 48i + 24", () => {
    const scene = verticalScene(buildRoute(sample(), { trunkId: "h", trunkName: "main", headBranch: "main" }));
    const at = Object.fromEntries(scene.stations.map((s) => [s.id, [s.x, s.y]]));
    expect(at.h).toEqual([FIRST_LANE_X, ROW_H + ROW_H / 2]);
    expect(at.f).toEqual([FIRST_LANE_X + 24, ROW_H / 2]);
    expect(at.b).toEqual([FIRST_LANE_X, 5 * ROW_H + ROW_H / 2]);
    expect(scene.height).toBe(6 * ROW_H);
  });

  it("곁가지는 합류하는 역 바로 위 한 행 안에서 꺾인다", () => {
    const route = buildRoute(sample(), { trunkId: "h", trunkName: "main" });
    const scene = verticalScene(route);
    const fToH = scene.lines.find((l) => l.span?.[0] === ROW_H / 2 && l.span?.[1] === ROW_H + ROW_H / 2)!;
    // 한 행 차이라 꺾임 하나로 바로 잇는다.
    expect(fToH.d).toBe(`M 64 24 C 64 48, 40 48, 40 72`);
    // 합친 곁가지는 합치기 바로 아래에서 갈라져 나와 아래로 간다.
    const mToS = route.layout.edges.find((e) => e.from === "m" && e.to === "s")!;
    expect(mToS).toMatchObject({ fromLane: 0, travel: 1, toLane: 1 });
  });

  it("합치기는 큰 점, 지금 위치는 바깥 고리, 접힌 구간은 마름모", () => {
    const nodes = [...sample().slice(0, 5), { ...n("fold", ["b"]), kind: "fold" as const, statOids: ["x", "y"] }, n("b")];
    nodes[4] = n("a", ["fold"]);
    const scene = verticalScene(buildRoute(nodes, { trunkId: "h" }));
    const by = Object.fromEntries(scene.stations.map((s) => [s.id, s]));
    expect(by.m!.kind).toBe("merge");
    expect(by.m!.r).toBeGreaterThan(by.a!.r);
    expect(by.h!.ring).toBe(11);
    expect(by.fold!.kind).toBe("fold");
  });

  it("굵기: 로그 눈금 2~5, 모름 2", () => {
    expect([0, 9, 10, 99, 100, 999, 1000, 100_000].map(edgeWidth)).toEqual([2, 3, 3, 4, 4, 5, 5, 5]);
    expect(edgeWidth(null)).toBe(2);
    expect(edgeWidth(undefined)).toBe(2);
  });

  it("지금 브랜치 선은 한 칸 더, 합치기의 첫 부모 선은 2, 접힌 구간은 합", () => {
    const nodes = [
      n("h", ["m"], { isHead: true }),
      n("m", ["a", "s"]),
      n("s", ["b"]),
      { ...n("fold", ["b"]), kind: "fold" as const, statOids: ["x", "y"] },
      n("b"),
    ];
    nodes[1] = n("m", ["fold", "s"]);
    const stats = new Map<string, number | null>([
      ["h", 1000],
      ["s", 50],
      ["x", 5],
      ["y", 4],
      ["b", 0],
    ]);
    const route = buildRoute(nodes);
    const scene = verticalScene(route, stats);
    const widthOf = (from: string, to: string) => {
      const i = route.layout.edges.findIndex((e) => e.from === from && e.to === to);
      return scene.lines[i]!.width;
    };
    expect(widthOf("h", "m")).toBe(6); // 1000줄 → 5, 지금 브랜치 +1
    expect(widthOf("m", "fold")).toBe(3); // 첫 부모 2 + 지금 브랜치 1
    expect(widthOf("m", "s")).toBe(4); // 다른 부모: 그 부모의 줄(50 → 4)
    expect(widthOf("fold", "b")).toBe(4); // 접힌 구간 9줄 → 3, 지금 브랜치 +1
  });

  it("레인이 많아도 그래프 폭은 240을 넘지 않는다", () => {
    expect(graphWidth(1)).toBeLessThanOrEqual(MAX_GRAPH_W);
    expect(graphWidth(40)).toBeLessThanOrEqual(MAX_GRAPH_W);
    const nodes = Array.from({ length: 40 }, (_, i) => n(`t${i}`, ["r"]));
    const scene = verticalScene(buildRoute([...nodes, n("r")]));
    expect(Math.max(...scene.stations.map((s) => s.x))).toBeLessThanOrEqual(MAX_GRAPH_W);
  });

  it("팔레트: 브랜치 색과 회색", () => {
    const branch = branchPalette("main");
    expect(branch("b:main")).toBe("var(--graph-1)");
    expect(branch("b:기능")).toMatch(/^var\(--graph-[2-6]\)$/);
    expect(branch("u:abc")).toMatch(/^var\(--graph-[2-6]\)$/);
    expect(branch("u:abc")).toBe(branch("u:abc"));
    const mono = monoPalette("main");
    expect(mono("b:main")).toBe("var(--border-strong)");
    expect(mono("b:기능")).toBe("var(--text-muted)");
  });

  it("판정은 장면 좌표로", () => {
    const scene = verticalScene(buildRoute(sample(), { trunkId: "h" }));
    expect(hitTest(scene, FIRST_LANE_X + 3, ROW_H + ROW_H / 2)).toBe("h");
    expect(hitTest(scene, FIRST_LANE_X + 30, ROW_H + ROW_H / 2, 2)).toBeNull();
    expect(rowAt(ROW_H * 2 + 1, ROW_H, 6)).toBe(2);
    expect(rowAt(-1, ROW_H, 6)).toBeNull();
    expect(rowAt(ROW_H * 6, ROW_H, 6)).toBeNull();
  });

  it("1만 행", () => {
    const nodes: RouteNode[] = [];
    for (let i = 0; i < 10_000; i++) {
      const parents = i === 9_999 ? [] : i % 40 === 0 && i + 9 < 10_000 ? [`c${i + 1}`, `c${i + 9}`] : [`c${i + 1}`];
      nodes.push(n(`c${i}`, parents));
    }
    let start = performance.now();
    const route = buildRoute(nodes);
    const build = performance.now() - start;
    start = performance.now();
    verticalScene(route);
    const scene = performance.now() - start;
    expect(build).toBeLessThan(500);
    expect(scene).toBeLessThan(500);
  });
});
