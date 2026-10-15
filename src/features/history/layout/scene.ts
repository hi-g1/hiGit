// 노선도 장면(architecture.md "버전 기록 그리기 계층", D-090). 배치가 만든 픽셀 좌표만 갖고 방향을
// 모른다. 버전 기록(세로)·홈·시작 화면(가로)이 같은 장면을 같은 Renderer로 그린다.

/** 색 이름. 그리는 쪽이 palette로 CSS 색(`var(--graph-n)` 등)으로 바꾼다. 글자에는 쓰지 않는다(D-031). */
export type ColorKey = string;

export type SceneLine = {
  /** SVG 경로. */
  d: string;
  color: ColorKey;
  width: number;
  dashed?: boolean;
  /** 이 선이 걸친 y 범위. 세로 노선도가 보이는 구간만 그릴 때 쓴다. */
  span?: readonly [number, number];
};

export type StationKind = "commit" | "merge" | "fold" | "open";

export type SceneStation = {
  /** 고를 수 있는 역의 id(버전 해시, 접힌 구간 id). */
  id?: string;
  x: number;
  y: number;
  r: number;
  /** 테두리 굵기. */
  stroke: number;
  color: ColorKey;
  kind: StationKind;
  /** 지금 위치 바깥 고리의 반지름(r-log). */
  ring?: number;
  /** 마우스를 올리면 보이는 이름(저장소 글자). */
  title?: string;
};

export type SceneText = {
  x: number;
  y: number;
  text: string;
  tone: "text" | "muted";
  anchor: "start" | "middle" | "end";
};

/** 역 위의 작은 표(예: upstream "GitHub"). */
export type SceneTag = { x: number; y: number; text: string };

export type Scene = {
  width: number;
  height: number;
  lines: SceneLine[];
  stations: SceneStation[];
  texts: SceneText[];
  tags: SceneTag[];
  /** 접어서 그리지 않은 버전 수. */
  folded: number;
};

export function emptyScene(width: number, height: number): Scene {
  return { width, height, lines: [], stations: [], texts: [], tags: [], folded: 0 };
}
