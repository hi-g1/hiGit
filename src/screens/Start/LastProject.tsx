import type { UseQueryResult } from "@tanstack/react-query";
import { MAP_HEIGHT } from "../../features/history/layout/horizontal";
import { RoutePreview } from "../../features/history/RoutePreview";
import { useOpenProject } from "../../features/open-project/useOpenProject";
import { pickLastVersion, pickStatus } from "../../features/start/summary";
import type { ApiError, ProjectSummaryView, RecentProject } from "../../lib/api";
import { useRoutePreview } from "../../lib/queries";
import { formatDate, formatRelative } from "../../lib/time";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import styles from "./Start.module.css";

/** "마지막으로 하던 곳": 이름(26px)·경로·상태 문장·큰 노선도·이어서 하기(start-normal.png, D-093). */
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
  const gone =
    summary.error?.code === "folderNotFound"
      ? m.folderMissing
      : summary.error?.code === "needsTrust"
        ? m.trustNeeded
        : summary.error
          ? m.folderNotRepo
          : null;
  // 노선도는 상태 요약이 성공한 저장소만 부른다(사라진 폴더·신뢰 필요에서는 git을 부르지 않는다).
  const hasCommits = summary.data?.branch.hasCommits === true;
  const preview = useRoutePreview(project.path, 12, !gone && hasCommits);
  const map = preview.data;

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
      {!gone && hasCommits && !preview.isError && (map === undefined || map.currentCommits.length > 0) && (
        // 불러오는 동안에도 같은 높이를 두어 아래가 밀리지 않게 한다.
        <div className={styles.route} style={{ height: MAP_HEIGHT }}>
          {map && (
            <RoutePreview
              view={map}
              density="preview"
              label={m.routePreviewLabel(map.current ?? "", map.currentCommits.length + map.currentMore)}
            />
          )}
        </div>
      )}
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
