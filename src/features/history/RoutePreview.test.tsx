import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { HomeMapView, MapCommit } from "../../lib/api";
import { useUiStore } from "../../state/uiStore";
import { MINI_HEIGHT } from "./layout/horizontal";
import { MINI_WIDTH, RoutePreview } from "./RoutePreview";

const c = (n: number): MapCommit => ({ oid: `${n}`.padStart(40, "0"), shortOid: `${n}`, summary: `버전 ${n}` });
const view: HomeMapView = {
  current: "기능",
  defaultBranch: "main",
  twoLines: true,
  currentCommits: [c(1), c(2)],
  currentMore: 0,
  mainSince: 1,
  mainLast: c(9),
  upstreamOid: null,
  upstreamLabel: null,
  changedFiles: 0,
};

describe("시작 화면 노선도 배율", () => {
  it("작은 노선도는 장면 폭 그대로 배치하고 배율만큼 크게 그린다", () => {
    useUiStore.setState({ uiScale: 1.25 });
    const { container } = render(<RoutePreview view={view} density="mini" />);
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("viewBox")).toBe(`0 0 ${MINI_WIDTH} ${MINI_HEIGHT}`);
    expect(Number(svg.getAttribute("width"))).toBe(MINI_WIDTH * 1.25);
    expect(Number(svg.getAttribute("height"))).toBe(MINI_HEIGHT * 1.25);
  });
});
