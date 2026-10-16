import type { Scene } from "./layout/scene";
import { Tooltip } from "../../ui/Tooltip";
import styles from "./StationTips.module.css";

/**
 * 노선도 역 위에 겹친 포커스 가능한 자리(D-155). 마우스를 올리거나 Tab으로 닿으면 전체 이름이 툴팁으로 뜬다
 * (SVG `<title>`은 키보드 포커스에 뜨지 않는다). 누르는 동작은 없어 단추가 아니다. 좌표는 장면 단위 × 배율.
 */
export function StationTips({ scene, scale }: { scene: Scene; scale: number }) {
  return (
    <>
      {scene.stations
        .filter((s) => s.title)
        .map((s) => {
          const size = (s.r * 2 + 8) * scale;
          return (
            <Tooltip key={s.id ?? `${s.x}`} content={s.title}>
              <span
                className={styles.tip}
                tabIndex={0}
                role="img"
                aria-label={s.title}
                style={{ left: s.x * scale - size / 2, top: s.y * scale - size / 2, width: size, height: size }}
              />
            </Tooltip>
          );
        })}
    </>
  );
}
