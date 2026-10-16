import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { HomeMapView, MapCommit } from "../../../lib/api";
import { branchPalette } from "../render/palette";
import { SvgRenderer } from "../render/SvgRenderer";
import { textWidth } from "./geometry";
import { homeToRouteNodes, layoutHorizontal, MINI_HEIGHT } from "./horizontal";

const c = (n: number, summary = `버전 ${n}`): MapCommit => ({ oid: `${n}`.padStart(40, "0"), shortOid: `${n}`, summary });
const labels = { here: "지금 여기", changed: (n: number) => `고친 파일 ${n}개`, earlier: (n: number) => `이전 ${n}개` };
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
    expect(s.texts.map((t) => t.text)).toEqual(expect.arrayContaining(["main", "고친 파일 3개", "버전 1"]));
    // 홈의 지금 여기는 --primary 바탕 이름표(D-141).
    expect(s.tags.map((t) => t.text)).toContain("지금 여기");
    // 기본 브랜치 줄은 3, 지금 줄은 4.
    expect(new Set(s.lines.map((l) => `${l.color}:${l.width}`))).toEqual(new Set(["b:main:3", "b:로그인-기능:4"]));
  });

  it("한 줄: 고친 파일이 없으면 점선 없이 마지막 역이 지금 여기", () => {
    const s = home(view({ twoLines: false, current: "main", changedFiles: 0, currentCommits: [c(1), c(2), c(3)] }));
    expect(new Set(s.lines.map((l) => l.color))).toEqual(new Set(["b:main"]));
    expect(s.stations.some((x) => x.kind === "open")).toBe(false);
    const here = s.tags.find((t) => t.text === "지금 여기")!;
    expect(here.x).toBe(s.stations.find((x) => x.id === c(3).oid)!.x);
  });

  it("너비가 모자라면 앞쪽 역을 접고, … 대신 왼쪽 끝 점선과 이전 N개(D-141)", () => {
    for (const twoLines of [true, false]) {
      const s = home(view({ twoLines, current: twoLines ? "로그인-기능" : "main", currentCommits: [c(1), c(2), c(3), c(4), c(5)] }), 420);
      expect(s.folded).toBeGreaterThan(0);
      expect(s.texts.find((t) => t.text === "…")).toBeUndefined();
      const dash = s.lines.find((l) => l.color === "~fold-line")!;
      expect(dash.dashed).toBe(true);
      expect(s.fold?.count).toBe(s.folded);
      expect(s.texts.find((t) => t.text === `이전 ${s.folded}개`)).toBeTruthy();
    }
  });

  it("홈은 역을 5개까지 그리고 나머지는 접는다", () => {
    const many = [1, 2, 3, 4, 5, 6, 7, 8].map((n) => c(n));
    const s = home(view({ currentCommits: many, currentMore: 2 }), 1600);
    expect(s.stations.filter((x) => x.color === "b:로그인-기능" && x.kind === "commit")).toHaveLength(5);
    expect(s.fold?.count).toBe(5);
    expect(s.texts.find((t) => t.text === "버전 3")).toBeUndefined();
    expect(s.texts.find((t) => t.text === "버전 8")).toBeTruthy();
  });

  it("홈 단추로 겹칠 때는 이전 N개를 SVG 글자로 그리지 않는다", () => {
    const s = layoutHorizontal(view({ currentCommits: [c(1), c(2), c(3)] }), 420, "home", { ...labels, interactiveFold: true });
    expect(s.fold).not.toBeNull();
    expect(s.texts.find((t) => t.text.startsWith("이전"))).toBeUndefined();
  });

  it("간격은 이름표 폭에 맞추고(최소 96, 최대 216) 72보다 좁아지면 더 접는다(R9)", () => {
    const short = home(view({ twoLines: false, current: "main", changedFiles: 0, currentCommits: [c(1, "a"), c(2, "b"), c(3, "c")] }), 1400);
    const xs = (s: ReturnType<typeof home>) =>
      s.stations.filter((x) => x.kind === "commit" && x.id?.length === 40).map((x) => x.x).sort((a, b) => a - b);
    expect(xs(short)[1]! - xs(short)[0]!).toBe(96);
    const longName = "가".repeat(30);
    const long = home(view({ twoLines: false, current: "main", changedFiles: 0, currentCommits: [c(1, longName), c(2, longName)] }), 1400);
    expect(xs(long)[1]! - xs(long)[0]!).toBe(216);
    // 배율 1.3에서 560px 창 → 장면 폭 약 430: 음수·72 미만 간격이 없고 이름표가 …만 남지 않는다.
    for (const width of [320, 430, 560]) {
      const s = home(view({ currentCommits: [c(1, "로그인 화면"), c(2, "버튼 문구"), c(3, "검색 칸"), c(4, "설정"), c(5, "오타")] }), width);
      const p = xs(s);
      for (let i = 1; i < p.length; i++) expect(p[i]! - p[i - 1]!, `${width}`).toBeGreaterThanOrEqual(72);
      for (const t of s.texts.filter((x) => x.tone === "text")) expect(t.text, `${width}`).not.toBe("…");
    }
  });

  it("지금 여기 이름표는 합치기 제안 앞(너비 − 72), 폭은 글자 폭 + 16", () => {
    const s = home(view(), 1000);
    const here = s.tags.find((t) => t.text === "지금 여기")!;
    expect(here.x).toBe(1000 - 72);
    expect(here.width).toBeCloseTo(textWidth("지금 여기", 13) + 16);
    expect(s.stations.find((x) => x.kind === "open")!.x).toBe(1000 - 72);
  });

  it("이전 N개 글자와 첫 역 이름이 겹치지 않는다", () => {
    for (const twoLines of [true, false]) {
      const s = home(view({ twoLines, current: twoLines ? "로그인-기능" : "main", currentCommits: [c(1, "아주 긴 첫 번째 버전 이름"), c(2), c(3)], currentMore: 4 }), 700);
      const fold = s.texts.find((t) => t.text.startsWith("이전"))!;
      const first = s.texts.find((t) => t.text.startsWith("아주"))!;
      expect(fold.x + textWidth(fold.text, 13)).toBeLessThanOrEqual(first.x - textWidth(first.text, 13) / 2);
    }
  });

  it("upstream 이름표가 지금 여기와 같은 역이면 한 칸 위로", () => {
    const s = home(view({ twoLines: false, current: "main", changedFiles: 0, currentCommits: [c(1), c(2)], upstreamOid: c(2).oid, upstreamLabel: "GitHub" }));
    const up = s.tags.find((t) => t.text === "GitHub")!;
    const here = s.tags.find((t) => t.text === "지금 여기")!;
    expect(up.x).toBe(here.x);
    expect(up.y).toBeLessThan(here.y - 12);
  });

  it("upstream이 가리키는 역에 표시를 단다", () => {
    expect(home(view({ upstreamOid: c(1).oid, upstreamLabel: "GitHub" })).tags[0]?.text).toBe("GitHub");
    // 이름표 바탕 폭은 글자 폭 + 16(D-153).
    const long = home(view({ upstreamOid: c(1).oid, upstreamLabel: "회사 저장소" })).tags[0]!;
    expect(long.width).toBeCloseTo(textWidth("회사 저장소", 13) + 16);
  });

  it("첫 역 이름이 왼쪽 끝 밖으로 나가지 않는다(한 줄 보기)", () => {
    const long = "가".repeat(14);
    for (const width of [420, 800, 1200]) {
      for (const changedFiles of [0, 2]) {
        const s = home(view({ twoLines: false, current: "main", changedFiles, currentCommits: [c(1, long), c(2)] }), width);
        const name = s.texts.find((t) => t.text.startsWith("가"))!;
        expect(name.x - textWidth(name.text, 12) / 2, `${width}`).toBeGreaterThanOrEqual(8);
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
      expect(left.x + textWidth(left.text, 12) / 2).toBeLessThanOrEqual(right.x - textWidth(right.text, 12) / 2);
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
