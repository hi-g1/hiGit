import { HistoryList } from "../../features/history/HistoryList";
import { toHistoryRows } from "../../features/history/model";
import type { RepoInfo } from "../../lib/api";
import { HISTORY_LIMIT, useLog, useStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { ChangedFiles } from "./ChangedFiles";
import { LeftTabs } from "./LeftTabs";
import { TopBar } from "./TopBar";
import styles from "./Workspace.module.css";

/** 작업 공간: 상단 바 + 2단(D-025). 버전 기록 탭은 목록을 창 전체 폭으로 쓴다. */
export function Workspace({ repo }: { repo: RepoInfo }) {
  const m = useMessages();
  const tab = useRepoStore((s) => s.leftTab);
  const status = useStatus(repo.path);
  const log = useLog(repo.path);

  return (
    <div className={styles.workspace}>
      <TopBar repo={repo} status={status.data} />
      <div className={tab === "changes" ? styles.twoPane : styles.onePane}>
        <section className={styles.left}>
          <LeftTabs changeCount={status.data?.files.length} />
          {tab === "changes" ? (
            <div id="panel-changes" role="tabpanel" aria-labelledby="tab-changes" className={styles.scroll}>
              {status.isPending && <EmptyState>{m.loading}</EmptyState>}
              {status.error && <ErrorNotice error={status.error} />}
              {status.data &&
                (status.data.files.length === 0 ? (
                  <EmptyState>{m.noChanges}</EmptyState>
                ) : (
                  <ChangedFiles folder={repo.path} files={status.data.files} />
                ))}
            </div>
          ) : (
            <div id="panel-history" role="tabpanel" aria-labelledby="tab-history" className={styles.scroll}>
              {log.isPending && <EmptyState>{m.loading}</EmptyState>}
              {log.error && <ErrorNotice error={log.error} />}
              {log.data &&
                (log.data.commits.length === 0 ? (
                  <EmptyState>{m.noCommits}</EmptyState>
                ) : (
                  <>
                    <HistoryList rows={toHistoryRows(log.data)} now={log.dataUpdatedAt} />
                    {log.data.hasMore && <p className={styles.note}>{m.moreCommits(HISTORY_LIMIT)}</p>}
                  </>
                ))}
            </div>
          )}
        </section>
        {tab === "changes" && (
          <section className={styles.right} aria-label={m.compareArea}>
            <EmptyState>{m.pickFileToCompare}</EmptyState>
          </section>
        )}
      </div>
    </div>
  );
}
