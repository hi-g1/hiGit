import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState, type KeyboardEvent, type ReactNode } from "react";
import { ApiError, api, type ActionInput, type BranchListView } from "../../lib/api";
import { useBranchNameCheck, useBranches, useMergeHint } from "../../lib/queries";
import { formatDate } from "../../lib/time";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { TermTooltip, useTermLabel } from "../../ui/Term";
import { useMergeAction } from "../merge/useMergeAction";
import { useToastStore } from "../../ui/toastStore";
import styles from "./BranchList.module.css";
import { branchColor } from "./color";
import { canOfferCreate, rightText, search, sections, type ListItem } from "./model";

type Props = {
  repo: string;
  /** 지금 브랜치 이름(status에서). 특정 버전을 보는 중이면 null. */
  current: string | null;
  /** 바로 한 일의 결과와 되돌리기를 띄운다(useUndo의 notifyDone). */
  onDone: (text: string) => void;
  /** 상단 바 단추 안에 보일 것. */
  children: ReactNode;
};

type Choice = { kind: "switch"; item: ListItem } | { kind: "create"; name: string };

/**
 * 브랜치 목록(D-047, r-branch.png). 고르면 묻지 않고 바로 갈아타고, 찾기 칸에 없는 이름을 쓰고
 * Enter를 누르면 새 브랜치를 만든다. Ctrl+B로 연다.
 */
