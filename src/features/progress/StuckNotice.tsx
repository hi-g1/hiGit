import * as Dialog from "@radix-ui/react-dialog";
import { useRef, useState } from "react";
import { ApiError, api } from "../../lib/api";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
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
  const closeRef = useRef<HTMLButtonElement>(null);
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
      <Dialog.Root open={confirm} onOpenChange={setConfirm}>
        <Dialog.Portal>
          <Dialog.Overlay className={styles.overlay} />
          <Dialog.Content
            className={styles.dialog}
            // Enter가 강제로 멈추기가 되지 않게 안전한 쪽에 포커스를 둔다(D-043).
            onOpenAutoFocus={(e) => {
              e.preventDefault();
              closeRef.current?.focus();
            }}
          >
            <Dialog.Title className={styles.title}>{m.forceStopTitle}</Dialog.Title>
            <Dialog.Description className={styles.detail}>{m.forceStopDetail}</Dialog.Description>
            <div className={styles.actions}>
              <Button variant="danger" onClick={() => void stop()}>
                {m.forceStop}
              </Button>
              <Button ref={closeRef} variant="primary" onClick={() => setConfirm(false)}>
                {m.close}
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}
