import { useEffect, useRef, useState } from "react";
import type { HomeMapView } from "../../lib/api";
import { useUiScale } from "../../lib/useUiScale";
import { useMessages } from "../../messages";
import { layoutHorizontal } from "./layout/horizontal";
import { branchPalette, monoPalette } from "./render/palette";
import { SvgRenderer } from "./render/SvgRenderer";
import styles from "./RoutePreview.module.css";

/** 큰 노선도 너비 상한(장면 단위). */
const MAX_WIDTH = 1100;
/** 작은 노선도 너비(장면 단위). CSS `.mini`·`.rowMap`의 15rem과 짝이다. */
export const MINI_WIDTH = 240;

/**
 * 시작 화면 노선도(D-090, D-093, start-normal.png). 홈과 같은 배치 함수로 그린다. 큰 것은 브랜치 색과
 * 역 이름, 작은 것은 회색 선만. 배치를 따로 갖지 않는다.
 */
export function RoutePreview({ view, density, label }: { view: HomeMapView; density: "preview" | "mini"; label?: string }) {
  const m = useMessages();
  const scale = useUiScale();
  const box = useRef<HTMLSpanElement>(null);
  const [width, setWidth] = useState(800);
  useEffect(() => {
    const el = box.current;
    if (density === "mini" || !el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry && entry.contentRect.width > 0) setWidth(Math.floor(entry.contentRect.width));
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [density]);
  // 배치는 장면 단위로, 그릴 때 배율을 곱한다(D-153).
  const units = density === "mini" ? MINI_WIDTH : Math.min(MAX_WIDTH, Math.floor(width / scale));
  const scene = layoutHorizontal(view, units, density, { here: m.hereNow, changed: m.changedFilesCount, earlier: m.earlierVersions });
  const palette = density === "mini" ? monoPalette(view.defaultBranch) : branchPalette(view.defaultBranch);
  return (
    // 줄 단추 안에도 놓이므로 span으로 그린다(단추 안에는 div를 둘 수 없다).
    <span ref={box} className={density === "mini" ? styles.mini : styles.preview}>
      <SvgRenderer scene={scene} palette={palette} label={label} scale={scale} />
    </span>
  );
}