export function BranchList({ repo, current, onDone, children }: Props) {
  const [open, setOpen] = useState(false);
  useShortcut("branches", () => setOpen((o) => !o));
  return (
    <Popover.Root
      open={open}
      onOpenChange={setOpen}
    >
      <Popover.Trigger asChild>
        <button type="button" className={styles.trigger} aria-keyshortcuts="Control+B">
          {children}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content className={styles.panel} align="start" sideOffset={6}>
          {open && <Panel repo={repo} current={current} onDone={onDone} close={() => setOpen(false)} />}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

function Panel({ repo, current, onDone, close }: Omit<Props, "children"> & { close: () => void }) {
  const m = useMessages();
  const branches = useBranches(repo);
  const [query, setQuery] = useState("");
  const [expanded, setExpanded] = useState(false);
  const [active, setActive] = useState(0);
  const [pending, setPending] = useState(false);
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const view = branches.data;
  // 지금 브랜치만 앞서 있으면 "main에 합치기"(D-071). 노선도 옆이 아니라 여기에 둔다(D-048).
  const hint = useMergeHint(repo).data;
  const merge = useMergeAction(repo, onDone);
  const debounced = useDebounced(query.trim(), 300);
  const nameCheck = useBranchNameCheck(repo, view && canOfferCreate(view, debounced) ? debounced : "");

  const choices = useMemo<Choice[]>(() => {
    if (!view) return [];
    if (query.trim()) {
      const found = search(view, query).map((item) => ({ kind: "switch" as const, item }));
      return canOfferCreate(view, query) ? [...found, { kind: "create", name: query.trim() }] : found;
    }
    const s = sections(view);
    const visible = [...(s.current ? [s.current] : []), ...s.recent, ...(expanded ? s.more : [])];
    return visible.map((item) => ({ kind: "switch", item }));
  }, [view, query, expanded]);

  const createBlocked =
    debounced !== query.trim() || nameCheck.isFetching || (nameCheck.data !== undefined && !nameCheck.data.ok);

  const run = async (choice: Choice) => {
    if (choice.kind === "switch" && choice.item.remote === null && choice.item.name === current) {
      close();
      return;
    }
    if (choice.kind === "create" && createBlocked) return;
    const input: ActionInput =
      choice.kind === "create"
        ? { id: "branch-create", input: { name: choice.name } }
        : { id: "switch", input: { branch: choice.item.name, remote: choice.item.remote } };
    setPending(true);
    try {
      await api.executeAction(repo, input);
      close();
      onDone(choice.kind === "create" ? m.createdBranch(choice.name) : m.switchedTo(choice.item.name));
    } catch (e) {
      show(m.errors[e instanceof ApiError ? e.code : "internal"]);
    } finally {
      setPending(false);
      for (const key of [["status", repo], ["log", repo], ["branches", repo]]) {
        void queryClient.invalidateQueries({ queryKey: key });
      }
    }
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, choices.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      // 한글 조합 중 Enter는 글자를 확정하는 키라서 실행하지 않는다.
      if (e.nativeEvent.isComposing || e.nativeEvent.keyCode === 229) return;
      e.preventDefault();
      const choice = choices[active];
      if (choice && !pending) void run(choice);
    }
  };

  return (
    <div className={styles.inner}>
      <input
        className={styles.search}
        value={query}
        placeholder={m.branchSearch}
        aria-label={m.branchSearch}
        autoFocus
        onChange={(e) => {
          setQuery(e.target.value);
          setActive(0);
        }}
        onKeyDown={onKeyDown}
      />
      {view && <List view={view} choices={choices} query={query} active={active} onPick={(c) => void run(c)} />}
      {!query.trim() && hint?.kind === "into" && hint.branch !== null && (
        <div className={styles.mergeInto}>
          <TermTooltip id="merge">
            <Button
              disabled={merge.pending}
              onClick={() => {
                const target = hint.branch;
                if (target === null) return;
                void merge.run("merge-into", target).then(close);
              }}
            >
              {m.mergeIntoBranch(hint.branch)}
            </Button>
          </TermTooltip>
        </div>
      )}
      {view && !query.trim() && !expanded && sections(view).more.length > 0 && (
        <button type="button" className={styles.more} onClick={() => setExpanded(true)}>
          {m.moreBranches(sections(view).more.length)}
        </button>
      )}
      {choices.some((c) => c.kind === "create") && nameCheck.data && !nameCheck.data.ok && (
        <p className={styles.hint}>
          {nameCheck.data.problem === "exists" ? m.sameBranchName : m.badBranchName}
        </p>
      )}
    </div>
  );
}

function List({
  view,
  choices,
  query,
  active,
  onPick,
}: {
  view: BranchListView;
  choices: Choice[];
  query: string;
  active: number;
  onPick: (c: Choice) => void;
}) {
  const m = useMessages();
  const branchLabel = useTermLabel("branch");
  const locale = useUiStore((s) => s.locale);
  // 목록을 여는 동안의 기준 시각(그릴 때마다 바뀌지 않게).
  const [now] = useState(() => Date.now());
  const right = (item: ListItem) => {
    const r = rightText(item.row);
    switch (r.kind) {
      case "parkBlocked":
        return m.parkedBlocked(r.count);
      case "parked":
        return m.parkedFiles(r.count);
      case "newVersions":
        return m.newVersions(r.count);
      case "author":
        return `${r.author} · ${formatDate(r.at, now, locale)}`;
    }
  };
  const s = sections(view);
  const searching = query.trim() !== "";
  return (
    <ul className={styles.list} role="listbox" aria-label={branchLabel}>
      {choices.map((c, i) => {
        const first = s.recent[0];
        const header =
          !searching && i === 0 && s.current
            ? m.currentSection
            : !searching && c.kind === "switch" && first && c.item.name === first.name && c.item.remote === first.remote
              ? m.recentSection
              : null;
        return (
          <li key={c.kind === "create" ? "+create" : `${c.item.remote ?? ""}/${c.item.name}`} role="presentation">
            {header && <p className={styles.section}>{header}</p>}
            <button
              type="button"
              role="option"
              aria-selected={i === active}
              className={`${styles.row} ${i === active ? styles.active : ""} ${c.kind === "create" ? styles.create : ""}`}
              onClick={() => onPick(c)}
            >
              {c.kind === "create" ? (
                <>
                  <span className={styles.name}>{m.createBranch(c.name)}</span>
                  <span className={styles.right}>Enter</span>
                </>
              ) : (
                <>
                  <span className={styles.dot} style={{ background: branchColor(c.item.name, view.defaultBranch) }} />
                  <span className={styles.name}>{c.item.remote ? `${c.item.remote}/${c.item.name}` : c.item.name}</span>
                  <span className={styles.right}>{right(c.item)}</span>
                </>
              )}
            </button>
          </li>
        );
      })}
      {!searching &&
        view.orphanParked.map((name) => (
          <li key={`orphan/${name}`} className={styles.orphan}>
            <span className={styles.name}>{name}</span>
            <span className={styles.right}>{m.parkedOnly}</span>
          </li>
        ))}
    </ul>
  );
}

function useDebounced(value: string, ms: number): string {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), ms);
    return () => clearTimeout(timer);
  }, [value, ms]);
  return debounced;
}
