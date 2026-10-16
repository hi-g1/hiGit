import { create } from "zustand";
import { useAppStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import styles from "../pending/PendingNotice.module.css";

/** 이번 실행에서 닫았는지. 시작 화면과 홈이 같이 본다. */
export const useUndoKeyNoticeStore = create<{ dismissed: boolean; dismiss: () => void }>((set) => ({
  dismissed: false,
  dismiss: () => set({ dismissed: true }),
}));

/**
 * 되돌리기 키를 이 컴퓨터에 저장하지 못했을 때의 알림 줄(D-210). 해결 버튼은 없고 "자세히 보기"에 앱 데이터
 * 폴더(홈은 가림)를 접는다. 닫으면 이번 실행 동안 시작 화면·홈 모두에서 다시 뜨지 않는다.
 */
export function UndoKeyNotice() {
  const m = useMessages();
  const status = useAppStatus().data;
  const { dismissed, dismiss } = useUndoKeyNoticeStore();
  if (!status || status.undoKey !== "temporary" || dismissed) return null;
  return (
    <div className={styles.lines}>
      <div className={styles.line} role="status">
        <span className={styles.text}>
          {m.undoKeyTemporary}
          {status.undoKeyDetail && (
            <details>
              <summary>{m.showDetail}</summary>
              <code>{status.undoKeyDetail}</code>
            </details>
          )}
        </span>
        <Button variant="text" onClick={dismiss}>
          {m.close}
        </Button>
      </div>
    </div>
  );
}
