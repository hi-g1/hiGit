// 가로 노선도(홈 두 줄 노선도, 시작 화면 큰·작은 노선도, D-090, D-093). 요약(HomeMapView)을 노선도
// 노드로 바꾸고 버전 기록과 같은 레인 배치(buildRoute)로 레인을 정한 뒤 가로로 놓는다. DOM 없음.
import type { HomeMapView } from "../../../lib/api";
import { commitNode, type RouteNode } from "../model";
import { fitText, textWidth } from "./geometry";
import { branchKey } from "./lanes";
import { buildRoute } from "./route";
import { emptyScene, FOLD_LINE, SCENE_FONT, type Scene, type SceneStation } from "./scene";

export type Density = "home" | "preview" | "mini";
export type HorizontalLabels = {
  here: string;
  changed: (n: number) => string;
  /** 접힌 앞쪽 버전 수 글자("이전 N개", D-141). */
  earlier: (n: number) => string;
  /** 참이면(홈) "이전 N개"를 SVG 글자로 그리지 않고 `scene.fold` 좌표만 둔다(화면이 단추로 겹친다). */
  interactiveFold?: boolean;
};

/** 높이(장면 단위, D-153): 홈·큰 노선도 104, 작은 노선도 28. 화면은 배율을 곱한다. */
export const MAP_HEIGHT = 104;
export const MINI_HEIGHT = 28;

/** 노선도 글자 크기(장면 단위, 배율 1일 때 --font-size-sm과 같음). */
const FONT = SCENE_FONT;
const LEFT = 24;
const STEP = 120;
const MIN_STEP = 84;
const CURVE = 48;
/** 홈(D-155): 역은 5개까지, 간격은 이름표 폭에 맞추되 72 아래로는 줄이지 않고 앞 역을 더 접는다(R9). */
const HOME_MAX_STATIONS = 5;
const HOME_MIN_STEP = 72;
const HOME_LABEL_STEP = 96;
const HOME_MAX_LABEL = 200;
/** 홈의 "지금 여기"는 합치기 제안 블록 앞(장면 너비 − 72)에 둔다. */
const HOME_RIGHT = 72;
/** 접힌 앞쪽을 알리는 왼쪽 끝 점선 길이(D-141). */
const FOLD_DASH = 24;
/** "지금 여기" 이름표 높이와 좌우 여백. */
const TAG_PAD = 16;

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

/**
 * 너비에 맞게 역을 놓는다. 역이 많아 모자라면 앞쪽 역을 접고, 왼쪽 끝 점선과 "이전 N개"로 알린다(D-141, D-155).
 * 홈은 역 5개까지·이름표 폭 간격·오른쪽 끝 "지금 여기" 이름표, 시작 화면은 지금 간격 규칙을 쓴다.
 */
