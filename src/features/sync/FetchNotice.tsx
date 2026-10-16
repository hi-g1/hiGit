import { useEffect, useState } from "react";
import { formatTime } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { useErrorAction } from "../../ui/errorActions";
import styles from "../pending/PendingNotice.module.css";
import { useSyncStore } from "./syncStore";

/**
 * 저절로 하는 원격 확인이 실패하면 상단 바 아래 알림 줄(D-197). "원격 확인 못 함 · 3분 전", 오류 코드의 해결
 * 버튼 하나(`ERROR_FIX`), 닫기. 닫으면 다음 실패까지 보이지 않고, 확인이 성공하면 사라진다.
 */
export function FetchNotice() {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const { fetchFailedAt, fetchFailedCode, fetchNoticeDismissed, set } = useSyncStore();
  const fix = useErrorAction(fetchFailedCode ?? "internal");
  const now = useNow(30_000);
  if (fetchFailedAt === null || fetchNoticeDismissed) return null;
  return (
    <div className={styles.lines}>
      <div className={styles.line} role="status">
        <span className={styles.text}>{m.fetchFailed(formatTime(Math.floor(fetchFailedAt / 1000), now, locale))}</span>
        {fix}
        <Button variant="text" onClick={() => set({ fetchNoticeDismissed: true })}>
          {m.close}
        </Button>
      </div>
    </div>
  );
}

/** 몇 초마다 바뀌는 지금 시각(그리는 중에 Date.now()를 부르지 않게). */
export function useNow(everyMs: number): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), everyMs);
    return () => window.clearInterval(timer);
  }, [everyMs]);
  return now;
}
