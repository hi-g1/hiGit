import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
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

/**
 * 설정의 GitHub 구역(D-109): 준비 안 됨 · 미연결 · 연결됨. 연결됨이면 워크플로 권한(D-135)을 같은 연결
 * 패널로 더한다. `workflow`면 열자마자 그 권한으로 연결을 시작한다(오류 줄의 "워크플로 권한 추가").
 */
export function GitHubSection({ autoConnect, workflow = false }: { autoConnect: boolean; workflow?: boolean }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const status = useGitHubStatus();
  const showToast = useToastStore((s) => s.show);
  const [disconnected, setDisconnected] = useState(false);
  const [disconnectError, setDisconnectError] = useState<unknown>(null);
  const asking = useRef(false);
  const { state, start, cancel } = useConnect(() => {
    showToast(asking.current ? m.settings.workflowAdded : m.settings.connected);
    asking.current = false;
    // 연결에 기대는 조회를 모두 다시 읽고, 인증 실패로 멈춘 원격 확인을 푼다.
    void queryClient.invalidateQueries({ queryKey: queryKeys.github });
    void queryClient.invalidateQueries({ queryKey: ["mergeHint"] });
    useSyncStore.getState().set({ paused: false });
  });
  const started = useRef(false);
  const data = status.data;
  const begin = useCallback(
    (withWorkflow: boolean) => {
      asking.current = withWorkflow && data?.connected === true;
      void start(withWorkflow);
    },
    [data, start],
  );
  useEffect(() => {
    if (!autoConnect || !data?.configured || started.current) return;
    if (!data.connected || (workflow && !data.workflow)) {
      started.current = true;
      begin(workflow);
    }
  }, [autoConnect, workflow, data, begin]);

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
            <Button variant="primary" disabled={busy} onClick={() => begin(false)}>
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
          <ConnectPanel state={state} onRetry={() => begin(workflow)} onCancel={cancel} />
        </>
      )}
      {data?.connected && (
        <>
          <div className={styles.row}>
            <code className={styles.login}>@{data.login}</code>
            <span className={styles.muted}>{data.remembered ? m.settings.savedHere : m.settings.sessionOnly}</span>
            <span className={styles.spacer} />
            <Button variant="secondary" disabled={busy} onClick={() => void disconnect()}>
              {m.settings.disconnect}
            </Button>
          </div>
          <div className={styles.row}>
            {data.workflow ? (
              <span className={styles.muted}>{m.settings.hasWorkflow}</span>
            ) : (
              <Button variant="text" disabled={busy} onClick={() => begin(true)}>
                {m.settings.addWorkflow}
              </Button>
            )}
          </div>
          <ConnectPanel state={state} onRetry={() => begin(true)} onCancel={cancel} />
        </>
      )}
      {disconnectError !== null && <ErrorNotice error={disconnectError} />}
    </section>
  );
}
