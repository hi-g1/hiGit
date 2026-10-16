import { useState } from "react";
import { ApiError, api } from "../../lib/api";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ConfirmDialog } from "../../ui/ConfirmDialog";
import { useToastStore } from "../../ui/toastStore";
import styles from "./StuckNotice.module.css";
import { REMOTE_ACTIONS, STUCK_LOCAL_SECS, useProgress } from "./useProgress";

/**
 * 로컬 쓰기(저장·합치기 등)가 60초를 넘으면 보이는 안내 줄(D-080). gpg 핀 입력이나 느린 hook이
 * 잠금째 붙잡은 것을 사람이 확인한 뒤 강제로 멈춘다. 원격 작업은 동기화 버튼이 취소한다.
 */
export function StuckNotice({ repo }: { repo: string }) {
  const m = useMessages();
  const show = useToastStore((s) => s.show);
  const progress = useProgress(repo);
  const [confirm, setConfirm] = useState(false);
  const stuck =
    progress !== null && !REMOTE_ACTIONS.has(progress.action) && progress.elapsedSecs >= STUCK_LOCAL_SECS;
  if (!stuck && !confirm) return null;

  const stop = async () => {
    setConfirm(false);
    try {
      await api.cancelAction(repo);
    } catch (e) {
      show(m.errors[e instanceof ApiError ? e.code : "internal"]);
    }
  };

  return (
    <>
      {stuck && (
        <div className={styles.notice} role="status">
          <span>{m.noResponse(progress.elapsedSecs)}</span>
          <Button variant="secondary" onClick={() => setConfirm(true)}>
            {m.forceStop}
          </Button>
        </div>
      )}
      <ConfirmDialog
        open={confirm}
        title={m.forceStopTitle}
        dangerLabel={m.forceStop}
        closeLabel={m.close}
        onConfirm={() => void stop()}
        onClose={() => setConfirm(false)}
      >
        <p className={styles.detail}>{m.forceStopDetail}</p>
      </ConfirmDialog>
    </>
  );
}
