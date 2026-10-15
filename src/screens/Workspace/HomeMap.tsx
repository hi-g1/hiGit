import { useEffect, useRef, useState } from "react";
import { branchColor } from "../../features/branches/color";
import { layoutTwoLine, MAP_HEIGHT } from "../../features/history/layout/twoLine";
import { SvgRenderer } from "../../features/history/render/SvgRenderer";
import { useHomeMap } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useTermLabel } from "../../ui/Term";
import styles from "./HomeMap.module.css";

/** 홈 위쪽 두 줄 노선도(D-046, r-home.png). main과 지금 브랜치만. 전체는 버전 기록 화면에서. */
export function HomeMap({ repo }: { repo: string }) {
  const m = useMessages();
  const label = useTermLabel("log");
  const map = useHomeMap(repo);
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(800);
  useEffect(() => {
    const el = box.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) setWidth(Math.max(320, Math.floor(entry.contentRect.width)));
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  const view = map.data;
  if (!view || view.currentCommits.length === 0) return null;
  const scene = layoutTwoLine(view, width, { here: m.hereNow, changed: m.changedFilesCount });
  const main = branchColor(view.defaultBranch ?? "", view.defaultBranch);
  const current = view.twoLines && view.current ? branchColor(view.current, view.defaultBranch) : main;
  return (
    <div ref={box} className={styles.map} style={{ height: MAP_HEIGHT }}>
      <SvgRenderer scene={scene} colors={{ main, current }} label={label} />
    </div>
  );
}
