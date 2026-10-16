import type { ReactNode } from "react";
import { useMessages } from "../messages";
import { Button } from "./Button";
import styles from "./TipLine.module.css";

/**
 * 처음 하는 일 안내 한 줄(D-168, D-205). 알림 줄 모양이고 행동을 막지 않는다. "알겠어요"로 닫으면 다시 뜨지 않는다.
 * `action`은 글자 버튼 하나(예: "작업 내역").
 */
export function TipLine({ children, action, onDismiss }: { children: ReactNode; action?: ReactNode; onDismiss: () => void }) {
  const m = useMessages();
  return (
    <div className={styles.tip} role="note">
      <span className={styles.text}>{children}</span>
      {action}
      <Button variant="text" onClick={onDismiss}>
        {m.tipOk}
      </Button>
    </div>
  );
}
