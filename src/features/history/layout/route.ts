// 노선도 장면 만들기(D-089, D-090, D-091). 비싼 쪽(buildRoute: 레인 배치)과 싼 쪽(장면: 좌표·굵기)을
// 나눠, 줄 수가 도착하면 장면만 다시 만든다. DOM 없음.
import type { RouteNode } from "../model";
import { assignLanes, type LaneLayout, type LaneOptions } from "./lanes";
import { emptyScene, type Scene, type SceneLine } from "./scene";

export type Route = { nodes: readonly RouteNode[]; layout: LaneLayout };

export function buildRoute(nodes: readonly RouteNode[], opts: LaneOptions = {}): Route {
  return { nodes, layout: assignLanes(nodes, opts) };
}

/** 세로 노선도(버전 기록) 치수(D-091). */
export const ROW_H = 48;
export const FIRST_LANE_X = 40;
const LANE_GAP = 24;
const MIN_LANE_GAP = 8;
export const MAX_GRAPH_W = 240;
const STATION = { r: 5, stroke: 2.4 };
const MERGE_R = 7;
const FOLD_R = 4;
const RING_R = 11;
/** 태그가 붙은 역은 이만큼 크다(D-207). 지금 위치 고리보다는 작다. */
const TAG_GROW = 2;

/** 레인이 많으면 간격을 8까지 줄여 그래프 폭이 240을 넘지 않게 한다. */
export function laneGap(laneCount: number): number {
  if (laneCount <= 1) return LANE_GAP;
  const fit = (MAX_GRAPH_W - FIRST_LANE_X - 16) / (laneCount - 1);
  return Math.max(MIN_LANE_GAP, Math.min(LANE_GAP, Math.floor(fit)));
}

/** 그래프 칸 폭. */
export function graphWidth(laneCount: number): number {
  const gap = laneGap(laneCount);
  const lanes = Math.min(laneCount, Math.floor((MAX_GRAPH_W - FIRST_LANE_X - 16) / gap) + 1);
  return Math.min(MAX_GRAPH_W, FIRST_LANE_X + gap * Math.max(0, lanes - 1) + 24);
}

/** 바뀐 줄 수 → 선 굵기(D-089). 0·모름은 2, 로그 눈금, 최대 5. */
export function edgeWidth(lines: number | null | undefined): number {
  if (!lines || lines <= 0) return 2;
  return Math.min(5, Math.round(2 + Math.log10(1 + lines)));
}

/** 버전 해시 → 바뀐 줄 수. 없거나 null이면 모른다. */
export type LineStats = ReadonlyMap<string, number | null>;

/** 노드 하나의 줄 수. 접힌 구간은 아는 것의 합(하나도 모르면 null). */
function nodeLines(node: RouteNode, stats: LineStats): number | null {
  let sum: number | null = null;
  for (const oid of node.statOids) {
    const v = stats.get(oid);
    if (v !== undefined && v !== null) sum = (sum ?? 0) + v;
  }
  return sum;
}

/** 두 점을 잇는 부드러운 꺾임(한 행 안, 45도 근처). */
function bend(x1: number, y1: number, x2: number, y2: number): string {
  const h = (y2 - y1) / 2;
  return `C ${x1} ${y1 + h}, ${x2} ${y2 - h}, ${x2} ${y2}`;
}

export function verticalScene(route: Route, stats: LineStats = new Map()): Scene {
  const { nodes, layout } = route;
  const gap = laneGap(layout.laneCount);
  const width = graphWidth(layout.laneCount);
  const maxX = width - 12;
  const x = (lane: number) => Math.min(maxX, FIRST_LANE_X + lane * gap);
  const y = (row: number) => row * ROW_H + ROW_H / 2;
  const height = Math.max(nodes.length, 1) * ROW_H;
  const scene = emptyScene(width, height);
  const byId = new Map(nodes.map((n) => [n.id, n]));

  for (const e of layout.edges) {
    const child = byId.get(e.from);
    const parent = byId.get(e.to);
    // 굵기: 부모가 하나면 그 자식의 줄, 합치기의 첫 부모 선은 2, 다른 부모 선은 그 부모의 줄.
    let lines: number | null = null;
    if (child) {
      if (child.parents.length <= 1) lines = nodeLines(child, stats);
      else if (child.parents[0] !== e.to && parent) lines = nodeLines(parent, stats);
    }
    const onHead = layout.onHeadLine.has(e.from) && (e.toRow === null || layout.onHeadLine.has(e.to));
    const width = Math.min(6, edgeWidth(lines) + (onHead ? 1 : 0));
    const x1 = x(e.fromLane);
    const y1 = y(e.fromRow);
    const xt = x(e.travel);
    const x2 = x(e.toLane);
    const y2 = e.toRow === null ? height : y(e.toRow);
    let d: string;
    if (e.toRow !== null && e.toRow < e.fromRow) {
      d = `M ${x1} ${y1} L ${x2} ${y2}`;
    } else if (e.toRow !== null && e.toRow === e.fromRow + 1 && (x1 !== xt || x2 !== xt)) {
      d = `M ${x1} ${y1} ${bend(x1, y1, x2, y2)}`;
    } else {
      d = `M ${x1} ${y1}`;
      let at = y1;
      if (x1 !== xt) {
        at = y1 + ROW_H;
        d += ` ${bend(x1, y1, xt, at)}`;
      }
      const end = e.toRow !== null && x2 !== xt ? y2 - ROW_H : y2;
      if (end > at) d += ` L ${xt} ${end}`;
      if (e.toRow !== null && x2 !== xt) d += ` ${bend(xt, end, x2, y2)}`;
    }
    const line: SceneLine = { d, color: e.color, width, span: [Math.min(y1, y2), Math.max(y1, y2)] };
    scene.lines.push(line);
  }

  for (const ln of layout.nodes) {
    const node = byId.get(ln.id)!;
    const cx = x(ln.lane);
    const cy = y(ln.row);
    const kind = node.kind === "fold" ? "fold" : ln.isMerge ? "merge" : "commit";
    const tagged = kind !== "fold" && node.tags.length > 0;
    scene.stations.push({
      id: node.id,
      x: cx,
      y: cy,
      r: (kind === "fold" ? FOLD_R : kind === "merge" ? MERGE_R : STATION.r) + (tagged ? TAG_GROW : 0),
      tagged: tagged || undefined,
      stroke: STATION.stroke,
      color: ln.color,
      kind,
      ring: node.isHead ? RING_R : undefined,
      title: node.summary || undefined,
    });
  }
  return scene;
}
