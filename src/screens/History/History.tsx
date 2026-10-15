import { useEffect } from "react";
import { HistoryList } from "../../features/history/HistoryList";
import { toHistoryRows } from "../../features/history/model";
import { HISTORY_LIMIT, useLogPages } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./History.module.css";

/** 버전 기록 화면(D-058). 창 전체 폭. Esc나 상단 바 "버전 기록"을 다시 누르면 홈으로 돌아간다. */
export function History({ repo }: { repo: string }) {
  const m = useMessages();
  const log = useLogPages(repo, "all");
  const first = log.data?.pages[0];
  const page = first && { ...first, commits: log.data!.pages.flatMap((p) => p.commits) };
  const setView = useRepoStore((s) => s.setView);
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented) setView("home");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setView]);

  return (
    <section className={styles.history} aria-label={m.historyArea}>
      {log.isPending && <EmptyState>{m.loading}</EmptyState>}
      {log.error && <ErrorNotice error={log.error} />}
      {page &&
        (page.commits.length === 0 ? (
          <EmptyState>{m.noCommits}</EmptyState>
        ) : (
          <>
            <HistoryList rows={toHistoryRows(page)} now={log.dataUpdatedAt} />
            {log.hasNextPage && <p className={styles.note}>{m.moreCommits(HISTORY_LIMIT)}</p>}
          </>
        ))}
    </section>
  );
}
