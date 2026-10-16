import { BranchList } from "../../features/branches/BranchList";
import { branchColor } from "../../features/branches/color";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { SyncButton } from "../../features/sync/SyncButton";
import type { RepoInfo, StatusView } from "../../lib/api";
import { useBranches, useGitHubStatus, useHostingContext } from "../../lib/queries";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Term } from "../../ui/Term";
import { ProjectMenu } from "./ProjectMenu";
import styles from "./TopBar.module.css";
import { Truncate } from "../../ui/Truncate";

/**
 * 상단 바(D-045). 왼쪽: 프로젝트 메뉴, 브랜치 단추(누르거나 Ctrl+B면 브랜치 목록).
 * 오른쪽: 동기화(H15)(원격이 없고 GitHub에 연결 안 됐으면 그 자리에 "GitHub 연결"), 버전 기록
 * (Ctrl+H, D-058), 작업 내역(Ctrl+Shift+H), 설정. 오른쪽 컨트롤은 다섯 개를 넘지 않는다.
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
  const github = useGitHubStatus().data;
  const hosting = useHostingContext(repo.path, hasRemote).data;
  const showPr = hasRemote && hosting?.provider === "github" && !merging;
  const togglePr = () => setView(view === "pr" ? "home" : "pr");
  const openSettings = useSettingsStore((s) => s.openSettings);
  const view = useRepoStore((s) => s.view);
  const setView = useRepoStore((s) => s.setView);
  const toggleHistory = () => setView(view === "history" ? "home" : "history");
  const toggleActivity = () => setView(view === "activity" ? "home" : "activity");
  useShortcut("history", toggleHistory, !merging);
  useShortcut("activity", toggleActivity, !merging);

  return (
    <header className={styles.bar}>
      <div className={styles.left}>
        <ProjectMenu repo={repo} />
        <span className={styles.divider} aria-hidden="true" />
        {branch && merging && <Truncate className={styles.branchName}>{branch.name ?? m.detached}</Truncate>}
        {branch && !merging && (
          <BranchList repo={repo.path} current={branch.name} onDone={onDone}>
            {branch.name && <span className={styles.dot} style={{ background: branchColor(branch.name, defaultBranch) }} />}
            <Truncate className={styles.branchName}>{branch.name ?? m.detached}</Truncate>
            <span aria-hidden="true">▾</span>
          </BranchList>
        )}
      </div>
      <div className={styles.right}>
        {branch && hasRemote && !merging && (
          <SyncButton repo={repo.path} branch={branch} changedFiles={status?.files.length ?? 0} />
        )}
        {branches && !hasRemote && github?.configured && !github.connected && !merging && (
          <button type="button" className={styles.textButton} onClick={() => openSettings(true)}>
            {m.settings.connect}
          </button>
        )}
        {showPr && (
          <button
            type="button"
            className={`${styles.textButton} ${view === "pr" ? styles.on : ""}`}
            aria-pressed={view === "pr"}
            onClick={togglePr}
          >
            <Term id="pr" focusable={false} />
          </button>
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
        <button type="button" className={styles.textButton} onClick={() => openSettings()}>
          {m.settings.title}
        </button>
      </div>
    </header>
  );
}
