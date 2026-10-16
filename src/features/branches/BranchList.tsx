import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import { api, type ActionInput, type BranchListView } from "../../lib/api";
import {
  invalidateLog,
  useBranchNameCheck,
  useBranches,
  useHostingContext,
  useMergeHint,
  useStatus,
} from "../../lib/queries";
import { formatTime } from "../../lib/time";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { Kbd } from "../../ui/Kbd";
import { Tooltip } from "../../ui/Tooltip";
import { TermTooltip, useTermLabel } from "../../ui/Term";
import { useMergeAction } from "../merge/useMergeAction";
import { PrSlot } from "../pr/PrSlot";
import { useToastStore } from "../../ui/toastStore";
import styles from "./BranchList.module.css";
import { branchColor } from "./color";
import { RemoteBranchMenu } from "../remote-branch/RemoteBranchMenu";
import { BranchRowMenu } from "./BranchRowMenu";
import { canOfferCreate, rightText, search, sections, type ListItem } from "./model";
import { SwitchDialog } from "./SwitchDialog";
import { useSwitch, type SwitchTarget } from "./useSwitch";
import { Truncate } from "../../ui/Truncate";

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
 * 브랜치 목록(D-047, r-branch.png). 고르면 바로 갈아타고, 고친 파일이 있으면 목록을 닫고 두고 갈지 옮길지
 * 묻는다(D-195). 찾기 칸에 없는 이름을 쓰고 Enter를 누르면 새 브랜치를 만든다. Ctrl+B로 연다.
 */
export function BranchList({ repo, current, onDone, children }: Props) {
  const [open, setOpen] = useState(false);
  // 이름을 바꾸는 줄(로컬 브랜치 이름). Esc는 목록을 닫지 않고 이름 바꾸기만 그만둔다.
  const [renaming, setRenaming] = useState<string | null>(null);
  // 고친 파일이 있어 묻는 중인 갈아타기. 목록(팝오버) 밖에 두어 목록이 닫혀도 남는다.
  const [asking, setAsking] = useState<{ target: SwitchTarget; files: number } | null>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const sw = useSwitch(repo, onDone);
  const m = useMessages();
  useShortcut("branches", () => setOpen((o) => !o));
  const stopAsking = () => {
    setAsking(null);
    triggerRef.current?.focus();
  };
  return (
    <>
      {asking && current !== null && (
        <SwitchDialog
          repo={repo}
          target={asking.target}
          from={current}
          files={asking.files}
          pending={sw.pending === "carry" || sw.pending === "park" ? sw.pending : null}
          onChoose={(dirty) => {
            void sw.run(asking.target, dirty).then((ok) => ok && stopAsking());
          }}
          onClose={stopAsking}
        />
      )}
      <Popover.Root
        open={open}
        onOpenChange={(o) => {
          setOpen(o);
          if (!o) setRenaming(null);
        }}
      >
        <Tooltip
          content={
            <span className={styles.tip}>
              {m.shortcutName.branches}
              <Kbd>{SHORTCUTS.branches.label}</Kbd>
            </span>
          }
        >
          <Popover.Trigger asChild>
            <button ref={triggerRef} type="button" className={styles.trigger} aria-keyshortcuts="Control+B">
              {children}
            </button>
          </Popover.Trigger>
        </Tooltip>
        <Popover.Portal>
          <Popover.Content
            className={styles.panel}
            align="start"
            sideOffset={6}
            onEscapeKeyDown={(e) => {
              if (renaming !== null) {
                e.preventDefault();
                setRenaming(null);
              }
            }}
          >
            {open && (
              <Panel
                repo={repo}
                current={current}
                onDone={onDone}
                close={() => setOpen(false)}
                renaming={renaming}
                setRenaming={setRenaming}
                ask={(target, files) => setAsking({ target, files })}
              />
            )}
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>
    </>
  );
}

type PanelProps = Omit<Props, "children"> & {
  close: () => void;
  renaming: string | null;
  setRenaming: (name: string | null) => void;
  /** 고친 파일이 있을 때 갈아타기를 대화상자로 넘긴다. */
  ask: (target: SwitchTarget, files: number) => void;
};

