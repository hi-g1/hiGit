import { ApiError } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { useAppSettings } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useForcePushStore } from "../force-push/forcePushStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { ClearLockButton } from "../pending/ClearLockButton";
import styles from "./SyncNotice.module.css";
import { useSyncStore } from "./syncStore";
import { useSync } from "./useSync";

/** 마지막 동기화 오류(상단 바 아래 안내 줄). 멈추거나 시간이 다 돼 그만뒀으면 다시 할 수 있다(D-080). */
export function SyncNotice({ repo }: { repo: string }) {
  const m = useMessages();
  const error = useSyncStore((s) => s.error);
  const sync = useSync(repo);
  const allowRewrite = (useAppSettings().data ?? DEFAULT_SETTINGS).allowPublishedRewrite;
  const openForcePush = useForcePushStore((s) => s.openDialog);
  if (error === null || error === undefined) return null;
  const code = error instanceof ApiError ? error.code : null;
  const retry = code === "stalled" || code === "timedOut";
  // 올린 기록을 바꿔 갈라졌다(D-129): 덮어쓰며 올리기(설정이 켜졌을 때)나 그래도 합치기.
  const rewrote = code === "rewrotePublished";
  return (
    <div className={styles.notice}>
      <ErrorNotice error={error}>
        {/* 버려진 잠금 파일이면 정리하기(D-081). 오래되지 않았으면 단추가 없다. */}
        {code === "indexLocked" && <ClearLockButton repo={repo} />}
        {rewrote && allowRewrite && (
          <Button variant="secondary" onClick={openForcePush}>
            <Term id="force-push" focusable={false} />
          </Button>
        )}
        {rewrote && (
          <Button variant="text" disabled={sync.running} onClick={sync.runAnyway}>
            {m.forcePush.mergeAnyway}
          </Button>
        )}
      </ErrorNotice>
      {retry && (
        <Button variant="secondary" disabled={sync.running} onClick={sync.run}>
          <Term id="sync" focusable={false} />
        </Button>
      )}
    </div>
  );
}
