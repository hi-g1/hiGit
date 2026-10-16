import styles from "./ProgressBar.module.css";

/** 진행 막대(D-172). 화면 읽기 프로그램에는 값과 최댓값이 읽힌다. */
export function ProgressBar({ value, max, label }: { value: number; max: number; label: string }) {
  const percent = max > 0 ? Math.min(100, Math.round((value / max) * 100)) : 0;
  return (
    <span
      className={styles.track}
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={value}
    >
      <span className={styles.fill} style={{ width: `${percent}%` }} />
    </span>
  );
}
