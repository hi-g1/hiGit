import type { RepoInfo, StatusView } from "../../lib/api";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { Term } from "../../ui/Term";
import styles from "./TopBar.module.css";

/**
 * 상단 바(D-025). 왼쪽: 프로젝트 이름, 브랜치, 올릴·가져올 버전 수. 오른쪽: 다른 폴더 열기.
 * 브랜치 단추는 1b에서 브랜치 목록 드롭다운이 된다. 지금은 표시만 한다.
 */
export function TopBar({ repo, status }: { repo: RepoInfo; status: StatusView | undefined }) {
  const m = useMessages();
  const closeRepo = useRepoStore((s) => s.closeRepo);
  const branch = status?.branch;

  return (
    <header className={styles.bar}>
      <h1 className={styles.name} title={repo.path}>
        {repo.name}
      </h1>
      {branch && (
        <span className={styles.branch}>
          <Term id="branch" />
          <span className={styles.branchName}>{branch.name ?? m.detached}</span>
        </span>
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
      <Button variant="text" onClick={closeRepo}>
        {m.openOtherFolder}
      </Button>
    </header>
  );
}
