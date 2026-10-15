import { BranchList } from "../../features/branches/BranchList";
import { branchColor } from "../../features/branches/color";
import type { RepoInfo, StatusView } from "../../lib/api";
import { useBranches } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import { ProjectMenu } from "./ProjectMenu";
import styles from "./TopBar.module.css";

/**
 * 상단 바(D-025, D-045). 왼쪽: 프로젝트 메뉴(이름), 브랜치 단추(누르거나 Ctrl+B면 브랜치 목록).
 * 오른쪽 동기화·버전 기록은 계획 005 H14·H15에서 채운다.
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

  return (
    <header className={styles.bar}>
      <ProjectMenu repo={repo} />
      {branch && (
        <BranchList repo={repo.path} current={branch.name} onDone={onDone}>
          {branch.name && <span className={styles.dot} style={{ background: branchColor(branch.name, defaultBranch) }} />}
          <span className={styles.branchName}>{branch.name ?? m.detached}</span>
          <span aria-hidden="true">▾</span>
        </BranchList>
      )}
      {branch?.hasCommits &&
        (branch.upstream ? (
          <span className={styles.sync}>
            <span>
              <Term id="push" /> {branch.ahead}
            </span>
            <span>
              <Term id="pull" /> {branch.behind}
            </span>
          </span>
        ) : (
          <span className={styles.sync}>{m.notUploaded}</span>
        ))}
      <span className={styles.spacer} />
    </header>
  );
}
