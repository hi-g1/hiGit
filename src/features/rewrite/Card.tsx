import { forwardRef, useState, type KeyboardEvent } from "react";
import { matches } from "../../lib/shortcuts";
import { formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import type { Card as CardModel, CardStatus } from "./model";
import styles from "./CardMap.module.css";

export type CardAction = "up" | "down" | "join" | "split" | "drop" | "edit";

type Props = {
  card: CardModel;
  status: CardStatus;
  position: number;
  total: number;
  /** 묶음의 머리이면 그 묶음의 카드 수(혼자면 1). 머리가 아니면 0. */
  groupSize: number;
  canUp: boolean;
  canDown: boolean;
  canJoin: boolean;
  readOnly: boolean;
  active: boolean;
  editing: boolean;
  onFocus: () => void;
  onKeyDown: (e: KeyboardEvent<HTMLLIElement>) => void;
  onAction: (action: CardAction) => void;
  onMessage: (text: string) => void;
  onCloseEdit: () => void;
};

/** 카드 안 메시지 글상자. 쓰는 동안의 글은 여기 두고(비워도 처음 글로 돌아가지 않게) 바뀔 때마다 알린다. */
function Editor({ initial, onMessage, onClose }: { initial: string; onMessage: (text: string) => void; onClose: () => void }) {
  const m = useMessages();
  const [draft, setDraft] = useState(initial);
  return (
    <textarea
      className={styles.editor}
      aria-label={m.cardEdit}
      autoFocus
      value={draft}
      onChange={(e) => {
        setDraft(e.target.value);
        onMessage(e.target.value);
      }}
      onKeyDown={(e) => {
        // 글상자 안의 키는 카드·화면으로 가지 않는다. Esc·Ctrl+Enter는 닫기(한글 조합 중 제외).
        e.stopPropagation();
        const composing = e.nativeEvent.isComposing || e.nativeEvent.keyCode === 229;
        if ((e.key === "Escape" && !composing) || matches("commit", e.nativeEvent)) {
          e.preventDefault();
          onClose();
        }
      }}
    />
  );
}

/** 카드 하나: 짧은 해시·이름·작성자·날짜, 상태 글자, 글자 버튼 줄. 메시지는 카드 안 글상자에서 고친다. */
export const Card = forwardRef<HTMLLIElement, Props>(function Card(props, ref) {
  const { card, status, position, total, groupSize, canUp, canDown, canJoin, readOnly, active, editing } = props;
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const [now] = useState(() => Date.now());
  const statusText = status === null ? null : m.cardStatus[status];
  const head = groupSize > 0;
  const name = card.newMessage?.split("\n")[0] ?? card.summary;
  const cls = [styles.card, active ? styles.active : "", card.joinBelow ? styles.joined : "", card.state === "drop" ? styles.dropped : ""].join(" ");
  const button = (action: CardAction, label: string, enabled: boolean) => (
    <button type="button" className={styles.button} tabIndex={active ? 0 : -1} disabled={readOnly || !enabled} onClick={() => props.onAction(action)}>
      {label}
    </button>
  );
  return (
    <li
      ref={ref}
      className={cls}
      tabIndex={active ? 0 : -1}
      aria-label={m.cardLabel(name, position, total, statusText)}
      onFocus={(e) => {
        if (e.target === e.currentTarget) props.onFocus();
      }}
      onKeyDown={props.onKeyDown}
    >
      <div className={styles.top}>
        <span className={styles.oid}>{card.shortOid}</span>
        <bdi className={styles.name}>{name}</bdi>
        {head && groupSize > 1 && <span className={styles.group}>{m.cardGroup(groupSize)}</span>}
        {statusText && <span className={styles.status}>{statusText}</span>}
      </div>
      <span className={styles.meta}>
        <bdi>{card.author}</bdi> · {formatDate(card.time, now, locale)}
      </span>
      {editing ? (
        <Editor initial={card.newMessage ?? card.message.trimEnd()} onMessage={props.onMessage} onClose={props.onCloseEdit} />
      ) : (
        <div className={styles.buttons}>
          {button("up", m.cardUp, canUp)}
          {button("down", m.cardDown, canDown)}
          {card.joinBelow ? button("split", m.cardSplit, true) : button("join", m.cardJoin, canJoin)}
          {button("drop", card.state === "drop" ? m.cardKeep : m.cardDrop, true)}
          {head && card.state === "keep" && button("edit", m.cardEdit, true)}
        </div>
      )}
    </li>
  );
});
