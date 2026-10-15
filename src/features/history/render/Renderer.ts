// 노선도를 그리는 쪽의 약속(architecture.md "버전 기록 그리기 계층", D-090). 배치(layout)는 장면
// 좌표만 만들고, 그리기는 이 인터페이스 뒤에 둔다. 지금은 SVG, 역이 아주 많아지면 Canvas로 바꿀 수 있다.
import type { ReactElement } from "react";
import type { ColorKey, Scene } from "../layout/scene";

/** 색 이름 → CSS 색. 선과 점에만 쓴다(D-031). */
export type Palette = (color: ColorKey) => string;

/** 세로 노선도에서 보이는 y 범위. 이 밖의 선·점·글자는 그리지 않는다. */
export type Viewport = { top: number; bottom: number };

export type Renderer = (props: {
  scene: Scene;
  palette: Palette;
  /** 스크린 리더 이름. 없으면 장식으로 숨긴다. */
  label?: string;
  viewport?: Viewport;
}) => ReactElement;
