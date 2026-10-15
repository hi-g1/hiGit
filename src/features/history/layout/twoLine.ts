// 홈 두 줄 노선도 배치(D-046). 데이터(HomeMapView) → 장면(좌표). 그리기는 Renderer가 한다
// (architecture.md "버전 기록 그리기 계층"). 화면 없이 시험할 수 있게 순수 함수로 둔다.
import type { HomeMapView } from "../../../lib/api";
import { emptyScene, type Scene } from "./scene";

export const MAP_HEIGHT = 104;
const LEFT = 24;
const MAIN_Y = 36;
const CURRENT_Y = 76;
const SINGLE_Y = 52;
const STEP = 120;
const MIN_STEP = 84;
const CURVE = 48;
/** 역 이름 글자 수 한도. 넘치면 "…"로 줄인다. */
export const LABEL_CHARS = 14;

/** 선 색 이름: 기본 브랜치 줄과 지금 브랜치 줄. */
const MAIN = "main";
const CURRENT = "current";
const MAIN_WIDTH = 3;
const CURRENT_WIDTH = 4;
const STATION = { r: 6, stroke: 2.5 };
const OPEN_R = 9;

export function shorten(label: string): string {
  return label.length > LABEL_CHARS ? `${label.slice(0, LABEL_CHARS - 1)}…` : label;
}

export type Labels = { here: string; changed: (n: number) => string };

/** 너비에 맞게 역을 놓는다. 역이 많아 모자라면 앞쪽 역을 "…"로 접는다. */
export function layoutTwoLine(view: HomeMapView, width: number, labels: Labels): Scene {
  const scene = emptyScene(width, MAP_HEIGHT);
  scene.folded = view.currentMore;
  if (view.currentCommits.length === 0) return scene;

  const y = view.twoLines ? CURRENT_Y : SINGLE_Y;
  const forkX = LEFT + STEP;
  let start = view.twoLines ? forkX + CURVE + STEP / 2 : LEFT + (view.currentMore > 0 ? STEP / 2 : 0);
  const hasNow = view.changedFiles > 0;
  // 오른쪽 여백: 마지막 역 이름과 "고친 파일 N개"가 들어갈 자리.
  const available = width - start - (hasNow ? STEP : 0) - 72;
  let commits = view.currentCommits;
  let step = commits.length > 1 ? Math.min(STEP, available / (commits.length - 1)) : STEP;
  while (commits.length > 1 && step < MIN_STEP) {
    commits = commits.slice(1);
    scene.folded += 1;
    step = commits.length > 1 ? Math.min(STEP, available / (commits.length - 1)) : STEP;
  }
  if (!view.twoLines && scene.folded > 0 && start === LEFT) start = LEFT + STEP / 2;

  if (view.twoLines) {
    const mainEnd = Math.max(forkX + STEP, Math.min(width - 80, start + step * (commits.length - 1)));
    scene.lines.push({ color: MAIN, width: MAIN_WIDTH, d: `M ${LEFT} ${MAIN_Y} H ${mainEnd}` });
    const mainStation = (x: number) => ({ x, y: MAIN_Y, color: MAIN, kind: "commit" as const, ...STATION });
    scene.stations.push(mainStation(LEFT), mainStation(forkX));
    if (view.mainSince > 0) scene.stations.push(mainStation(mainEnd));
    if (view.defaultBranch) {
      scene.texts.push({ x: mainEnd + 10, y: MAIN_Y + 4, text: view.defaultBranch, tone: "muted", anchor: "start" });
    }
    scene.lines.push({
      color: CURRENT,
      width: CURRENT_WIDTH,
      d: `M ${forkX} ${MAIN_Y} C ${forkX + CURVE / 2} ${MAIN_Y}, ${forkX + CURVE / 2} ${y}, ${forkX + CURVE} ${y} H ${start}`,
    });
  }

  const xs = commits.map((_, i) => start + i * step);
  const lastX = xs[xs.length - 1]!;
  if (!view.twoLines) {
    scene.lines.push({ color: CURRENT, width: CURRENT_WIDTH, d: `M ${LEFT} ${y} H ${lastX}` });
  }
  if (scene.folded > 0) {
    scene.texts.push({ x: start - STEP / 4, y: y + 4, text: "…", tone: "muted", anchor: "middle" });
  }
  commits.forEach((c, i) => {
    const x = xs[i]!;
    scene.stations.push({ id: c.oid, x, y, color: CURRENT, kind: "commit", title: c.summary, ...STATION });
    scene.texts.push({ x, y: y + 22, text: shorten(c.summary), tone: "text", anchor: "middle" });
    if (view.upstreamOid === c.oid && view.upstreamLabel) {
      scene.tags = [{ x, y: y - 26, text: view.upstreamLabel }];
    }
  });

  if (hasNow) {
    const nowX = lastX + STEP;
    scene.lines.push({ color: CURRENT, width: CURRENT_WIDTH, d: `M ${lastX} ${y} H ${nowX}`, dashed: true });
    scene.stations.push({ x: nowX, y, r: OPEN_R, stroke: STATION.stroke, color: CURRENT, kind: "open" });
    scene.texts.push(
      { x: nowX, y: y - 16, text: labels.here, tone: "text", anchor: "middle" },
      { x: nowX, y: y + 22, text: labels.changed(view.changedFiles), tone: "muted", anchor: "middle" },
    );
  } else {
    scene.texts.push({ x: lastX, y: y - 16, text: labels.here, tone: "text", anchor: "middle" });
  }
  return scene;
}
