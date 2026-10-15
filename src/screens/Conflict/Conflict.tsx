import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { HunkCards, WholeFile } from "../../features/conflict/HunkPanel";
import { useConflictStore } from "../../features/conflict/conflictStore";
import {
  canFinish,
  hunkPick,
  nextUnresolved,
  pickHunk,
  progress,
  step,
  toResolutions,
  type ConflictFile,
  type HunkPick,
} from "../../features/conflict/model";
import { api, type ConflictStateView } from "../../lib/api";
import { useConflictFile } from "../../lib/queries";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { TermTooltip } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./Conflict.module.css";

type Props = {
  repo: string;
  state: ConflictStateView;
  /** 마치면 결과 토스트와 되돌리기를 띄운다(useUndo의 notifyDone). */
  onDone: (text: string) => void;
};

/**
 * 충돌 해결 화면(D-028 1안, D-069, D-086). 한 번에 한 곳만 묻고, 고르면 저장될 모습이 바뀐다.
 * git의 충돌 표시 문자는 조회 결과에 없으므로 화면 어디에도 나오지 않는다.
 */
export function Conflict({ repo, state, onDone }: Props) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const store = useConflictStore();
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const files: ConflictFile[] = state.files.map((f) => ({ path: f.path, kind: f.kind, hunks: f.hunks }));
  const key = `${repo}|${state.theirs?.shortOid ?? ""}`;
  const { open } = store;
  useEffect(() => open(key), [open, key]);

  const picks = store.picks;
  const { total, remaining, byFile } = progress(files, picks);
  const place =
    (store.current && files.some((f) => f.path === store.current?.path) ? store.current : null) ??
    nextUnresolved(files, picks, null) ??
    (files[0] ? { path: files[0].path, hunk: 0 } : null);
  const fileInfo = files.find((f) => f.path === place?.path);
  const file = useConflictFile(repo, place?.path ?? null);
  const done = canFinish(files, picks);

  const refresh = () => {
    for (const k of ["status", "log", "mergeState", "homeMap", "branches", "mergeHint"]) {
      void queryClient.invalidateQueries({ queryKey: [k, repo] });
    }
  };

  const choose = (pick: HunkPick) => {
    if (!place || !fileInfo) return;
    // 보고 있던 곳에 머문다(고른 곳이 다 골라져도 다음 곳으로 저절로 넘어가지 않는다).
    if (!store.current) store.setCurrent(place);
    if (fileInfo.kind === "whole") {
      if (pick.kind === "ours" || pick.kind === "theirs") store.setPicks({ ...picks, [place.path]: { kind: pick.kind } });
      return;
    }
    store.setPicks(pickHunk(picks, fileInfo, place.hunk, pick));
  };
  const move = (delta: 1 | -1) => {
    if (place) store.setCurrent(step(files, place, delta));
  };
  // 모두 고르면 포커스가 "합치기 마치기"로 간다.
  useEffect(() => {
    if (done && files.length > 0) document.getElementById("finish-merge")?.focus();
  }, [done, files.length]);

  const run = async (finish: boolean) => {
    setBusy(true);
    setError(null);
    try {
      if (finish) {
        await api.executeAction(repo, {
          id: "conflict-finish",
          input: { resolutions: toResolutions(files, picks), locale },
        });
        store.clear();
        onDone(unpark ? m.parkedResolved : m.merged);
      } else {
        await api.executeAction(repo, { id: "conflict-abort" });
        store.clear();
        show(m.mergeAborted);
      }
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
      refresh();
    }
  };

  const active = !busy;
  useShortcut("chooseOurs", () => choose({ kind: "ours" }), active);
  useShortcut("chooseTheirs", () => choose({ kind: "theirs" }), active);
  useShortcut("chooseBoth", () => fileInfo?.kind === "text" && choose({ kind: "both" }), active);
  useShortcut("nextHunk", () => move(1), active);
  useShortcut("prevHunk", () => move(-1), active);

  // 꺼내다 겹친 맡긴 파일이면 합칠 쪽은 "맡겨 둔 파일"이고, 마치면 버전 대신 고친 파일로 남는다(D-083).
  const unpark = state.context === "unpark";
  const theirs = state.theirs && unpark ? { ...state.theirs, label: m.parkedSide, summary: "" } : state.theirs;
  const view = file.data;
  const hunk = view?.kind === "text" ? view.hunks[place?.hunk ?? 0] : undefined;

  return (
    <div className={styles.screen}>
      <header className={styles.head}>
        <span className={styles.route}>
          <span className={styles.dotTheirs} aria-hidden="true" />
          <span>{theirs?.label}</span>
          <span aria-hidden="true">→</span>
          <span className={styles.dotOurs} aria-hidden="true" />
          <span>{state.ours?.label}</span>
        </span>
        {files.length > 0 && <span className={styles.progress}>{m.conflictsLeft(total, remaining)}</span>}
        <span className={styles.spacer} />
        <TermTooltip id="merge">
          <Button variant="text" disabled={busy} onClick={() => void run(false)}>
            {unpark ? m.abortUnpark : m.abortMerge}
          </Button>
        </TermTooltip>
        {done && (
          <Button id="finish-merge" variant="primary" disabled={busy} onClick={() => void run(true)}>
            {unpark ? m.finishUnpark : m.finishMerge}
          </Button>
        )}
      </header>
      {error !== null && error !== undefined && (
        <div className={styles.error}>
          <ErrorNotice error={error} />
        </div>
      )}
      {files.length === 0 ? (
        <p className={styles.allChosen}>{m.allChosen}</p>
      ) : (
        <div className={styles.body}>
          <nav className={styles.files} aria-label={m.conflictFiles}>
            <h2 className={styles.filesTitle}>{m.conflictFiles}</h2>
            <ul>
              {files.map((f) => {
                const left = byFile[f.path] ?? 0;
                return (
                  <li key={f.path}>
                    <button
                      type="button"
                      className={`${styles.file} ${place?.path === f.path ? styles.current : ""}`}
                      aria-current={place?.path === f.path}
                      onClick={() => store.setCurrent({ path: f.path, hunk: 0 })}
                    >
                      <span aria-hidden="true">{left === 0 ? "✓" : "●"}</span>
                      <span className={styles.fileName}>{f.path.split("/").pop()}</span>
                      {left > 0 && f.kind === "text" && <span className={styles.left}>{left}</span>}
                    </button>
                  </li>
                );
              })}
            </ul>
            {state.cleanFiles > 0 && <p className={styles.clean}>{m.cleanMerged(state.cleanFiles)}</p>}
          </nav>
          <main className={styles.main}>
            {place && (
              <p className={styles.where}>
                <span className={styles.path}>{place.path}</span>
                {hunk && <> · {m.atLine(hunk.line)}</>}
              </p>
            )}
            <p className={styles.question}>{view?.kind === "whole" ? m.wholeQuestion : m.conflictQuestion}</p>
            {file.error && <ErrorNotice error={file.error} />}
            {view && hunk && place && (
              <HunkCards
                key={`${place.path}#${place.hunk}`}
                hunk={hunk}
                pick={hunkPick(picks, place.path, place.hunk)}
                ours={state.ours}
                theirs={theirs}
                onPick={choose}
              />
            )}
            {view && view.kind === "whole" && place && (
              <WholeFile
                file={view}
                pick={picks[place.path]}
                ours={state.ours}
                theirs={theirs}
                onPick={(side) => choose({ kind: side })}
              />
            )}
            <div className={styles.nav}>
              <Button variant="text" onClick={() => move(-1)}>
                ← {m.prevPlace}
              </Button>
              <Button variant="secondary" onClick={() => move(1)}>
                {m.nextPlace} →
              </Button>
            </div>
          </main>
        </div>
      )}
    </div>
  );
}
