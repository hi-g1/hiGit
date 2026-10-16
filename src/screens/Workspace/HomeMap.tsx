import { useEffect, useRef, useState } from "react";
import { MergeHint } from "../../features/merge/MergeHint";
import { layoutHorizontal, MAP_HEIGHT } from "../../features/history/layout/horizontal";
import { branchPalette } from "../../features/history/render/palette";
import { SvgRenderer } from "../../features/history/render/SvgRenderer";
import { useHomeMap } from "../../lib/queries";
import { useUiScale } from "../../lib/useUiScale";
import { useMessages } from "../../messages";
import { useTermLabel } from "../../ui/Term";
import styles from "./HomeMap.module.css";

/** 홈 위쪽 두 줄 노선도(D-046, r-home.png). main과 지금 브랜치만. 전체는 버전 기록 화면에서. */
export function HomeMap({ repo, onDone }: { repo: string; onDone: (text: string) => void }) {
  const m = useMessages();
  const label = useTermLabel("log");
  const map = useHomeMap(repo);
  const scale = useUiScale();
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(800);
  useEffect(() => {
    const el = box.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) setWidth(Math.floor(entry.contentRect.width));
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  const view = map.data;
  if (!view || view.currentCommits.length === 0) return null;
  // 배치는 장면 단위로, 그릴 때 배율을 곱한다(D-153).
  const scene = layoutHorizontal(view, Math.max(320, Math.floor(width / scale)), "home", {
    here: m.hereNow,
    changed: m.changedFilesCount,
  });
  return (
    <div className={styles.map} style={{ height: MAP_HEIGHT * scale }}>
      <div ref={box} className={styles.route}>
        <SvgRenderer scene={scene} palette={branchPalette(view.defaultBranch)} label={label} scale={scale} />
      </div>
      <MergeHint repo={repo} onDone={onDone} />
    </div>
  );
}
