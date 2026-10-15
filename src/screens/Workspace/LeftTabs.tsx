import type { KeyboardEvent } from "react";
import { useMessages } from "../../messages";
import { useRepoStore, type LeftTab } from "../../state/repoStore";
import { Term } from "../../ui/Term";
import styles from "./LeftTabs.module.css";

const TABS: LeftTab[] = ["changes", "history"];

/** 왼쪽 칸 탭 "변경 사항 N | 버전 기록". 좌우 화살표로 옮긴다. */
export function LeftTabs({ changeCount }: { changeCount: number | undefined }) {
  const m = useMessages();
  const tab = useRepoStore((s) => s.leftTab);
  const setTab = useRepoStore((s) => s.setLeftTab);

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
    e.preventDefault();
    const next = TABS[(TABS.indexOf(tab) + (e.key === "ArrowRight" ? 1 : TABS.length - 1)) % TABS.length]!;
    setTab(next);
    document.getElementById(`tab-${next}`)?.focus();
  };

  return (
    <div role="tablist" aria-label={m.workspaceTabs} className={styles.tabs} onKeyDown={onKeyDown}>
      {TABS.map((id) => (
        <button
          key={id}
          id={`tab-${id}`}
          type="button"
          role="tab"
          aria-selected={tab === id}
          aria-controls={`panel-${id}`}
          tabIndex={tab === id ? 0 : -1}
          className={styles.tab}
          onClick={() => setTab(id)}
        >
          {id === "changes" ? (
            <>
              <Term id="status" focusable={false} />
              {changeCount !== undefined && <span className={styles.count}>{changeCount}</span>}
            </>
          ) : (
            <Term id="log" focusable={false} />
          )}
        </button>
      ))}
    </div>
  );
}
