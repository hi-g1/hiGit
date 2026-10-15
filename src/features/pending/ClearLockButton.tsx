import * as Dialog from "@radix-ui/react-dialog";
import { useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { ApiError, api } from "../../lib/api";
import { queryKeys, usePendingWork } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { useToastStore } from "../../ui/toastStore";
import styles from "./PendingNotice.module.css";

/**
 * 멈춘 작업 정리(D-081). 오래된 잠금 파일이 있을 때만 보인다. 되돌릴 수 없어 지울 파일 이름을
 * 보여 주는 확인을 거친다(D-043). 확인 전에 미리보기 토큰을 받는다(고친 것 버리기와 같은 방식).
 */
export function ClearLockButton({ repo }: { repo: string }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const locks = usePendingWork(repo).data?.staleLocks ?? [];
  const [token, setToken] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const closeRef = useRef<HTMLButtonElement>(null);
  if (locks.length === 0 && token === null) return null;

  const fail = (e: unknown) => show(m.errors[e instanceof ApiError ? e.code : "internal"]);
  const refresh = () => {
    for (const key of [queryKeys.pendingWork(repo), queryKeys.status(repo)]) {
      void queryClient.invalidateQueries({ queryKey: key });
    }
  };

  const ask = async () => {
    try {
      setToken((await api.previewAction(repo, { id: "clear-lock" })).token);
    } catch (e) {
      fail(e);
    }
  };

  const clear = async () => {
    if (token === null) return;
    setPending(true);
    try {
      const result = await api.executeAction(repo, { id: "clear-lock" }, token);
      if (result.kind === "locksCleared") show(m.locksCleared(result.count));
    } catch (e) {
      fail(e);
    } finally {
      setToken(null);
      setPending(false);
      refresh();
    }
  };

  return (
    <>
      <Button variant="secondary" disabled={pending} onClick={() => void ask()}>
        {m.clearLocks}
      </Button>
      <Dialog.Root open={token !== null} onOpenChange={(open) => !open && setToken(null)}>
        <Dialog.Portal>
          <Dialog.Overlay className={styles.overlay} />
          <Dialog.Content
            className={styles.dialog}
            // 되돌릴 수 없어 Enter가 정리하기가 되지 않게 안전한 쪽에 포커스를 둔다(D-043).
            onOpenAutoFocus={(e) => {
              e.preventDefault();
              closeRef.current?.focus();
            }}
          >
            <Dialog.Title className={styles.title}>{m.clearLocksTitle(locks.length)}</Dialog.Title>
            <Dialog.Description className={styles.paths}>{locks.join(" · ")}</Dialog.Description>
            <div className={styles.actions}>
              <Button variant="danger" disabled={pending} onClick={() => void clear()}>
                {m.clearLocks}
              </Button>
              <Button ref={closeRef} variant="primary" onClick={() => setToken(null)}>
                {m.close}
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}
