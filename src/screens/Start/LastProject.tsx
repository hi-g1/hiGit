import type { UseQueryResult } from "@tanstack/react-query";
import { useOpenProject } from "../../features/open-project/useOpenProject";
import { pickLastVersion, pickStatus } from "../../features/start/summary";
import type { ApiError, ProjectSummaryView, RecentProject } from "../../lib/api";
import { formatDate, formatRelative } from "../../lib/time";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import styles from "./Start.module.css";

/** "마지막으로 하던 곳": 이름(26px)·경로·상태 문장·이어서 하기. 노선도 자리는 D-017 확정 뒤. */
/** `now`: 상대 시각의 기준(목록을 불러온 시각, ms). 다시 그려도 글자가 바뀌지 않게 한다. */
export function LastProject({
  project,
  summary,
  now,
}: {
  project: RecentProject;
  summary: UseQueryResult<ProjectSummaryView, ApiError>;
  now: number;
}) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const open = useOpenProject();
  const selectFirstChange = useRepoStore((s) => s.setSelectFirstChange);
  const gone = summary.error?.code === "folderNotFound" ? m.folderMissing : summary.error ? m.folderNotRepo : null;

  return (
    <div className={styles.last}>
      <p className={styles.small}>{m.lastPlace(formatRelative(project.lastOpenedAt, now, locale))}</p>
      <div className={styles.nameLine}>
        <h1 className={gone ? `${styles.name} ${styles.muted}` : styles.name}>{project.name}</h1>
        <span className={styles.path}>{project.path}</span>
      </div>
      <p className={styles.sentence}>
        {gone ??
          (summary.data
            ? `${m.statusSentence(pickStatus(summary.data))} ${m.lastVersionSentence(
                pickLastVersion(summary.data),
                summary.data.lastCommit ? formatDate(summary.data.lastCommit.authorTime, now, locale) : "",
              )}`
            : " ")}
      </p>
      {!gone && (
        <div className={styles.actions}>
          <Button variant="primary" disabled={open.isPending} onClick={() => void open.open(project.path, "recent")}>
            {m.continueWork}
          </Button>
          {summary.data && summary.data.changedFiles > 0 && (
            <Button
              variant="text"
              disabled={open.isPending}
              onClick={() => {
                selectFirstChange(true);
                void open.open(project.path, "recent");
              }}
            >
              {m.viewChangedFiles(summary.data.changedFiles)}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
