import { useRef, useState } from "react";
import type { GitHubRepoRowView } from "../../lib/api";
import { listKey } from "../../lib/listKeys";
import { useGithubRepos } from "../../lib/queries";
import { formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./CloneDialog.module.css";

/**
 * 복제 대화상자의 내 저장소 목록(D-133, design.md "복제 대화상자"). 보는 동안만 읽고 고르면 `onPick`.
 * 글자는 React 글자로만 그린다. 주소는 부르는 쪽이 만들고 core가 복제 때 다시 검사한다.
 */
export function RepoList({ onPick }: { onPick: (repo: GitHubRepoRowView) => void }) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const repos = useGithubRepos(true);
  const rows = repos.data?.pages.flatMap((p) => p.items) ?? [];
  const [active, setActive] = useState(0);
  const list = useRef<HTMLUListElement>(null);
  const [now] = useState(() => Date.now());

  const move = (index: number) => {
    const next = Math.max(0, Math.min(rows.length - 1, index));
    setActive(next);
    list.current?.children[next]?.scrollIntoView({ block: "nearest" });
  };

  if (repos.error) return <ErrorNotice error={repos.error} />;
  if (repos.isPending) return <p className={styles.target}>{m.clone.loadingRepos}</p>;
  if (rows.length === 0) return <p className={styles.target}>{m.clone.noRepos}</p>;
  return (
    <div className={styles.repos}>
      <ul
        ref={list}
        role="listbox"
        aria-label={m.clone.myRepos}
        aria-activedescendant={`repo-${active}`}
        tabIndex={0}
        className={styles.repoList}
        autoFocus
        onKeyDown={(e) => {
          const key = listKey(e.nativeEvent);
          if (!key) return;
          e.preventDefault();
          if (key === "next") move(active + 1);
          else if (key === "prev") move(active - 1);
          else if (key === "pageNext") move(active + 8);
          else if (key === "pagePrev") move(active - 8);
          else if (key === "first") move(0);
          else if (key === "last") move(rows.length - 1);
          else if (key === "open" && rows[active]) onPick(rows[active]);
        }}
      >
        {rows.map((r, i) => (
          <li
            key={`${r.owner}/${r.name}`}
            id={`repo-${i}`}
            role="option"
            aria-selected={i === active}
            className={styles.repo}
            onMouseEnter={() => setActive(i)}
            onClick={() => onPick(r)}
          >
            <bdi className={styles.repoName}>
              {r.owner}/{r.name}
            </bdi>
            {r.private && <span className={styles.repoMeta}>{m.clone.privateRepo}</span>}
            {r.pushedAt !== null && <span className={styles.repoMeta}>{formatDate(r.pushedAt, now, locale)}</span>}
          </li>
        ))}
      </ul>
      {repos.hasNextPage && (
        <Button variant="text" disabled={repos.isFetchingNextPage} onClick={() => void repos.fetchNextPage()}>
          {m.clone.more}
        </Button>
      )}
    </div>
  );
}
