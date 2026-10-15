// 가로 노선도(홈 두 줄 노선도, 시작 화면 큰·작은 노선도, D-090, D-093). 요약(HomeMapView)을 노선도
// 노드로 바꾸고 버전 기록과 같은 레인 배치(buildRoute)로 레인을 정한 뒤 가로로 놓는다. DOM 없음.
import type { HomeMapView } from "../../../lib/api";
import { commitNode, type RouteNode } from "../model";
import { fitText, textWidth } from "./geometry";
import { branchKey } from "./lanes";
import { buildRoute } from "./route";
import { emptyScene, type Scene, type SceneStation } from "./scene";

export type Density = "home" | "preview" | "mini";
export type HorizontalLabels = { here: string; changed: (n: number) => string };

/** 높이: 홈·큰 노선도 104, 작은 노선도 28. */
export const MAP_HEIGHT = 104;
export const MINI_HEIGHT = 28;

const FONT = 13;
const LEFT = 24;
const STEP = 120;
const MIN_STEP = 84;
const CURVE = 48;

type Spec = {
  height: number;
  /** 기본 브랜치 줄, 지금 줄, 한 줄일 때의 y. */
  mainY: number;
  currentY: number;
  singleY: number;
  /** 역 이름을 지금 줄의 위(above)·아래(below)에. 없으면 이름 없음. */
  names: "above" | "below" | null;
  widths: { main: number; current: number };
  station: { r: number; stroke: number };
};

const SPECS: Record<Density, Spec> = {
  // 홈: 기본 브랜치가 위(r-home.png).
  home: {
    height: MAP_HEIGHT,
    mainY: 36,
    currentY: 76,
    singleY: 52,
    names: "below",
    widths: { main: 3, current: 4 },
    station: { r: 6, stroke: 2.5 },
  },
  // 시작 화면 큰 노선도: 기본 브랜치가 아래, 갈라진 브랜치가 위(start-normal.png).
  preview: {
    height: MAP_HEIGHT,
    mainY: 76,
    currentY: 36,
    singleY: 52,
    names: "above",
    widths: { main: 3, current: 4 },
    station: { r: 6, stroke: 2.5 },
  },
  // 작은 노선도: 이름 없이 회색 줄(start-normal.png).
  mini: {
    height: MINI_HEIGHT,
    mainY: 20,
    currentY: 8,
    singleY: 14,
    names: null,
    widths: { main: 2, current: 2 },
    station: { r: 3.5, stroke: 1.5 },
  },
};

const START = "~start";
const FORK = "~fork";
const MAIN_END = "~main";
const FOLD = "~fold";
const NOW = "~now";

/**
 * 요약을 노선도 노드로(최신이 위). 두 줄이면 기본 브랜치 줄은 시작 → 갈라진 곳 → 끝, 지금 줄은
 * 갈라진 곳에서 나온다. 앞쪽에서 접은 버전은 접힌 구간, 고친 파일은 아직 버전이 아닌 역(open).
 */
export function homeToRouteNodes(view: HomeMapView, shown: HomeMapView["currentCommits"], folded: number, withNow: boolean): RouteNode[] {
  const current = view.current ? [view.current] : [];
  const main = view.defaultBranch ? [view.defaultBranch] : [];
  const nodes: RouteNode[] = [];
  const firstParent = folded > 0 ? FOLD : view.twoLines ? FORK : START;
  const newest = shown[shown.length - 1];
  if (withNow && newest) nodes.push({ ...commitNode(NOW, [newest.oid], { branches: current }), kind: "open" });
  for (let i = shown.length - 1; i >= 0; i--) {
    const c = shown[i]!;
    const parent = i > 0 ? shown[i - 1]!.oid : firstParent;
    nodes.push(commitNode(c.oid, [parent], { shortOid: c.shortOid, summary: c.summary, branches: i === shown.length - 1 ? current : [], isHead: i === shown.length - 1 }));
  }
  if (folded > 0) {
    nodes.push({ ...commitNode(FOLD, [view.twoLines ? FORK : START]), kind: "fold", count: folded, statOids: [] });
  }
  if (view.twoLines) {
    nodes.push(commitNode(MAIN_END, [FORK], { branches: main, summary: view.mainLast?.summary ?? "" }));
    nodes.push(commitNode(FORK, [START]));
  }
  nodes.push(commitNode(START, [], { branches: view.twoLines ? main : current }));
  return nodes;
}

