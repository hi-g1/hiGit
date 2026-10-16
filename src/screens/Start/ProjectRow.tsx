import { useRef } from "react";
import { RoutePreview } from "../../features/history/RoutePreview";
import { useOpenProject } from "../../features/open-project/useOpenProject";
import { branchPrefix, pickShort } from "../../features/start/summary";
import type { RecentProject } from "../../lib/api";
import { useProjectSummary, useRoutePreview } from "../../lib/queries";
import { useInView } from "../../lib/useInView";
import { Time } from "../../ui/Time";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import styles from "./Start.module.css";
import { Truncate } from "../../ui/Truncate";

/**
 * 다른 프로젝트 한 줄: 이름 · 작은 노선도 · 짧은 상태 · 마지막으로 연 때. 줄 전체가 열기 버튼이다.
 * 작은 노선도는 줄이 화면에 들어오고 상태 요약이 성공했을 때만 부른다(D-093).
 */
export function ProjectRow({
  project,
  now,
  onRemove,
}: {
  project: RecentProject;
  now: number;
  onRemove: () => void;
}) {
  const m = useMessages();
  const open = useOpenProject();
  const summary = useProjectSummary(project.path);
  const gone =
    summary.error?.code === "folderNotFound"
      ? m.folderMissing
      : summary.error?.code === "needsTrust"
        ? m.trustNeeded
        : summary.error
          ? m.folderNotRepo
          : null;

  const row = useRef<HTMLLIElement>(null);
  const inView = useInView(row);
  const preview = useRoutePreview(project.path, 6, inView && !gone && summary.data?.branch.hasCommits === true);
  const map = preview.data;

  const short = summary.data ? pickShort(summary.data) : null;
  const prefix = summary.data ? branchPrefix(summary.data) : null;
  const status = short ? `${prefix ? `${prefix} · ` : ""}${m.shortStatus(short)}` : "";

  return (
    <li ref={row} className={styles.row}>
      <button
        type="button"
        className={styles.rowButton}
        disabled={gone !== null || open.isPending}
        onClick={() => void open.open(project.path, "recent")}
      >
        <Truncate full={project.path} className={gone ? `${styles.rowName} ${styles.muted}` : styles.rowName}>
          {project.name}
        </Truncate>
        {/* 장식: 상태 글자가 같은 정보를 준다. 비어 있어도 자리는 둔다. */}
        <span className={styles.rowMap} aria-hidden="true">
          {map && map.currentCommits.length > 0 && <RoutePreview view={map} density="mini" />}
        </span>
        <span className={short?.kind === "conflict" ? `${styles.rowStatus} ${styles.danger}` : styles.rowStatus}>
          {gone ?? status}
        </span>
      </button>
      {gone ? (
        <Button variant="text" onClick={onRemove}>
          {m.removeFromList}
        </Button>
      ) : (
        <span className={styles.rowWhen}>
          <Time at={project.lastOpenedAt} now={now} />
        </span>
      )}
    </li>
  );
}
