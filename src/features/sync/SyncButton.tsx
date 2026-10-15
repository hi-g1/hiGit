import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api, type BranchView } from "../../lib/api";
import { formatRelative } from "../../lib/time";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./SyncButton.module.css";
import { useSyncStore } from "./syncStore";

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
 * 겹치면 합치지 않고 버튼 옆에 "겹친 파일 N개"를 남긴다(D-048).
 */
export function SyncButton({ repo, branch, changedFiles, shortcut = true }: Props) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const { overlap, fetchFailedAt, set } = useSyncStore();

  const sync = useMutation({
    mutationFn: () => api.executeAction(repo, { id: "sync", input: { locale } }),
    onMutate: () => set({ paused: false, error: null }),
    onSuccess: (result) => {
      if (result.kind !== "synced") return;
      set({ overlap: result.overlap, fetchFailedAt: null });
      if (result.overlap.length === 0) show(m.synced(result.sent, result.received));
    },
    onError: (e) => set({ error: e }),
    onSettled: () => {
      for (const key of [["status", repo], ["log", repo], ["branches", repo]]) {
        void queryClient.invalidateQueries({ queryKey: key });
      }
    },
  });
  useShortcut(
    "sync",
    () => {
      if (!sync.isPending) sync.mutate();
    },
    shortcut,
  );
  const now = useNow(30_000);

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
      <button
        type="button"
        className={`${styles.button} ${primary ? styles.primary : ""}`}
        disabled={sync.isPending}
        aria-busy={sync.isPending}
        aria-keyshortcuts="Control+Shift+S"
        onClick={() => sync.mutate()}
      >
        <Term id="sync" focusable={false} />
        {ahead > 0 && <span className={styles.count}>↑{ahead}</span>}
        {behind > 0 && <span className={styles.count}>↓{behind}</span>}
      </button>
      {fetchFailedAt !== null && (
        <span
          className={styles.failed}
          role="img"
          aria-label={m.fetchFailed(formatRelative(fetchFailedAt / 1000, now, locale))}
          title={m.fetchFailed(formatRelative(fetchFailedAt / 1000, now, locale))}
        />
      )}
    </span>
  );
}

/** 몇 초마다 바뀌는 지금 시각(그리는 중에 Date.now()를 부르지 않게). */
function useNow(everyMs: number): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), everyMs);
    return () => window.clearInterval(timer);
  }, [everyMs]);
  return now;
}