/** 너비에 맞게 역을 놓는다. 역이 많아 모자라면 앞쪽 역을 "…"로 접는다(홈과 시작 화면이 같다). */
export function layoutHorizontal(view: HomeMapView, width: number, density: Density, labels: HorizontalLabels): Scene {
  const spec = SPECS[density];
  const scene = emptyScene(width, spec.height);
  if (view.currentCommits.length === 0) return scene;
  const mini = density === "mini";
  const hasNow = view.changedFiles > 0 && !mini;
  const y = view.twoLines ? spec.currentY : spec.singleY;
  const forkX = mini ? 8 + width * 0.25 : LEFT + STEP;
  const curve = mini ? 12 : CURVE;
  let folded = view.currentMore;
  let commits = view.currentCommits;

  // 첫 역 이름이 왼쪽 끝 밖으로 나가지 않게 시작 위치를 잡는다(반복 2 끝 점검).
  const nameRoom = (step: number) => step - 8;
  const firstNameHalf = (step: number) =>
    spec.names && commits[0] ? textWidth(fitText(commits[0].summary, nameRoom(step), FONT), FONT) / 2 + 8 : 0;
  const startFor = (step: number) => {
    if (mini) return view.twoLines ? forkX + curve + 8 : 8;
    if (view.twoLines) return forkX + CURVE + STEP / 2;
    const base = LEFT + (folded > 0 ? STEP / 2 : 0);
    return Math.max(base, firstNameHalf(step));
  };
  const rightRoom = mini ? 8 : (hasNow ? STEP : 0) + 72;
  const stepFor = (n: number, start: number) => {
    const available = width - start - rightRoom;
    if (mini) return n > 1 ? available / (n - 1) : 0;
    return n > 1 ? Math.min(STEP, available / (n - 1)) : STEP;
  };
  let step = stepFor(commits.length, startFor(STEP));
  while (!mini && commits.length > 1 && step < MIN_STEP) {
    commits = commits.slice(1);
    folded += 1;
    step = stepFor(commits.length, startFor(STEP));
  }
  const start = startFor(step);
  scene.folded = folded;

  const nodes = homeToRouteNodes(view, commits, folded, hasNow);
  const route = buildRoute(nodes, {
    trunkId: view.twoLines ? MAIN_END : null,
    trunkName: view.defaultBranch,
    headBranch: view.current,
    maxLanes: 2,
  });

  // 가로 위치: 요약의 규칙(갈라진 곳, 간격, 끝 맞춤). 세로 위치: 레인.
  const xs = commits.map((_, i) => start + i * step);
  const lastX = xs[xs.length - 1]!;
  const mainEnd = mini ? lastX : Math.max(forkX + STEP, Math.min(width - 80, lastX));
  const xOf = new Map<string, number>([
    [START, mini ? 8 : LEFT],
    [FORK, forkX],
    [MAIN_END, mainEnd],
    [FOLD, start - (mini ? 6 : STEP / 4)],
    [NOW, lastX + STEP],
  ]);
  commits.forEach((c, i) => xOf.set(c.oid, xs[i]!));
  const laneOf = new Map(route.layout.nodes.map((n) => [n.id, n.lane]));
  const yOf = (id: string) => (view.twoLines ? (laneOf.get(id) === 0 ? spec.mainY : spec.currentY) : spec.singleY);

  for (const e of route.layout.edges) {
    const xc = xOf.get(e.from)!;
    const xp = xOf.get(e.to)!;
    const yc = yOf(e.from);
    const yp = yOf(e.to);
    const trunk = view.twoLines && e.travel === 0;
    const d =
      yc === yp
        ? `M ${xp} ${yp} H ${xc}`
        : `M ${xp} ${yp} C ${xp + curve / 2} ${yp}, ${xp + curve / 2} ${yc}, ${xp + curve} ${yc} H ${xc}`;
    scene.lines.push({
      d,
      color: e.color,
      width: trunk ? spec.widths.main : spec.widths.current,
      dashed: e.from === NOW || undefined,
    });
  }

  const color = new Map(route.layout.nodes.map((n) => [n.id, n.color]));
  const station = (id: string, over: Partial<SceneStation> = {}): SceneStation => ({
    id,
    x: xOf.get(id)!,
    y: yOf(id),
    color: color.get(id) ?? branchKey(view.current ?? ""),
    kind: "commit",
    ...spec.station,
    ...over,
  });
  if (view.twoLines) {
    scene.stations.push(station(START), station(FORK));
    if (view.mainSince > 0) scene.stations.push(station(MAIN_END, { title: view.mainLast?.summary }));
    if (view.defaultBranch && !mini) {
      scene.texts.push({ x: mainEnd + 10, y: spec.mainY + 4, text: view.defaultBranch, tone: "muted", anchor: "start" });
    }
  }
  if (folded > 0 && !mini) scene.texts.push({ x: xOf.get(FOLD)!, y: y + 4, text: "…", tone: "muted", anchor: "middle" });
  const nameY = spec.names === "above" ? y - 16 : y + 22;
  const otherY = spec.names === "above" ? y + 22 : y - 16;
  commits.forEach((c, i) => {
    const x = xs[i]!;
    scene.stations.push(station(c.oid, { title: c.summary }));
    if (spec.names) scene.texts.push({ x, y: nameY, text: fitText(c.summary, nameRoom(step), FONT), tone: "text", anchor: "middle" });
    if (!mini && view.upstreamOid === c.oid && view.upstreamLabel) {
      scene.tags.push({ x, y: (spec.names === "above" ? y + 26 : y - 26), text: view.upstreamLabel });
    }
  });
  if (mini) return scene;
  if (hasNow) {
    const nowX = xOf.get(NOW)!;
    scene.stations.push(station(NOW, { r: 9, kind: "open" }));
    scene.texts.push(
      { x: nowX, y: otherY, text: labels.here, tone: "text", anchor: "middle" },
      { x: nowX, y: nameY, text: labels.changed(view.changedFiles), tone: "muted", anchor: "middle" },
    );
  } else {
    scene.texts.push({ x: lastX, y: otherY, text: labels.here, tone: "text", anchor: "middle" });
  }
  return scene;
}
