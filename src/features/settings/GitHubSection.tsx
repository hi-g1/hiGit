import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { api } from "../../lib/api";
import { queryKeys, useGitHubStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { useToastStore } from "../../ui/toastStore";
import { Tooltip } from "../../ui/Tooltip";
import { useConnect } from "../github/useConnect";
import { useSyncStore } from "../sync/syncStore";
import { ConnectPanel } from "./ConnectPanel";
import styles from "./SettingsDialog.module.css";

/** 설정의 GitHub 구역(D-109): 준비 안 됨 · 미연결 · 연결됨. */
export function GitHubSection({ autoConnect }: { autoConnect: boolean }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const status = useGitHubStatus();
  const showToast = useToastStore((s) => s.show);
  const [disconnected, setDisconnected] = useState(false);
  const [disconnectError, setDisconnectError] = useState<unknown>(null);
  const { state, start, cancel } = useConnect(() => {
    showToast(m.settings.connected);
    // 연결에 기대는 조회를 모두 다시 읽고, 인증 실패로 멈춘 원격 확인을 푼다.
    void queryClient.invalidateQueries({ queryKey: queryKeys.github });
    void queryClient.invalidateQueries({ queryKey: ["mergeHint"] });
    useSyncStore.getState().set({ paused: false });
  });
  const started = useRef(false);
  const data = status.data;
  useEffect(() => {
    if (autoConnect && data?.configured && !data.connected && !started.current) {
      started.current = true;
      void start();
    }
  }, [autoConnect, data, start]);

  const busy = state.kind === "starting" || state.kind === "waiting";
  const disconnect = async () => {
    setDisconnectError(null);
    try {
      await api.githubDisconnect();
      setDisconnected(true);
    } catch (error) {
      setDisconnectError(error);
    }
    void queryClient.invalidateQueries({ queryKey: queryKeys.github });
  };

  return (
    <section className={styles.section} aria-labelledby="settings-github">
      <h3 id="settings-github" className={styles.sectionTitle}>
        {m.settings.github}
      </h3>
      {status.error && <ErrorNotice error={status.error} />}
      {data && !data.configured && (
        <div className={styles.row}>
          <p className={styles.muted}>{m.settings.notConfigured}</p>
          <Tooltip content={m.settings.notConfigured}>
            <span>
              <Button variant="primary" disabled>
                {m.settings.connect}
              </Button>
            </span>
          </Tooltip>
        </div>
      )}
      {data?.configured && !data.connected && (
        <>
          <div className={styles.row}>
            <Button variant="primary" disabled={busy} onClick={() => void start()}>
              {m.settings.connect}
            </Button>
            {disconnected && (
              <Button
                variant="text"
                onClick={() => void api.openHostingPage(null, { kind: "appSettings" }).catch(() => undefined)}
              >
                {m.settings.removeAccess}
              </Button>
            )}
          </div>
          <ConnectPanel state={state} onRetry={() => void start()} onCancel={cancel} />
        </>
      )}
      {data?.connected && (
        <div className={styles.row}>
          <code className={styles.login}>@{data.login}</code>
          <span className={styles.muted}>{data.remembered ? m.settings.savedHere : m.settings.sessionOnly}</span>
          <span className={styles.spacer} />
          <Button variant="secondary" onClick={() => void disconnect()}>
            {m.settings.disconnect}
          </Button>
        </div>
      )}
      {disconnectError !== null && <ErrorNotice error={disconnectError} />}
    </section>
  );
}
