import { useState, type ReactNode } from "react";
import type { ConflictFileView, ConflictHunkView, ConflictSide, SideState } from "../../lib/api";
import { formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { Term, TermTooltip } from "../../ui/Term";
import { customStart, savedPreview, type FilePick, type HunkPick } from "./model";
import styles from "./HunkPanel.module.css";

type Side = "ours" | "theirs";

/** 카드 머리의 쪽 이름. 없으면 용어(`ours`·`theirs`). 복사·되돌리기·기록 정리는 맥락의 이름을 쓴다. */
export type SideNames = { ours: ReactNode; theirs: ReactNode };

/** 카드 머리: "내 브랜치 · 사람 · 날짜 · 이름". */
function SideHeader({ side, info, name }: { side: Side; info: ConflictSide | null; name?: ReactNode }) {
  const locale = useUiStore((s) => s.locale);
  // 날짜 표시의 기준 시각. 다시 그려도 글자가 바뀌지 않게 처음 한 번만 잰다.
  const [now] = useState(() => Date.now());
  return (
    <span className={styles.cardHead}>
      <span className={side === "ours" ? styles.dotOurs : styles.dotTheirs} aria-hidden="true" />
      {name ?? <Term id={side} />}
      {info && (
        <span className={styles.meta}>
          {[info.label, info.author, formatDate(info.at, now, locale), info.summary].filter(Boolean).join(" · ")}
        </span>
      )}
    </span>
  );
}

function Lines({ lines, side }: { lines: string[]; side: Side | "base" }) {
  const cls = side === "ours" ? styles.codeOurs : side === "theirs" ? styles.codeTheirs : styles.codeBase;
  return (
    <pre className={`${styles.code} ${cls}`}>
      {lines.length === 0 ? "\u00a0" : lines.join("\n")}
    </pre>
  );
}

type HunkProps = {
  hunk: ConflictHunkView;
  pick: HunkPick | null;
  ours: ConflictSide | null;
  theirs: ConflictSide | null;
  names?: SideNames;
  onPick: (pick: HunkPick) => void;
};

/** 텍스트 파일의 곳 하나: 카드 둘, 둘 다·직접 고치기, 원래 코드, 저장될 모습(D-028 1안). */
export function HunkCards({ hunk, pick, ours, theirs, names, onPick }: HunkProps) {
  const m = useMessages();
  const [editing, setEditing] = useState(pick?.kind === "custom");
  const [draft, setDraft] = useState(() => customStart(hunk, pick));
  const chosen = (side: Side) => pick?.kind === side;
  const card = (side: Side, lines: string[], info: ConflictSide | null) => (
    <section className={`${styles.card} ${side === "ours" ? styles.cardOurs : styles.cardTheirs} ${pick && !chosen(side) ? styles.dim : ""}`}>
      <div className={styles.cardTop}>
        <SideHeader side={side} info={info} name={names?.[side]} />
        <Button variant={chosen(side) ? "primary" : "secondary"} onClick={() => onPick({ kind: side })} aria-pressed={chosen(side)}>
          {chosen(side) ? `✓ ${m.useThis}` : m.useThis}
        </Button>
      </div>
      <Lines lines={lines} side={side} />
    </section>
  );
  return (
    <div className={styles.panel}>
      {card("ours", hunk.ours, ours)}
      {card("theirs", hunk.theirs, theirs)}
      <div className={styles.row}>
        <Button variant={pick?.kind === "both" ? "primary" : "secondary"} aria-pressed={pick?.kind === "both"} onClick={() => onPick({ kind: "both" })}>
          {m.useBoth}
        </Button>
        <Button
          variant={pick?.kind === "custom" ? "primary" : "secondary"}
          aria-pressed={pick?.kind === "custom"}
          onClick={() => {
            setEditing(true);
            setDraft(customStart(hunk, pick));
          }}
        >
          {m.editByHand}
        </Button>
      </div>
      {editing && (
        <textarea
          className={styles.editor}
          aria-label={m.editByHand}
          value={draft}
          rows={Math.max(3, draft.split("\n").length)}
          onChange={(e) => {
            setDraft(e.target.value);
            onPick({ kind: "custom", text: e.target.value });
          }}
        />
      )}
      <details className={styles.base}>
        <summary>
          <TermTooltip id="base">
            <span>{m.showBase}</span>
          </TermTooltip>
        </summary>
        <Lines lines={hunk.base} side="base" />
      </details>
      <Preview hunk={hunk} pick={pick} />
    </div>
  );
}

function Preview({ hunk, pick }: { hunk: ConflictHunkView; pick: HunkPick | null }) {
  const m = useMessages();
  const lines = savedPreview(hunk, pick);
  return (
    <section className={styles.preview} aria-label={m.savedLooks}>
      <h3 className={styles.previewTitle}>{m.savedLooks}</h3>
      <pre className={styles.code}>
        {lines.map((l, i) => (
          <span key={i} className={styles[`line_${l.from}`]}>
            {l.text === "" ? "\u00a0" : l.text}
          </span>
        ))}
      </pre>
    </section>
  );
}

function sideText(state: SideState, m: ReturnType<typeof useMessages>): string | null {
  switch (state.kind) {
    case "deleted":
      return m.sideDeleted;
    case "binary":
      return m.sideBinary;
    case "large":
      return m.sideLarge;
    case "submodule":
      return m.sideSubmodule;
    case "link":
      return m.sideLink(state.target);
    default:
      return null;
  }
}

type WholeProps = {
  file: ConflictFileView;
  pick: FilePick | undefined;
  ours: ConflictSide | null;
  theirs: ConflictSide | null;
  names?: SideNames;
  onPick: (side: Side) => void;
};

/** 파일 단위로 고르는 파일(이진·삭제·링크 등, D-086). */
export function WholeFile({ file, pick, ours, theirs, names, onPick }: WholeProps) {
  const m = useMessages();
  const card = (side: Side, state: SideState, preview: string[], info: ConflictSide | null) => {
    const chosen = pick?.kind === side;
    const text = sideText(state, m);
    return (
      <section className={`${styles.card} ${side === "ours" ? styles.cardOurs : styles.cardTheirs} ${pick && !chosen ? styles.dim : ""}`}>
        <div className={styles.cardTop}>
          <SideHeader side={side} info={info} name={names?.[side]} />
          <Button variant={chosen ? "primary" : "secondary"} aria-pressed={chosen} onClick={() => onPick(side)}>
            {chosen ? `✓ ${m.useThis}` : m.useThis}
          </Button>
        </div>
        {text !== null ? <p className={styles.sideState}>{text}</p> : <Lines lines={preview} side={side} />}
      </section>
    );
  };
  return (
    <div className={styles.panel}>
      {card("ours", file.oursSide, file.oursPreview, ours)}
      {card("theirs", file.theirsSide, file.theirsPreview, theirs)}
    </div>
  );
}
