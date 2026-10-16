import { ProgressCount } from "../progress/ProgressCount";
import { api, type BranchView } from "../../lib/api";
import { formatExact, formatTime } from "../../lib/time";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Kbd } from "../../ui/Kbd";
import { useTermLabel } from "../../ui/Term";
import { useUiStore } from "../../state/uiStore";
import { Tooltip } from "../../ui/Tooltip";
import { useToastStore } from "../../ui/toastStore";
import { useNow } from "./FetchNotice";
import { CANCEL_AFTER_SECS, NO_RESPONSE_SECS, REMOTE_ACTIONS, useProgress } from "../progress/useProgress";
import styles from "./SyncButton.module.css";
import { useSyncStore } from "./syncStore";
import { useSync } from "./useSync";

type Props = {
  repo: string;
  branch: BranchView;
  /** 고친 파일 수. 0이고 주고받을 것이 있으면 이 버튼이 주 버튼이 된다(r-saved.png). */
  changedFiles: number;
  /** Ctrl+Shift+S를 받는다. 같은 화면에 버튼이 둘이면 하나만 받는다. */
  shortcut?: boolean;
};

/**
 * 동기화 버튼(D-045). 가져온 다음 올린다. 받을 것·보낼 것 수를 옅게 붙인다. Ctrl+Shift+S.
 * 겹치면 합치기를 시작해 충돌 해결 화면으로 넘긴다(D-069). 시작하지 못했을 때만 버튼 옆에
 * "겹친 파일 N개"를 남긴다(D-048).
 * 도는 동안 "동기화 중 · N초", 3초 뒤 "취소", 30초 동안 출력이 없으면 "응답 없음 · N초"(D-080).
 * 툴팁은 할 일("2개 올리기 · 1개 가져오기"), 옆에 마지막 확인 시각(D-197). 원격 확인 실패는 알림 줄(`FetchNotice`).
 */
export function SyncButton({ repo, branch, changedFiles, shortcut = true }: Props) {
  const m = useMessages();
  const label = useTermLabel("sync");
  const locale = useUiStore((s) => s.locale);
  const showError = useToastStore((s) => s.showError);
  const { overlap, lastCheckedAt } = useSyncStore();
  const sync = useSync(repo);
  useShortcut("sync", sync.run, shortcut);
  const now = useNow(30_000);
  const tick = useProgress(repo);
  const progress = sync.running && tick && REMOTE_ACTIONS.has(tick.action) ? tick : null;

  const cancel = async () => {
    try {
      await api.cancelAction(repo);
    } catch (e) {
      showError(e);
    }
  };

  const ahead = branch.upstream ? branch.ahead : 0;
  const behind = branch.upstream ? branch.behind : 0;
  const primary = changedFiles === 0 && (ahead > 0 || behind > 0);
  return (
    <span className={styles.wrap}>
      {overlap.length > 0 && (
        <span className={styles.overlap} title={overlap.slice(0, 5).join("\n")}>
          {m.overlapFiles(overlap.length)}
        </span>
      )}
      <Tooltip
        content={
          <span className={styles.tip}>
            {branch.upstream ? m.syncPlan(ahead, behind) : m.syncFirstPush}
            <Kbd>{SHORTCUTS.sync.label}</Kbd>
          </span>
        }
      >
        <button
          type="button"
          className={`${styles.button} ${primary ? styles.primary : ""}`}
          disabled={sync.running}
          aria-busy={sync.running}
          aria-keyshortcuts="Control+Shift+S"
          onClick={sync.run}
        >
          {label}
        {sync.running ? (
          <span className={styles.count}>
            <ProgressCount tick={progress} fallback={m.syncRunning(progress?.elapsedSecs ?? null)} />
          </span>
        ) : (
          <>
            {ahead > 0 && <span className={styles.count}>↑{ahead}</span>}
            {behind > 0 && <span className={styles.count}>↓{behind}</span>}
          </>
        )}
        </button>
      </Tooltip>
      {lastCheckedAt !== null && !sync.running && (
        <span className={styles.checked} title={formatExact(Math.floor(lastCheckedAt / 1000), locale)}>
          {m.lastChecked(formatTime(Math.floor(lastCheckedAt / 1000), now, locale))}
        </span>
      )}
      {progress && progress.idleSecs >= NO_RESPONSE_SECS && (
        <span className={styles.noResponse}>{m.noResponse(progress.idleSecs)}</span>
      )}
      {progress && progress.elapsedSecs >= CANCEL_AFTER_SECS && (
        <Button variant="text" onClick={() => void cancel()}>
          {m.cancel}
        </Button>
      )}
    </span>
  );
}
