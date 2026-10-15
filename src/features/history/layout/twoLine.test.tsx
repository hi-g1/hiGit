import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { HomeMapView, MapCommit } from "../../../lib/api";
import { SvgRenderer } from "../render/SvgRenderer";
import { LABEL_CHARS, layoutTwoLine, shorten } from "./twoLine";

const c = (n: number): MapCommit => ({ oid: `${n}`.padStart(40, "0"), shortOid: `${n}`, summary: `버전 ${n}` });
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

describe("두 줄 노선도 배치", () => {
  it("두 줄: main 선과 갈라진 선, 지금 여기 점선 원", () => {
    const s = layoutTwoLine(view(), 900, labels);
    expect(s.lines.map((l) => l.color)).toEqual(["main", "current", "current"]);
    expect(s.lines.map((l) => l.width)).toEqual([3, 4, 4]);
    expect(s.lines[2]!.dashed).toBe(true);
    expect(s.stations.filter((x) => x.color === "current" && x.kind === "commit")).toHaveLength(2);
    expect(s.stations.some((x) => x.kind === "open")).toBe(true);
    expect(s.texts.map((t) => t.text)).toEqual(expect.arrayContaining(["main", "지금 여기", "고친 파일 3개", "버전 1"]));
  });

  it("한 줄: 고친 파일이 없으면 점선 없이 마지막 역이 지금 여기", () => {
    const s = layoutTwoLine(view({ twoLines: false, changedFiles: 0, currentCommits: [c(1), c(2), c(3)] }), 900, labels);
    expect(s.lines.map((l) => l.color)).toEqual(["current"]);
    expect(s.stations.some((x) => x.kind === "open")).toBe(false);
    expect(s.texts.find((t) => t.text === "지금 여기")).toBeTruthy();
  });

  it("너비가 모자라면 앞쪽 역을 접는다", () => {
    const s = layoutTwoLine(view({ currentCommits: [c(1), c(2), c(3)] }), 420, labels);
    expect(s.folded).toBeGreaterThan(0);
    expect(s.texts.find((t) => t.text === "…")).toBeTruthy();
  });

  it("upstream이 가리키는 역에 표시를 단다", () => {
    const s = layoutTwoLine(view({ upstreamOid: c(1).oid, upstreamLabel: "GitHub" }), 900, labels);
    expect(s.tags[0]?.text).toBe("GitHub");
  });

  it("긴 역 이름은 줄인다", () => {
    expect(shorten("가".repeat(30))).toHaveLength(LABEL_CHARS);
    expect(shorten("짧음")).toBe("짧음");
  });

  it("그리기: 글자에는 브랜치 색을 쓰지 않는다(D-031)", () => {
    const scene = layoutTwoLine(view(), 900, labels);
    const { container } = render(
      <SvgRenderer
        scene={scene}
        palette={(key) => (key === "main" ? "var(--graph-1)" : "var(--graph-3)")}
        label="버전 기록"
      />,
    );
    const texts = [...container.querySelectorAll("text")];
    expect(texts.length).toBeGreaterThan(0);
    for (const t of texts) expect(t.getAttribute("fill") ?? "").not.toMatch(/graph/);
    expect([...container.querySelectorAll("path")].some((p) => p.getAttribute("stroke") === "var(--graph-3)")).toBe(true);
  });
});
