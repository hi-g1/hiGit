import { CommitBox } from "../../features/commit/CommitBox";
import { useEffect } from "react";
import { DiffView } from "../../features/diff/DiffView";
import { HistoryList } from "../../features/history/HistoryList";
import { toHistoryRows } from "../../features/history/model";
import type { RepoInfo } from "../../lib/api";
import { HISTORY_LIMIT, useLog, useStatus } from "../../lib/queries";
import { useRepoWatch } from "../../lib/useRepoWatch";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
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
  const selectedPath = useRepoStore((s) => s.selectedPath);
  const status = useStatus(repo.path);
  const log = useLog(repo.path);
  const watch = useRepoWatch(repo.path);
  const selectPath = useRepoStore((s) => s.selectPath);
  const selected = status.data?.files.find((f) => f.path === selectedPath);
  // 고른 파일이 목록에서 사라지면(버전으로 저장됨 등) 선택도 지운다.
  // 같은 경로가 나중에 다시 생겨도 비교가 저절로 열리지 않게 한다.
  useEffect(() => {
    if (status.data && selectedPath !== null && !selected) selectPath(null);
  }, [status.data, selectedPath, selected, selectPath]);
  // 시작 화면의 "바뀐 파일 N개 보기"로 왔으면 첫 바뀐 파일을 고른다.
  const selectFirstChange = useRepoStore((s) => s.selectFirstChange);
  const setSelectFirstChange = useRepoStore((s) => s.setSelectFirstChange);
  useEffect(() => {
    const first = status.data?.files[0];
    if (selectFirstChange && first) {
      selectPath(first.path);
      setSelectFirstChange(false);
    }
  }, [selectFirstChange, status.data, selectPath, setSelectFirstChange]);

  return (
    <div className={styles.workspace}>
      <TopBar repo={repo} status={status.data} />
      {watch.active === false && (
        <div className={styles.watchNotice} role="status">
          <span>{m.errors.watchUnavailable}</span>
          <Button variant="secondary" onClick={watch.reload}>
            {m.reload}
          </Button>
        </div>
      )}
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
          {tab === "changes" && (
            <CommitBox
              folder={repo.path}
              includedCount={status.data?.files.filter((f) => f.staged).length ?? 0}
            />
          )}
        </section>
        {tab === "changes" && (
          <section className={styles.right} aria-label={m.compareArea}>
            {selected ? (
              <DiffView
                key={selected.path}
                repo={repo.path}
                request={{
                  path: selected.path,
                  origPath: selected.origPath,
                  untracked: selected.change === "untracked",
                }}
              />
            ) : (
              <EmptyState>{m.pickFileToCompare}</EmptyState>
            )}
          </section>
        )}
      </div>
    </div>
  );
}
