import { useQueryClient } from "@tanstack/react-query";
import { useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { api, type ActionInput } from "../../lib/api";
import { listKey } from "../../lib/listKeys";
import { invalidateLog, useActions, useBranches, useRecentProjects, useStatus } from "../../lib/queries";
import { SHORTCUTS, matches } from "../../lib/shortcuts";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { useUiStore } from "../../state/uiStore";
import { DialogFrame } from "../../ui/DialogFrame";
import { Kbd } from "../../ui/Kbd";
import { useToastStore } from "../../ui/toastStore";
import { useBranchListStore } from "../branches/branchListStore";
import { search as branchSearch } from "../branches/model";
import { useSwitch } from "../branches/useSwitch";
import { useCloneStore } from "../clone/cloneStore";
import { useForcePushStore } from "../force-push/forcePushStore";
import { useOpenFolder } from "../open-project/useOpenFolder";
import { useOpenProject } from "../open-project/useOpenProject";
import { usePrDialogStore } from "../pr/prDialogStore";
import { usePublishStore } from "../publish/publishStore";
import { useSettingsStore } from "../settings/settingsStore";
import { useShortcutsStore } from "../shortcuts/shortcutsStore";
import { useSync } from "../sync/useSync";
import { buildCommands, searchCommands, type Command, type OpenTarget } from "./commands";
import styles from "./Palette.module.css";
import { usePaletteStore } from "./paletteStore";

type Props = {
  /** 열린 저장소. 시작 화면이면 null(폴더 열기·복제·최근 프로젝트·설정만). */
  repo: string | null;
  merging?: boolean;
  /** 바로 한 일의 결과와 되돌리기(useUndo의 notifyDone). */
  onDone: (text: string) => void;
};

/** Ctrl+K 명령 팔레트(D-170, D-199). 열린 동안 전역 단축키는 쉬므로 Ctrl+K(닫기)는 여기서 받는다. */
export function Palette({ repo, merging = false, onDone }: Props) {
  const { open, setOpen } = usePaletteStore();
  useShortcut("palette", () => setOpen(true));
  if (!open) return null;
  return <PaletteDialog repo={repo} merging={merging} onDone={onDone} close={() => setOpen(false)} />;
}

function PaletteDialog({ repo, merging, onDone, close }: Required<Props> & { close: () => void }) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const listId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const actions = useActions().data;
  const status = useStatus(repo ?? "", repo !== null).data;
  const branchView = useBranches(repo ?? "", repo !== null).data;
  const recent = useRecentProjects().data;

  const all = useMemo(
    () =>
      buildCommands({
        actions: actions ?? [],
        locale,
        m,
        hasRepo: repo !== null,
        changedFiles: status?.files.length ?? 0,
        hasRemote: (branchView?.remotes.length ?? 0) > 0,
        hasUpstream: status?.branch.upstream != null,
        merging,
        current: status?.branch.name ?? null,
        branches: branchView ? branchSearch(branchView, "").map((b) => ({ name: b.name, remote: b.remote })) : [],
        recent: recent ?? [],
        repoPath: repo,
      }),
    [actions, locale, m, repo, status, branchView, merging, recent],
  );
  const shown = useMemo(() => searchCommands(all, query), [all, query]);
  const run = useRun(repo, onDone, status?.branch.name != null && status.branch.hasCommits ? status.files.length : 0);

  const choose = (cmd: Command | undefined) => {
    if (!cmd || cmd.disabledReason) return;
    close();
    run(cmd);
  };
  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (matches("palette", e.nativeEvent)) {
      e.preventDefault();
      close();
      return;
    }
    // 한글 조합 중 Enter는 글자를 확정하는 키라서 실행하지 않는다(WebKitGTK는 keyCode 229).
    if (e.nativeEvent.isComposing || e.nativeEvent.keyCode === 229) return;
    const move = listKey(e.nativeEvent);
    const last = shown.length - 1;
    const page = 8;
    const to: Record<string, number> = {
      next: Math.min(active + 1, last),
      prev: Math.max(active - 1, 0),
      pageNext: Math.min(active + page, last),
      pagePrev: Math.max(active - page, 0),
    };
    if (move === "open" && e.key === "Enter") {
      e.preventDefault();
      choose(shown[active]);
    } else if (move && move in to) {
      e.preventDefault();
      setActive(Math.max(to[move]!, 0));
    }
  };

  const optionId = (i: number) => `${listId}-${i}`;
  return (
    <DialogFrame title={m.shortcutName.palette} onClose={close} size="wide" className={styles.palette} initialFocus={inputRef}>
      <input
        ref={inputRef}
        className={styles.search}
        role="combobox"
        aria-expanded="true"
        aria-controls={listId}
        aria-activedescendant={shown.length > 0 ? optionId(active) : undefined}
        aria-label={m.paletteSearch}
        placeholder={m.paletteSearch}
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setActive(0);
        }}
        onKeyDown={onKeyDown}
      />
      {shown.length === 0 ? (
        <p className={styles.empty}>{m.paletteEmpty}</p>
      ) : (
        <ul id={listId} role="listbox" className={styles.list} aria-label={m.shortcutName.palette}>
          {shown.map((cmd, i) => (
            <li
              key={cmd.key}
              id={optionId(i)}
              role="option"
              aria-selected={i === active}
              aria-disabled={cmd.disabledReason ? true : undefined}
              className={`${styles.option} ${i === active ? styles.active : ""} ${cmd.disabledReason ? styles.disabled : ""}`}
              onMouseMove={() => setActive(i)}
              onClick={() => choose(cmd)}
            >
              <span className={styles.label}>{cmd.label}</span>
              <span className={styles.detail}>{cmd.disabledReason ?? cmd.detail}</span>
              {cmd.shortcut && <Kbd>{SHORTCUTS[cmd.shortcut].label}</Kbd>}
            </li>
          ))}
        </ul>
      )}
    </DialogFrame>
  );
}

