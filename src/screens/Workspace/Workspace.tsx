import { HistoryList } from "../../features/history/HistoryList";
import { toHistoryRows } from "../../features/history/model";
import type { RepoInfo } from "../../lib/api";
import { HISTORY_LIMIT, useLog, useStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Panel } from "../../ui/Panel";
import { Term } from "../../ui/Term";
import { ChangedFiles } from "./ChangedFiles";
import styles from "./Workspace.module.css";

export function Workspace({ repo }: { repo: RepoInfo }) {
  const m = useMessages();
  const closeRepo = useRepoStore((s) => s.closeRepo);
  const status = useStatus(repo.path);
  const log = useLog(repo.path);

  return (
    <div className={styles.workspace}>
      <header className={styles.header}>
        <div className={styles.titles}>
          <h1 className={styles.name}>{repo.name}</h1>
          <span className={styles.path}>{repo.path}</span>
        </div>
        <Button variant="text" onClick={closeRepo}>
          {m.openOtherFolder}
        </Button>
      </header>

      <main className={styles.panels}>
        <Panel title={<Term id="status" />} aside={status.data && status.data.files.length}>
          {status.isPending && <EmptyState>{m.loading}</EmptyState>}
          {status.error && <ErrorNotice error={status.error} />}
          {status.data &&
            (status.data.files.length === 0 ? (
              <EmptyState>{m.noChanges}</EmptyState>
            ) : (
              <ChangedFiles files={status.data.files} />
            ))}
        </Panel>

        <Panel title={<Term id="log" />} aside={log.data?.hasMore ? m.moreCommits(HISTORY_LIMIT) : undefined}>
          {log.isPending && <EmptyState>{m.loading}</EmptyState>}
          {log.error && <ErrorNotice error={log.error} />}
          {log.data &&
            (log.data.commits.length === 0 ? (
              <EmptyState>{m.noCommits}</EmptyState>
            ) : (
              <HistoryList rows={toHistoryRows(log.data)} now={log.dataUpdatedAt} />
            ))}
        </Panel>
      </main>
    </div>
  );
}
