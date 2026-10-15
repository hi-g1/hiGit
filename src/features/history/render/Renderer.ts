// 노선도를 그리는 쪽의 약속(architecture.md "버전 기록 그리기 계층"). 배치(layout)는 좌표만 만들고,
// 그리기는 이 인터페이스 뒤에 둔다. 지금은 SVG, 역이 아주 많아지면 Canvas로 바꿀 수 있다.
import type { ReactElement } from "react";
import type { MapScene } from "../layout/twoLine";

export type Renderer = (props: {
  scene: MapScene;
  /** 선 색. 브랜치 색 함수(--graph-*)에서 온다. 글자에는 쓰지 않는다(D-031). */
  colors: { main: string; current: string };
  label: string;
}) => ReactElement;
