import { useEffect, useMemo, useRef, useState } from "react";
import type { GitHubRepoRowView } from "../../lib/api";
import { listKey } from "../../lib/listKeys";
import { useGithubRepos } from "../../lib/queries";
import { matchesSearch } from "../../lib/search";
import { formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./CloneDialog.module.css";
import { composing, isEnter } from "../../lib/keys";

/** 찾기 칸에 글자가 있을 때 저절로 더 읽는 쪽 수(50개씩, 500개까지, D-185). */
const SEARCH_PAGES = 10;

/**
 * 복제 대화상자의 내 저장소 목록(D-133, design.md "복제 대화상자"). 보는 동안만 읽고 고르면 `onPick`.
 * 위 찾기 칸은 `소유자/이름`에 글자가 든 줄만 남기고, 글자가 있는 동안 500개까지 한 쪽씩 더 읽는다(D-185).
 * 글자는 React 글자로만 그린다. 주소는 부르는 쪽이 만들고 core가 복제 때 다시 검사한다.
 */
export function RepoList({ onPick }: { onPick: (repo: GitHubRepoRowView) => void }) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const repos = useGithubRepos(true);
  const data = repos.data;
  const pageCount = data?.pages.length ?? 0;
  const [query, setQuery] = useState("");
  const all = useMemo(() => data?.pages.flatMap((p) => p.items) ?? [], [data]);
  const rows = useMemo(() => all.filter((r) => matchesSearch(`${r.owner}/${r.name}`, query)), [all, query]);
  const [active, setActive] = useState(0);
  const list = useRef<HTMLUListElement>(null);
  const [now] = useState(() => Date.now());
  const searching = query.trim() !== "";
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = repos;
  const capped = pageCount >= SEARCH_PAGES;
  useEffect(() => {
    if (searching && hasNextPage && !isFetchingNextPage && !capped) void fetchNextPage();
  }, [searching, hasNextPage, isFetchingNextPage, capped, fetchNextPage]);

  const move = (index: number) => {
    const next = Math.max(0, Math.min(rows.length - 1, index));
    setActive(next);
    list.current?.children[next]?.scrollIntoView({ block: "nearest" });
  };
  const onListKey = (e: React.KeyboardEvent) => {
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
  };

  if (repos.error) return <ErrorNotice error={repos.error} />;
  if (repos.isPending) return <p className={styles.target}>{m.clone.loadingRepos}</p>;
  if (all.length === 0) return <p className={styles.target}>{m.clone.noRepos}</p>;
  return (
    <div className={styles.repos}>
      <input
        className={styles.field}
        role="searchbox"
        aria-label={m.clone.searchRepos}
        placeholder={m.clone.searchRepos}
        value={query}
        autoFocus
        spellCheck={false}
        onChange={(e) => {
          setQuery(e.target.value);
          setActive(0);
        }}
        onKeyDown={(e) => {
          if (composing(e)) return;
          // ↓는 목록 첫 줄로, Enter는 지금 줄을 고른다.
          if (e.key === "ArrowDown" && rows.length > 0) {
            e.preventDefault();
            list.current?.focus();
            move(0);
          } else if (isEnter(e) && rows[active]) {
            e.preventDefault();
            onPick(rows[active]);
          }
        }}
      />
      {searching && hasNextPage && !capped && <p className={styles.target}>{m.clone.searching(all.length)}</p>}
      {searching && hasNextPage && capped && <p className={styles.target}>{m.clone.tooMany}</p>}
      {rows.length === 0 ? (
        <p className={styles.target}>{m.clone.noMatch}</p>
      ) : (
        <ul
          ref={list}
          role="listbox"
          aria-label={m.clone.myRepos}
          aria-activedescendant={`repo-${active}`}
          tabIndex={0}
          className={styles.repoList}
          onKeyDown={onListKey}
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
      )}
      {!searching && hasNextPage && (
        <Button variant="text" disabled={isFetchingNextPage} onClick={() => void fetchNextPage()}>
          {m.clone.more}
        </Button>
      )}
    </div>
  );
}
