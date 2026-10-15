import { ApiError } from "../../lib/api";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import styles from "./SyncNotice.module.css";
import { useSyncStore } from "./syncStore";
import { useSync } from "./useSync";

/** 마지막 동기화 오류(상단 바 아래 안내 줄). 멈추거나 시간이 다 돼 그만뒀으면 다시 할 수 있다(D-080). */
export function SyncNotice({ repo }: { repo: string }) {
  const error = useSyncStore((s) => s.error);
  const sync = useSync(repo);
  if (error === null || error === undefined) return null;
  const retry = error instanceof ApiError && (error.code === "stalled" || error.code === "timedOut");
  return (
    <div className={styles.notice}>
      <ErrorNotice error={error} />
      {retry && (
        <Button variant="secondary" disabled={sync.running} onClick={sync.run}>
          <Term id="sync" focusable={false} />
        </Button>
      )}
    </div>
  );
}
