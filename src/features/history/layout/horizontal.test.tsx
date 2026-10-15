import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { HomeMapView, MapCommit } from "../../../lib/api";
import { branchPalette } from "../render/palette";
import { SvgRenderer } from "../render/SvgRenderer";
import { textWidth } from "./geometry";
import { homeToRouteNodes, layoutHorizontal, MINI_HEIGHT } from "./horizontal";

const c = (n: number, summary = `버전 ${n}`): MapCommit => ({ oid: `${n}`.padStart(40, "0"), shortOid: `${n}`, summary });
const labels = { here: "지금 여기", changed: (n: number) => `고친 파일 ${n}개` };
const view = (over: Partial<HomeMapView> = {}): HomeMapView => ({
  current: "로그인-기능",
  defaultBranch: "main",
  twoLines: true,
  currentCommits: [c(1), c(2)],
  currentMore: 0,
  mainSince: 1,
  mainLast: c(9),
  upstreamOid: null,
  upstreamLabel: null,
  changedFiles: 3,
  ...over,
});
const home = (v: HomeMapView, width = 900) => layoutHorizontal(v, width, "home", labels);

describe("홈 두 줄 노선도(같은 배치 함수, D-090)", () => {
  it("두 줄: main 선과 갈라진 선, 지금 여기 점선 원", () => {
    const s = home(view());
    expect(new Set(s.lines.map((l) => l.color))).toEqual(new Set(["b:main", "b:로그인-기능"]));
    expect(s.lines.filter((l) => l.dashed)).toHaveLength(1);
    expect(s.stations.filter((x) => x.color === "b:로그인-기능" && x.kind === "commit")).toHaveLength(2);
    expect(s.stations.some((x) => x.kind === "open")).toBe(true);
    expect(s.texts.map((t) => t.text)).toEqual(expect.arrayContaining(["main", "지금 여기", "고친 파일 3개", "버전 1"]));
    // 기본 브랜치 줄은 3, 지금 줄은 4.
    expect(new Set(s.lines.map((l) => `${l.color}:${l.width}`))).toEqual(new Set(["b:main:3", "b:로그인-기능:4"]));
  });

  it("한 줄: 고친 파일이 없으면 점선 없이 마지막 역이 지금 여기", () => {
    const s = home(view({ twoLines: false, current: "main", changedFiles: 0, currentCommits: [c(1), c(2), c(3)] }));
    expect(new Set(s.lines.map((l) => l.color))).toEqual(new Set(["b:main"]));
    expect(s.stations.some((x) => x.kind === "open")).toBe(false);
    expect(s.texts.find((t) => t.text === "지금 여기")).toBeTruthy();
  });

  it("너비가 모자라면 앞쪽 역을 접는다", () => {
    const s = home(view({ currentCommits: [c(1), c(2), c(3)] }), 420);
    expect(s.folded).toBeGreaterThan(0);
    expect(s.texts.find((t) => t.text === "…")).toBeTruthy();
  });

  it("upstream이 가리키는 역에 표시를 단다", () => {
    expect(home(view({ upstreamOid: c(1).oid, upstreamLabel: "GitHub" })).tags[0]?.text).toBe("GitHub");
  });

  it("첫 역 이름이 왼쪽 끝 밖으로 나가지 않는다(한 줄 보기)", () => {
    const long = "가".repeat(14);
    for (const width of [420, 800, 1200]) {
      for (const changedFiles of [0, 2]) {
        const s = home(view({ twoLines: false, current: "main", changedFiles, currentCommits: [c(1, long), c(2)] }), width);
        const name = s.texts.find((t) => t.text.startsWith("가"))!;
        expect(name.x - textWidth(name.text, 13) / 2, `${width}`).toBeGreaterThanOrEqual(8);
      }
    }
  });

  it("이웃 역 이름이 겹치지 않는다", () => {
    const long = "로그인 화면을 다시 만들고 버튼 정리";
    const s = home(view({ currentCommits: [c(1, long), c(2, long), c(3, long)] }), 800);
    const names = s.texts.filter((t) => t.text.startsWith("로그인")).sort((a, b) => a.x - b.x);
    for (let i = 1; i < names.length; i++) {
      const left = names[i - 1]!;
      const right = names[i]!;
      expect(left.x + textWidth(left.text, 13) / 2).toBeLessThanOrEqual(right.x - textWidth(right.text, 13) / 2);
    }
  });

  it("요약을 노선도 노드로: 지금 줄은 갈라진 곳에서 나온다", () => {
    const nodes = homeToRouteNodes(view(), [c(1), c(2)], 0, true);
    expect(nodes.map((n) => n.id.slice(0, 6))).toEqual(["~now", "000000", "000000", "~main", "~fork", "~start"]);
    expect(nodes.find((n) => n.id === c(1).oid)!.parents).toEqual(["~fork"]);
  });

  it("그리기: 글자에는 브랜치 색을 쓰지 않는다(D-031)", () => {
    const { container } = render(<SvgRenderer scene={home(view())} palette={branchPalette("main")} label="버전 기록" />);
    const texts = [...container.querySelectorAll("text")];
    expect(texts.length).toBeGreaterThan(0);
    for (const t of texts) expect(t.getAttribute("fill") ?? "").not.toMatch(/graph/);
    expect([...container.querySelectorAll("path")].some((p) => p.getAttribute("stroke") === "var(--graph-1)")).toBe(true);
  });
});

describe("시작 화면 노선도", () => {
  it("큰 노선도: 기본 브랜치가 아래, 갈라진 브랜치와 이름이 위", () => {
    const s = layoutHorizontal(view(), 900, "preview", labels);
    const mainY = s.stations.find((x) => x.color === "b:main")!.y;
    const currentY = s.stations.find((x) => x.color === "b:로그인-기능")!.y;
    expect(mainY).toBeGreaterThan(currentY);
    const name = s.texts.find((t) => t.text === "버전 1")!;
    expect(name.y).toBeLessThan(currentY);
  });

  it("작은 노선도: 이름 없이 가는 선, 고정 높이", () => {
    const s = layoutHorizontal(view(), 240, "mini", labels);
    expect(s.height).toBe(MINI_HEIGHT);
    expect(s.texts).toHaveLength(0);
    expect(s.lines.every((l) => l.width === 2)).toBe(true);
    expect(Math.max(...s.stations.map((x) => x.x))).toBeLessThanOrEqual(240);
  });
});
