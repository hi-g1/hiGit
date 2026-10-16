import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, type ScopeAdd } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { queryKeys, useAppSettings, useGitHubStatus } from "../../lib/queries";
import { changeSettings } from "../../lib/settingsSave";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Checkbox } from "../../ui/Checkbox";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { useToastStore } from "../../ui/toastStore";
import { Tooltip } from "../../ui/Tooltip";
import { useConnect } from "../github/useConnect";
import { useSyncStore } from "../sync/syncStore";
import { ConnectPanel } from "./ConnectPanel";
import styles from "./SettingsDialog.module.css";

/**
 * 설정의 GitHub 구역(D-109): 준비 안 됨 · 미연결 · 연결됨. 연결됨이면 워크플로·이메일 권한(D-135, D-186)을
 * 같은 연결 패널로 더한다. `add`면 열자마자 그 권한으로 연결을 시작한다(오류 줄·이메일 줄의 단추).
 * "올리기와 가져오기에 이 연결 쓰기"(D-187)는 바로 저장하는 체크 상자다.
 */
export function GitHubSection({ autoConnect, add = null }: { autoConnect: boolean; add?: ScopeAdd | null }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const status = useGitHubStatus();
  const showToast = useToastStore((s) => s.show);
  const [disconnected, setDisconnected] = useState(false);
  const [disconnectError, setDisconnectError] = useState<unknown>(null);
  const asking = useRef(false);
  const lastAdd = useRef<ScopeAdd | null>(null);
  const settings = useAppSettings().data ?? DEFAULT_SETTINGS;
  const { state, start, cancel } = useConnect(() => {
    showToast(asking.current ? m.settings.workflowAdded : m.settings.connected);
    asking.current = false;
    // 연결에 기대는 조회를 모두 다시 읽고, 인증 실패로 멈춘 원격 확인을 푼다.
    void queryClient.invalidateQueries({ queryKey: queryKeys.github });
    void queryClient.invalidateQueries({ queryKey: ["mergeHint"] });
    useSyncStore.getState().set({ paused: false });
  });
  // 열린 대화상자에서 다른 권한 단추를 누르면 다시 시작할 수 있게 무엇으로 시작했는지 기억한다.
  const startedFor = useRef<string | null>(null);
  const data = status.data;
  const begin = useCallback(
    (scope: ScopeAdd | null) => {
      asking.current = scope !== null && data?.connected === true;
      lastAdd.current = scope;
      void start(scope);
    },
    [data, start],
  );
  useEffect(() => {
    const key = add ?? "connect";
    if (!autoConnect || !data?.configured || startedFor.current === key) return;
    const missing = (add === "workflow" && !data.workflow) || (add === "email" && !data.email);
    if (!data.connected || missing) {
      startedFor.current = key;
      begin(add);
    }
  }, [autoConnect, add, data, begin]);

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
            <Button variant="primary" disabled={busy} onClick={() => begin(null)}>
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
          <ConnectPanel state={state} onRetry={() => begin(lastAdd.current)} onCancel={cancel} />
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
              <Button variant="text" disabled={busy} onClick={() => begin("workflow")}>
                {m.settings.addWorkflow}
              </Button>
            )}
          </div>
          <ConnectPanel state={state} onRetry={() => begin(lastAdd.current)} onCancel={cancel} />
        </>
      )}
      {data?.configured && (
        <div className={styles.stack}>
          <label className={styles.field}>
            <span className={styles.fieldName}>{m.settings.gitHelper}</span>
            <Checkbox
              checked={settings.gitHelper}
              aria-describedby="settings-git-helper-hint"
              onChange={(e) => changeSettings(queryClient, { gitHelper: e.currentTarget.checked })}
            />
          </label>
          <p id="settings-git-helper-hint" className={styles.hint}>
            {m.settings.gitHelperHint}
          </p>
        </div>
      )}
      {disconnectError !== null && <ErrorNotice error={disconnectError} />}
    </section>
  );
}
