import { useOpenProject } from "../../features/open-project/useOpenProject";
import { branchPrefix, pickShort } from "../../features/start/summary";
import type { RecentProject } from "../../lib/api";
import { useProjectSummary } from "../../lib/queries";
import { formatRelative } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import styles from "./Start.module.css";

/** 다른 프로젝트 한 줄: 이름 · 짧은 상태 · 마지막으로 연 때. 줄 전체가 열기 버튼이다. */
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
  const locale = useUiStore((s) => s.locale);
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

  const short = summary.data ? pickShort(summary.data) : null;
  const prefix = summary.data ? branchPrefix(summary.data) : null;
  const status = short ? `${prefix ? `${prefix} · ` : ""}${m.shortStatus(short)}` : "";

  return (
    <li className={styles.row}>
      <button
        type="button"
        className={styles.rowButton}
        disabled={gone !== null || open.isPending}
        onClick={() => void open.open(project.path, "recent")}
      >
        <span className={gone ? `${styles.rowName} ${styles.muted}` : styles.rowName}>{project.name}</span>
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
          {formatRelative(project.lastOpenedAt, now, locale)}
        </span>
      )}
    </li>
  );
}
