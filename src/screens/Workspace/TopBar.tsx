import { BranchList } from "../../features/branches/BranchList";
import { branchColor } from "../../features/branches/color";
import type { RepoInfo, StatusView } from "../../lib/api";
import { useBranches } from "../../lib/queries";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Term } from "../../ui/Term";
import { ProjectMenu } from "./ProjectMenu";
import styles from "./TopBar.module.css";

/**
 * 상단 바(D-045). 왼쪽: 프로젝트 메뉴, 브랜치 단추(누르거나 Ctrl+B면 브랜치 목록).
 * 오른쪽: 동기화(H15), 버전 기록(Ctrl+H, D-058). "작업 내역"은 반복 2, "설정"은 설정 화면과 함께.
 */
export function TopBar({
  repo,
  status,
  onDone,
}: {
  repo: RepoInfo;
  status: StatusView | undefined;
  onDone: (text: string) => void;
}) {
  const m = useMessages();
  const branch = status?.branch;
  const defaultBranch = useBranches(repo.path).data?.defaultBranch ?? null;
  const view = useRepoStore((s) => s.view);
  const setView = useRepoStore((s) => s.setView);
  const toggleHistory = () => setView(view === "history" ? "home" : "history");
  useShortcut("history", toggleHistory);

  return (
    <header className={styles.bar}>
      <ProjectMenu repo={repo} />
      <span className={styles.divider} aria-hidden="true" />
      {branch && (
        <BranchList repo={repo.path} current={branch.name} onDone={onDone}>
          {branch.name && <span className={styles.dot} style={{ background: branchColor(branch.name, defaultBranch) }} />}
          <span className={styles.branchName}>{branch.name ?? m.detached}</span>
          <span aria-hidden="true">▾</span>
        </BranchList>
      )}
      <span className={styles.spacer} />
      <button
        type="button"
        className={`${styles.textButton} ${view === "history" ? styles.on : ""}`}
        aria-pressed={view === "history"}
        aria-keyshortcuts="Control+H"
        onClick={toggleHistory}
      >
        <Term id="log" focusable={false} />
      </button>
    </header>
  );
}