/** 고른 항목을 한다. 바로 하기는 액션, 열기는 그 화면, 브랜치는 U12의 갈아타기 흐름, 최근 프로젝트는 열기. */
function useRun(repo: string | null, onDone: (text: string) => void, dirtyFiles: number) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const setView = useRepoStore((s) => s.setView);
  const sync = useSync(repo ?? "");
  const sw = useSwitch(repo ?? "", onDone);
  const branchList = useBranchListStore();
  const openFolder = useOpenFolder();
  const openProject = useOpenProject();
  const openers: Record<OpenTarget, () => void> = {
    commitBox: () => {
      setView("home");
      // 홈이 그려진 뒤 이름 칸으로.
      setTimeout(() => document.getElementById("version-name")?.focus(), 0);
    },
    branchList: () => branchList.setOpen(true),
    clone: useCloneStore((s) => s.openClone),
    publish: usePublishStore((s) => s.openDialog),
    prCreate: usePrDialogStore((s) => s.openPrDialog),
    settings: () => useSettingsStore.getState().openSettings(),
    forcePush: useForcePushStore((s) => s.openDialog),
    openFolder: () => void openFolder.pick(),
    shortcuts: () => useShortcutsStore.getState().setOpen(true),
    history: () => setView("history"),
    activity: () => setView("activity"),
    none: () => undefined,
  };

  const execute = async (input: ActionInput, label: string) => {
    if (!repo) return;
    try {
      const result = await api.executeAction(repo, input);
      if (result.kind === "stashed") onDone(m.stashed);
      else show(m.paletteDone(label));
    } catch (e) {
      showError(e);
    } finally {
      invalidateLog(queryClient, repo);
      for (const key of ["status", "branches", "homeMap", "mergeHint", "stashes", "mergeState"]) {
        void queryClient.invalidateQueries({ queryKey: [key, repo] });
      }
    }
  };

  return (cmd: Command) => {
    switch (cmd.kind) {
      case "run":
        // 동기화는 겹침·멈춤 안내까지 하는 기존 흐름으로.
        if (cmd.input.id === "sync") sync.run();
        else void execute(cmd.input, cmd.label);
        return;
      case "open":
        openers[cmd.open]();
        return;
      case "switch": {
        const target = { branch: cmd.branch, remote: cmd.remote };
        if (dirtyFiles > 0) branchList.setAsking({ target, files: dirtyFiles });
        else void sw.run(target);
        return;
      }
      case "recent":
        void openProject.open(cmd.path, "recent").catch((e: unknown) => showError(e));
        return;
    }
  };
}
