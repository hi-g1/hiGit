import { useEffect, useState } from "react";
import { MergeHint } from "../../features/merge/MergeHint";
import { layoutHorizontal, MAP_HEIGHT } from "../../features/history/layout/horizontal";
import { SCENE_FONT } from "../../features/history/layout/scene";
import { branchPalette } from "../../features/history/render/palette";
import { SvgRenderer } from "../../features/history/render/SvgRenderer";
import { StationTips } from "../../features/history/StationTips";
import { useHomeMap } from "../../lib/queries";
import { useUiScale } from "../../lib/useUiScale";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { useTermLabel } from "../../ui/Term";
import styles from "./HomeMap.module.css";

/** 홈 위쪽 두 줄 노선도(D-046, r-home.png). main과 지금 브랜치만. 전체는 버전 기록 화면에서. */
export function HomeMap({ repo, onDone }: { repo: string; onDone: (text: string) => void }) {
  const m = useMessages();
  const label = useTermLabel("log");
  const map = useHomeMap(repo);
  const scale = useUiScale();
  const setView = useRepoStore((s) => s.setView);
  // 노선도는 자료가 온 뒤에야 그려지므로 칸이 생길 때 폭을 재기 시작한다(처음 그릴 때는 아직 없다).
  const [box, setBox] = useState<HTMLDivElement | null>(null);
  const [width, setWidth] = useState(800);
  useEffect(() => {
    if (!box || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) setWidth(Math.floor(entry.contentRect.width));
    });
    observer.observe(box);
    return () => observer.disconnect();
  }, [box]);
  const view = map.data;
  if (!view || view.currentCommits.length === 0) return null;
  // 배치는 장면 단위로, 그릴 때 배율을 곱한다(D-153).
  const scene = layoutHorizontal(view, Math.max(320, Math.floor(width / scale)), "home", {
    here: m.hereNow,
    changed: m.changedFilesCount,
    earlier: m.earlierVersions,
    interactiveFold: true,
  });
  const fold = scene.fold;
  return (
    <div className={styles.map} style={{ height: MAP_HEIGHT * scale }}>
      <div ref={setBox} className={styles.route}>
        <SvgRenderer scene={scene} palette={branchPalette(view.defaultBranch)} label={label} scale={scale} />
        {/* 접힌 앞쪽 "이전 N개"는 누르면 버전 기록을 여는 글자 단추(D-141). Tab 순서는 이것 → 역들. */}
        {fold && (
          <button
            type="button"
            className={styles.earlier}
            style={{ left: fold.x * scale, top: (fold.y - SCENE_FONT) * scale }}
            onClick={() => setView("history")}
          >
            {m.earlierVersions(fold.count)}
          </button>
        )}
        <StationTips scene={scene} scale={scale} />
      </div>
      <MergeHint repo={repo} onDone={onDone} />
    </div>
  );
}
