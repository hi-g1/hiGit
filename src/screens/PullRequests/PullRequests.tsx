import { useEffect, useMemo, useRef, useState } from "react";
import { PrRow } from "../../features/pr/PrRow";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { toApiError, type PrStateFilterView } from "../../lib/api";
import { listKey } from "../../lib/listKeys";
import { useGitHubStatus, useHostingContext, usePrPages } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import styles from "./PullRequests.module.css";

const FILTERS: PrStateFilterView[] = ["open", "closed", "all"];

/**
 * 검토 요청 화면(D-112). 열린 동안만 읽고(60초 동안 다시 읽지 않음, 창 포커스 때 새로), 폴링하지 않는다.
 * 줄을 고르면 그 아래로 합칠 수 있는지와 시험 결과가 펼쳐진다. Esc는 홈.
 */
export function PullRequests({ repo }: { repo: string }) {
  const m = useMessages();
  const status = useGitHubStatus().data;
  const context = useHostingContext(repo).data;
  const openSettings = useSettingsStore((s) => s.openSettings);
  const setView = useRepoStore((s) => s.setView);
  const selected = useRepoStore((s) => s.selectedPr);
  const select = useRepoStore((s) => s.selectPr);
  const [filter, setFilter] = useState<PrStateFilterView>("open");
  const connected = status?.connected === true;
  const pages = usePrPages(repo, filter, connected);
  const rows = useMemo(() => pages.data?.pages.flatMap((p) => p.items) ?? [], [pages.data]);
  const [now] = useState(() => Date.now());
  const listRef = useRef<HTMLUListElement>(null);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented) setView("home");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setView]);

  const move = (delta: number) => {
    if (rows.length === 0) return;
    const index = rows.findIndex((r) => r.number === selected);
    const next = rows[index < 0 ? 0 : Math.min(rows.length - 1, Math.max(0, index + delta))];
    if (!next) return;
    select(next.number);
    document.getElementById(`pr-${next.number}`)?.scrollIntoView?.({ block: "nearest" });
  };

  const error = pages.error;
  const expired = error !== null && toApiError(error).code === "githubExpired";

  return (
    <section className={styles.screen}>
      <header className={styles.head}>
        <h2 className={styles.title}>
          <Term id="pr" />
        </h2>
        <div className={styles.filters} role="group" aria-label={m.pr.filter}>
          {FILTERS.map((f) => (
            <button
              key={f}
              type="button"
              className={`${styles.filter} ${filter === f ? styles.on : ""}`}
              aria-pressed={filter === f}
              onClick={() => setFilter(f)}
            >
              {m.pr[f]}
            </button>
          ))}
        </div>
        <span className={styles.spacer} />
        {context?.repo && <bdi className={styles.repo}>{context.repo}</bdi>}
      </header>
      <div className={styles.body}>
        {status && !status.configured && <p className={styles.muted}>{m.settings.notConfigured}</p>}
        {status?.configured && !status.connected && (
          <EmptyState
            action={
              <Button variant="primary" onClick={() => openSettings(true)}>
                {m.settings.connect}
              </Button>
            }
          >
            {m.pr.connectToSee}
          </EmptyState>
        )}
        {error && (
          <ErrorNotice error={error}>
            {expired && (
              <Button variant="secondary" onClick={() => openSettings(true)}>
                {m.pr.reconnect}
              </Button>
            )}
          </ErrorNotice>
        )}
        {connected && pages.isSuccess && rows.length === 0 && <EmptyState>{m.pr.empty}</EmptyState>}
        {rows.length > 0 && (
          <ul
            ref={listRef}
            className={styles.list}
            role="listbox"
            aria-label={m.pr.filter}
            tabIndex={0}
            aria-activedescendant={selected !== null ? `pr-${selected}` : undefined}
            onKeyDown={(e) => {
              const action = listKey(e);
              if (action === "next" || action === "prev") {
                e.preventDefault();
                move(action === "next" ? 1 : -1);
              } else if (action === "first" || action === "last") {
                e.preventDefault();
                move(action === "first" ? -rows.length : rows.length);
              }
            }}
          >
            {rows.map((row) => (
              <PrRow
                key={row.number}
                row={row}
                repo={repo}
                now={now}
                selected={row.number === selected}
                onSelect={() => select(row.number === selected ? null : row.number)}
              />
            ))}
          </ul>
        )}
        {pages.hasNextPage && (
          <div className={styles.more}>
            <Button variant="text" disabled={pages.isFetchingNextPage} onClick={() => void pages.fetchNextPage()}>
              {m.pr.more}
            </Button>
          </div>
        )}
      </div>
    </section>
  );
}