function Panel({ repo, current, onDone, close, renaming, setRenaming, ask }: PanelProps) {
  const m = useMessages();
  const branches = useBranches(repo);
  const [query, setQuery] = useState("");
  const searchRef = useRef<HTMLInputElement>(null);
  // 이름 바꾸기가 끝나거나 그만두면(입력칸이 사라지면) 찾기 칸으로 포커스를 돌린다.
  const wasRenaming = useRef(renaming);
  useEffect(() => {
    if (wasRenaming.current !== null && renaming === null) searchRef.current?.focus();
    wasRenaming.current = renaming;
  }, [renaming]);
  const [expanded, setExpanded] = useState(false);
  const [active, setActive] = useState(0);
  const [pending, setPending] = useState(false);
  const status = useStatus(repo).data;
  const sw = useSwitch(repo, onDone);
  const busy = pending || sw.pending !== null;
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const view = branches.data;
  // 지금 브랜치만 앞서 있으면 "main에 합치기"(D-071). 노선도 옆이 아니라 여기에 둔다(D-048).
  const hint = useMergeHint(repo).data;
  // GitHub 프로젝트면 "main에 합치기"보다 "검토 요청"을 먼저 권한다(D-071, D-113).
  const github = useHostingContext(repo).data?.provider === "github";
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
    if (choice.kind === "switch") {
      const target = { branch: choice.item.name, remote: choice.item.remote };
      // 버전이 없거나 특정 버전을 보는 중이면 core가 맡기지 않으므로 묻지 않는다.
      const files = status?.branch.name != null && status.branch.hasCommits ? status.files.length : 0;
      if (files > 0) {
        close();
        ask(target, files);
      } else if (await sw.run(target)) {
        close();
      }
      return;
    }
    if (createBlocked) return;
    setPending(true);
    try {
      await api.executeAction(repo, { id: "branch-create", input: { name: choice.name } });
      close();
      onDone(m.createdBranch(choice.name));
    } catch (e) {
      showError(e);
    } finally {
      setPending(false);
      invalidateLog(queryClient, repo);
      for (const key of [["status", repo], ["branches", repo]]) {
        void queryClient.invalidateQueries({ queryKey: key });
      }
    }
  };

  // 이름 바꾸기·지우기(D-103). 확인 창 없이 하고 토스트의 되돌리기로 되돌린다.
  const edit = async (input: ActionInput, done: string) => {
    setPending(true);
    try {
      await api.executeAction(repo, input);
      setRenaming(null);
      onDone(done);
      // 지운 줄의 `…`이 사라지므로 찾기 칸으로 돌아간다.
      searchRef.current?.focus();
    } catch (e) {
      showError(e);
    } finally {
      setPending(false);
      invalidateLog(queryClient, repo);
      for (const key of ["status", "branches", "homeMap", "recovery", "mergeHint"]) {
        void queryClient.invalidateQueries({ queryKey: [key, repo] });
      }
    }
  };

  // 꺼내다 겹친 맡긴 파일을 충돌 해결 화면에서 푼다(D-083). 겹치지 않게 됐으면 바로 꺼낸다.
  const resolveParked = async (branch: string) => {
    setPending(true);
    try {
      const result = await api.executeAction(repo, {
        id: "take-out",
        input: { source: { kind: "parked", branch } },
      });
      close();
      if (result.kind === "tookOut") show(m.parkedResolved);
    } catch (e) {
      showError(e);
    } finally {
      setPending(false);
      for (const key of ["status", "branches", "mergeState", "homeMap"]) {
        void queryClient.invalidateQueries({ queryKey: [key, repo] });
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
      if (choice && !busy) void run(choice);
    }
  };

  return (
    <div className={styles.inner}>
      <input
        ref={searchRef}
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
      {view && (
        <List
          view={view}
          choices={choices}
          query={query}
          active={active}
          onPick={(c) => void run(c)}
          repo={repo}
          renaming={renaming}
          pending={busy}
          onRenameStart={setRenaming}
          onRename={(from, to) => void edit({ id: "branch-rename", input: { from, to } }, m.branchRenamed(to))}
          onDelete={(name) => void edit({ id: "branch-delete", input: { name } }, m.branchDeleted(name))}
          onClose={close}
        />
      )}
      {!query.trim() && view?.local.some((b) => b.isHead && b.parkBlocked) && current !== null && (
        <div className={styles.mergeInto}>
          <Button disabled={pending} onClick={() => void resolveParked(current)}>
            {m.resolveOverlap}
          </Button>
        </div>
      )}
      {!query.trim() && hint?.kind === "into" && hint.branch !== null && (
        <div className={`${styles.mergeInto} ${github ? styles.prFirst : ""}`}>
          {github && <PrSlot repo={repo} onClose={close} />}
          <TermTooltip id="merge">
            <Button
              variant={github ? "text" : "secondary"}
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
  repo,
  renaming,
  pending,
  onRenameStart,
  onRename,
  onDelete,
  onClose,
}: {
  view: BranchListView;
  choices: Choice[];
  query: string;
  active: number;
  onPick: (c: Choice) => void;
  repo: string;
  renaming: string | null;
  pending: boolean;
  onRenameStart: (name: string) => void;
  onRename: (from: string, to: string) => void;
  onDelete: (name: string) => void;
  /** 원격 줄 메뉴가 확인 창을 열기 전에 목록을 닫는다. */
  onClose: () => void;
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
        return `${r.author} · ${formatTime(r.at, now, locale)}`;
    }
  };
  const s = sections(view);
  const searching = query.trim() !== "";
  return (
    // 줄 단추 옆에 `…` 단추와 이름 입력칸이 있어 listbox가 아니라 목록이다. 찾기 칸의 ↑↓가 고른 줄은
    // `aria-current`로 알린다.
    <ul className={styles.list} aria-label={branchLabel}>
      {choices.map((c, i) => {
        const first = s.recent[0];
        const header =
          !searching && i === 0 && s.current
            ? m.currentSection
            : !searching && c.kind === "switch" && first && c.item.name === first.name && c.item.remote === first.remote
              ? m.recentSection
              : null;
        const local = c.kind === "switch" && c.item.remote === null ? c.item : null;
        const remoteRow = c.kind === "switch" && c.item.remote !== null ? c.item : null;
        if (local && renaming === local.name) {
          return (
            <li key={`/${local.name}`}>
              {header && <p className={styles.section}>{header}</p>}
              <RenameField repo={repo} from={local.name} pending={pending} onSubmit={(to) => onRename(local.name, to)} />
            </li>
          );
        }
        return (
          <li
            key={c.kind === "create" ? "+create" : `${c.item.remote ?? ""}/${c.item.name}`}
            className={local || remoteRow ? `${styles.rowWrap} ${i === active ? styles.activeWrap : ""}` : undefined}
          >
            {header && <p className={styles.section}>{header}</p>}
            <button
              type="button"
              aria-current={i === active ? "true" : undefined}
              className={`${styles.row} ${i === active ? styles.active : ""} ${c.kind === "create" ? styles.create : ""}`}
              onClick={() => onPick(c)}
            >
              {c.kind === "create" ? (
                <>
                  <Truncate className={styles.name}>{m.createBranch(c.name)}</Truncate>
                  <span className={styles.right}>Enter</span>
                </>
              ) : (
                <>
                  <span className={styles.dot} style={{ background: branchColor(c.item.name, view.defaultBranch) }} />
                  <Truncate className={styles.name}>{c.item.remote ? `${c.item.remote}/${c.item.name}` : c.item.name}</Truncate>
                  <span className={styles.right}>{right(c.item)}</span>
                </>
              )}
            </button>
            {local && (
              <BranchRowMenu row={local.row} onRename={() => onRenameStart(local.name)} onDelete={() => onDelete(local.name)} />
            )}
            {remoteRow?.remote && (
              <RemoteBranchMenu remote={remoteRow.remote} branch={remoteRow.name} onChosen={onClose} />
            )}
          </li>
        );
      })}
      {!searching &&
        view.orphanParked.map((name) => (
          <li key={`orphan/${name}`} className={styles.orphan}>
            <Truncate className={styles.name}>{name}</Truncate>
            <span className={styles.right}>{m.parkedOnly}</span>
          </li>
        ))}
    </ul>
  );
}

/** 줄 안의 이름 입력칸(D-103). Enter로 바꾸고 Esc는 목록이 받아 그만둔다. 한글 조합 중 Enter는 무시. */
function RenameField({
  repo,
  from,
  pending,
  onSubmit,
}: {
  repo: string;
  from: string;
  pending: boolean;
  onSubmit: (to: string) => void;
}) {
  const m = useMessages();
  const [draft, setDraft] = useState(from);
  const name = draft.trim();
  const debounced = useDebounced(name, 300);
  // 대소문자만 바꾸는 것은 같은 브랜치라 "같은 이름 있음"으로 막지 않는다.
  const check = useBranchNameCheck(repo, debounced.toLowerCase() === from.toLowerCase() ? "" : debounced);
  const problem = check.data && !check.data.ok ? check.data.problem : null;
  const ready = name !== "" && name !== from && debounced === name && !check.isFetching && problem === null && !pending;
  return (
    <>
      <input
        className={styles.renameField}
        value={draft}
        aria-label={m.branchRowMenu(from)}
        autoFocus
        onFocus={(e) => e.currentTarget.select()}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Enter" || e.nativeEvent.isComposing || e.nativeEvent.keyCode === 229) return;
          e.preventDefault();
          if (ready) onSubmit(name);
        }}
      />
      {problem !== null && <p className={styles.renameHint}>{problem === "exists" ? m.sameBranchName : m.badBranchName}</p>}
    </>
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