export function layoutHorizontal(view: HomeMapView, width: number, density: Density, labels: HorizontalLabels): Scene {
  const spec = SPECS[density];
  const scene = emptyScene(width, spec.height);
  if (view.currentCommits.length === 0) return scene;
  const mini = density === "mini";
  const home = density === "home";
  const hasNow = view.changedFiles > 0 && !mini;
  const y = view.twoLines ? spec.currentY : spec.singleY;
  const forkX = mini ? 8 + width * 0.25 : LEFT + STEP;
  const curve = mini ? 12 : CURVE;
  let folded = view.currentMore;
  let commits = view.currentCommits;
  if (home && commits.length > HOME_MAX_STATIONS) {
    folded += commits.length - HOME_MAX_STATIONS;
    commits = commits.slice(commits.length - HOME_MAX_STATIONS);
  }
  /** 지금 줄이 시작하는 곳(한 줄이면 왼쪽 끝, 두 줄이면 갈라진 곡선이 끝난 곳). 접힌 점선이 여기서 시작한다. */
  const lineStart = view.twoLines ? forkX + curve : LEFT;
  const foldText = (n: number) => labels.earlier(n);

  const labelWidth = (summary: string) => Math.min(HOME_MAX_LABEL, textWidth(summary, FONT));
  /** 홈의 간격: 이름표 폭(최대 200) + 16, 최소 96. */
  const homeDesired = () => Math.max(HOME_LABEL_STEP, Math.max(0, ...commits.map((c) => labelWidth(c.summary))) + TAG_PAD);
  const nameRoom = (step: number) => step - 8;
  const firstNameHalf = (step: number) =>
    spec.names && commits[0] ? textWidth(fitText(commits[0].summary, nameRoom(step), FONT), FONT) / 2 + 8 : 0;
  const startFor = (step: number) => {
    if (mini) return view.twoLines ? forkX + curve + 8 : 8;
    const base = view.twoLines ? forkX + CURVE + STEP / 2 : Math.max(LEFT, firstNameHalf(step));
    if (folded === 0) return base;
    // 점선 뒤, 그리고 첫 이름표가 "이전 N개" 글자와 겹치지 않게.
    const afterText = lineStart + textWidth(foldText(folded), FONT) + 8 + firstNameHalf(step);
    return Math.max(base, lineStart + FOLD_DASH + 16, afterText);
  };
  const nowX = home ? width - HOME_RIGHT : 0;
  /** 역을 놓을 수 있는 오른쪽 끝. */
  const endFor = () => {
    if (mini) return width - 8;
    if (home) return hasNow ? nowX - HOME_LABEL_STEP : width - HOME_RIGHT;
    return width - ((hasNow ? STEP : 0) + 72);
  };
  const stepFor = (n: number, start: number) => {
    const available = endFor() - start;
    if (mini) return n > 1 ? available / (n - 1) : 0;
    if (home) return n > 1 ? Math.min(homeDesired(), available / (n - 1)) : homeDesired();
    return n > 1 ? Math.min(STEP, available / (n - 1)) : STEP;
  };
  const minStep = home ? HOME_MIN_STEP : MIN_STEP;
  const firstStep = () => (home ? homeDesired() : STEP);
  let step = stepFor(commits.length, startFor(firstStep()));
  while (!mini && commits.length > 1 && step < minStep) {
    commits = commits.slice(1);
    folded += 1;
    step = stepFor(commits.length, startFor(firstStep()));
  }
  const start = startFor(step);
  scene.folded = folded;
  const showFold = folded > 0 && !mini;

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
    [FOLD, mini ? start - 6 : lineStart + FOLD_DASH],
    [NOW, home ? nowX : lastX + STEP],
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
    const lineWidth = trunk ? spec.widths.main : spec.widths.current;
    // 접힌 앞쪽: 곡선은 실선으로, 지금 줄 왼쪽 끝 24는 점선으로(D-141).
    if (showFold && e.from === FOLD) {
      const dashFrom = yc === yp ? xp : xp + curve;
      if (yc !== yp) {
        scene.lines.push({
          d: `M ${xp} ${yp} C ${xp + curve / 2} ${yp}, ${xp + curve / 2} ${yc}, ${xp + curve} ${yc}`,
          color: e.color,
          width: lineWidth,
        });
      }
      scene.lines.push({ d: `M ${dashFrom} ${yc} H ${xc}`, color: FOLD_LINE, width: lineWidth, dashed: true });
      continue;
    }
    const d =
      yc === yp
        ? `M ${xp} ${yp} H ${xc}`
        : `M ${xp} ${yp} C ${xp + curve / 2} ${yp}, ${xp + curve / 2} ${yc}, ${xp + curve} ${yc} H ${xc}`;
    scene.lines.push({ d, color: e.color, width: lineWidth, dashed: e.from === NOW || undefined });
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
  const nameY = spec.names === "above" ? y - 16 : y + 22;
  const otherY = spec.names === "above" ? y + 22 : y - 16;
  if (showFold) {
    scene.fold = { x: lineStart, y: nameY, count: folded };
    if (!labels.interactiveFold) {
      scene.texts.push({ x: lineStart, y: nameY, text: foldText(folded), tone: "muted", anchor: "start" });
    }
  }
  const tagWidth = (text: string) => textWidth(text, FONT) + TAG_PAD;
  /** 홈에서 "지금 여기" 이름표가 놓이는 역(고친 파일이 없으면 마지막 버전). */
  const hereOn = home ? (hasNow ? NOW : commits[commits.length - 1]!.oid) : null;
  commits.forEach((c, i) => {
    const x = xs[i]!;
    scene.stations.push(station(c.oid, { title: c.summary }));
    if (spec.names) scene.texts.push({ x, y: nameY, text: fitText(c.summary, nameRoom(step), FONT), tone: "text", anchor: "middle" });
    if (!mini && view.upstreamOid === c.oid && view.upstreamLabel) {
      const text = view.upstreamLabel;
      const ty = spec.names === "above" ? y + 26 : y - 26;
      // "지금 여기" 이름표와 같은 역이면 한 칸 위로.
      scene.tags.push({ x, y: hereOn === c.oid ? ty - 20 : ty, text, width: tagWidth(text) });
    }
  });
  if (mini) return scene;
  if (hasNow) {
    scene.stations.push(station(NOW, { r: 9, kind: "open" }));
    scene.texts.push({ x: xOf.get(NOW)!, y: nameY, text: labels.changed(view.changedFiles), tone: "muted", anchor: "middle" });
  }
  const hereX = hasNow ? xOf.get(NOW)! : lastX;
  if (home) {
    scene.tags.push({ x: hereX, y: otherY - 4, text: labels.here, width: tagWidth(labels.here) });
  } else {
    scene.texts.push({ x: hereX, y: otherY, text: labels.here, tone: "text", anchor: "middle" });
  }
  return scene;
}
