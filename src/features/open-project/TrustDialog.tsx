import * as Dialog from "@radix-ui/react-dialog";
import { useState } from "react";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./TrustDialog.module.css";
import { useOpenProject } from "./useOpenProject";
import { useTrustStore } from "./trustStore";

/**
 * 저장소 신뢰 확인(D-039, D-067). 열 때 프로그램을 실행시키는 설정이나 hook이 있으면
 * 이름을 보여주고 "믿고 열기"와 "끄고 열기"를 고르게 한다. 주 버튼은 안전한 "끄고 열기".
 */
export function TrustDialog() {
  const m = useMessages();
  const pending = useTrustStore((s) => s.pending);
  const clear = useTrustStore((s) => s.clear);
  const project = useOpenProject();
  const [error, setError] = useState<unknown>(null);

  const choose = async (choice: "trust" | "disable") => {
    if (!pending) return;
    try {
      await project.trust(pending.folder, choice);
      clear();
      setError(null);
    } catch (e) {
      setError(e);
    }
  };

  return (
    <Dialog.Root
      open={pending !== null}
      onOpenChange={(open) => {
        if (!open) clear();
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay className={styles.overlay} />
        <Dialog.Content className={styles.dialog}>
          <Dialog.Title className={styles.title}>{m.trustTitle}</Dialog.Title>
          <Dialog.Description className={styles.folder}>{pending?.folder}</Dialog.Description>
          <ul className={styles.keys}>
            {pending?.keys.map((k) => (
              <li key={k}>{k}</li>
            ))}
          </ul>
          {error !== null && error !== undefined && <ErrorNotice error={error} />}
          <div className={styles.actions}>
            <Button variant="secondary" onClick={() => void choose("trust")}>
              {m.trustOpen}
            </Button>
            <Button variant="primary" onClick={() => void choose("disable")}>
              {m.trustDisable}
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
