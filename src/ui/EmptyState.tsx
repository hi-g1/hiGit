import type { ReactNode } from "react";
import styles from "./EmptyState.module.css";

/** 빈 상태: 그림 없이 한 줄 설명과, 있으면 버튼 하나(docs/design.md). */
export function EmptyState({ children, action }: { children: ReactNode; action?: ReactNode }) {
  return (
    <div className={styles.empty}>
      <p>{children}</p>
      {action}
    </div>
  );
}
