import type { RepoInfo, StatusView } from "../../lib/api";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import { ProjectMenu } from "./ProjectMenu";
import styles from "./TopBar.module.css";

/**
 * 상단 바(D-025). 왼쪽: 프로젝트 메뉴(이름), 브랜치, 올릴·가져올 버전 수.
 * 오른쪽은 1b 20단계의 새로 고침·가져오기·올리기 자리다.
 * 브랜치 단추는 1b에서 브랜치 목록 드롭다운이 된다. 지금은 표시만 한다.
 */
export function TopBar({ repo, status }: { repo: RepoInfo; status: StatusView | undefined }) {
  const m = useMessages();
  const branch = status?.branch;

  return (
    <header className={styles.bar}>
      <ProjectMenu repo={repo} />
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
    </header>
  );
}
