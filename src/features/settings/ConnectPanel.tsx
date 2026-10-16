import { useEffect, useState } from "react";
import { copyText } from "../../lib/copy";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import type { ConnectState } from "../github/useConnect";
import styles from "./SettingsDialog.module.css";

/** 남은 시간 `m:ss`. */
export function remaining(ms: number): string {
  const secs = Math.max(0, Math.ceil(ms / 1000));
  return `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, "0")}`;
}

/** 연결 코드 판(D-121): 큰 코드·복사·남은 시간·취소, 만료·거절·오류는 한 줄과 다시 시도. */
export function ConnectPanel({
  state,
  onRetry,
  onCancel,
}: {
  state: ConnectState;
  onRetry: () => void;
  onCancel: () => void;
}) {
  const m = useMessages();
  const [now, setNow] = useState(() => Date.now());
  const [copied, setCopied] = useState(false);
  const waiting = state.kind === "waiting";
  useEffect(() => {
    if (!waiting) return;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [waiting]);

  if (state.kind === "waiting") {
    return (
      <div className={styles.panel}>
        <div className={styles.codeRow}>
          <code className={styles.code} aria-label={m.settings.code}>
            {state.code}
          </code>
          <Button
            variant="text"
            onClick={() => {
              void copyText(state.code).then(setCopied);
            }}
          >
            {copied ? m.settings.copied : m.settings.copy}
          </Button>
        </div>
        <p className={styles.muted}>{m.settings.enterCode(remaining(state.expiresAt - now))}</p>
        <div>
          <Button variant="text" onClick={onCancel}>
            {m.settings.cancel}
          </Button>
        </div>
      </div>
    );
  }
  if (state.kind === "expired" || state.kind === "denied" || state.kind === "error") {
    return (
      <div className={styles.panel}>
        {state.kind === "error" ? (
          <ErrorNotice error={state.error} />
        ) : (
          <p className={styles.muted} role="status">
            {state.kind === "expired" ? m.settings.expired : m.settings.denied}
          </p>
        )}
        <div>
          <Button variant="secondary" onClick={onRetry}>
            {m.settings.retry}
          </Button>
        </div>
      </div>
    );
  }
  return null;
}
