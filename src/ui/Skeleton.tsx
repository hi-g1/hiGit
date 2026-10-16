import { useMessages } from "../messages";
import styles from "./Skeleton.module.css";

/** 처음 불러오는 동안의 빈 줄 모양(D-193). 화면 읽기 프로그램에는 "불러오는 중"으로 읽힌다. */
export function Skeleton({ lines = 4 }: { lines?: number }) {
  const m = useMessages();
  return (
    <div className={styles.skeleton} role="status">
      <span className={styles.hidden}>{m.loading}</span>
      {Array.from({ length: Math.min(6, Math.max(3, lines)) }, (_, i) => (
        <span key={i} className={styles.bar} style={{ width: `${88 - (i % 3) * 18}%` }} aria-hidden="true" />
      ))}
    </div>
  );
}
