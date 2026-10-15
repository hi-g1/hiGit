import { BranchList } from "../../features/branches/BranchList";
import { branchColor } from "../../features/branches/color";
import { SyncButton } from "../../features/sync/SyncButton";
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
  merging = false,
}: {
  repo: RepoInfo;
  status: StatusView | undefined;
  onDone: (text: string) => void;
  /** 합치는 중이면 브랜치 목록·동기화·버전 기록을 숨긴다(그 단축키도 꺼진다). */
  merging?: boolean;
}) {
  const m = useMessages();
  const branch = status?.branch;
  const branches = useBranches(repo.path).data;
  const defaultBranch = branches?.defaultBranch ?? null;
  const hasRemote = (branches?.remotes.length ?? 0) > 0;
  const view = useRepoStore((s) => s.view);
  const setView = useRepoStore((s) => s.setView);
  const toggleHistory = () => setView(view === "history" ? "home" : "history");
  const toggleActivity = () => setView(view === "activity" ? "home" : "activity");
  useShortcut("history", toggleHistory, !merging);
  useShortcut("activity", toggleActivity, !merging);

  return (
    <header className={styles.bar}>
      <ProjectMenu repo={repo} />
      <span className={styles.divider} aria-hidden="true" />
      {branch && merging && (
        <span className={styles.branchName}>{branch.name ?? m.detached}</span>
      )}
      {branch && !merging && (
        <BranchList repo={repo.path} current={branch.name} onDone={onDone}>
          {branch.name && <span className={styles.dot} style={{ background: branchColor(branch.name, defaultBranch) }} />}
          <span className={styles.branchName}>{branch.name ?? m.detached}</span>
          <span aria-hidden="true">▾</span>
        </BranchList>
      )}
      <span className={styles.spacer} />
      {branch && hasRemote && !merging && (
        <SyncButton repo={repo.path} branch={branch} changedFiles={status?.files.length ?? 0} />
      )}
      {!merging && (
      <button
        type="button"
        className={`${styles.textButton} ${view === "history" ? styles.on : ""}`}
        aria-pressed={view === "history"}
        aria-keyshortcuts="Control+H"
        onClick={toggleHistory}
      >
        <Term id="log" focusable={false} />
      </button>
      )}
      {!merging && (
        <button
          type="button"
          className={`${styles.textButton} ${view === "activity" ? styles.on : ""}`}
          aria-pressed={view === "activity"}
          aria-keyshortcuts="Control+Shift+H"
          onClick={toggleActivity}
        >
          <Term id="reflog" focusable={false} />
        </button>
      )}
    </header>
  );
}
