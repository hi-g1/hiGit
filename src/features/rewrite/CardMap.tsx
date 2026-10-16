import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { cardKey } from "../../lib/cardKeys";
import { useMessages } from "../../messages";
import { Card, type CardAction } from "./Card";
import { canJoin, groups, joinWithBelow, move, setMessage, split, status, toggleDrop, type Card as CardModel } from "./model";
import styles from "./CardMap.module.css";

type Props = {
  cards: readonly CardModel[];
  original: readonly CardModel[];
  onChange: (cards: CardModel[]) => void;
  readOnly: boolean;
};

/**
 * 기록 정리 카드 지도(D-106). 카드 목록은 `role="list"`이고 한 카드만 Tab으로 닿는다(roving tabindex).
 * 카드에서 Alt+↑↓는 옮기기, ↑↓는 고르기, Delete는 빼기, Enter는 메시지 고치기. 옮긴 결과는 소리로 알린다.
 */
export function CardMap({ cards, original, onChange, readOnly }: Props) {
  const m = useMessages();
  const [active, setActive] = useState(0);
  const [editing, setEditing] = useState<string | null>(null);
  const [announce, setAnnounce] = useState("");
  const refs = useRef<(HTMLLIElement | null)[]>([]);
  const focusWanted = useRef(false);
  const at = Math.min(active, Math.max(0, cards.length - 1));
  const gs = groups(cards);

  useEffect(() => {
    if (!focusWanted.current) return;
    focusWanted.current = false;
    refs.current[at]?.focus();
  }, [at, cards]);

  const focusCard = (index: number) => {
    focusWanted.current = true;
    setActive(Math.max(0, Math.min(cards.length - 1, index)));
  };

  const act = (index: number, action: CardAction) => {
    if (readOnly) return;
    const card = cards[index];
    if (!card) return;
    switch (action) {
      case "up":
      case "down": {
        const next = move(cards, index, action === "up" ? -1 : 1);
        onChange(next);
        const to = next.findIndex((c) => c.oid === card.oid);
        setAnnounce(m.cardMovedTo(card.summary, to + 1));
        focusCard(to);
        return;
      }
      case "join":
        return onChange(joinWithBelow(cards, index));
      case "split":
        return onChange(split(cards, index));
      case "drop":
        if (editing === card.oid) setEditing(null);
        return onChange(toggleDrop(cards, index));
      case "edit":
        return setEditing(card.oid);
    }
  };

  const onKeyDown = (index: number) => (e: KeyboardEvent<HTMLLIElement>) => {
    // 카드 안의 단추·글상자 키는 그 부품이 처리한다.
    if (e.target !== e.currentTarget) return;
    const key = cardKey({ ...e, isComposing: e.nativeEvent.isComposing, keyCode: e.nativeEvent.keyCode });
    if (key === null) return;
    e.preventDefault();
    if (key === "prev") return focusCard(index - 1);
    if (key === "next") return focusCard(index + 1);
    const g = gs.find(([a, b]) => index >= a && index <= b);
    if (key === "moveUp") return act(index, "up");
    if (key === "moveDown") return act(index, "down");
    if (key === "toggleDrop") return act(index, "drop");
    // 메시지는 묶음의 머리(맨 아래 카드)에서 고친다.
    if (key === "edit" && g && cards[g[1]]?.state === "keep") {
      setEditing(cards[g[1]]!.oid);
    }
  };

  return (
    <>
      <ol className={styles.list} aria-label={m.cardList} role="list">
        {cards.map((card, i) => {
          const g = gs.findIndex(([a, b]) => i >= a && i <= b);
          const [start, end] = gs[g]!;
          return (
            <Card
              key={card.oid}
              ref={(el) => {
                refs.current[i] = el;
              }}
              card={card}
              status={status(cards, original, i)}
              position={i + 1}
              total={cards.length}
              groupSize={i === end ? end - start + 1 : 0}
              canUp={g > 0}
              canDown={g < gs.length - 1}
              canJoin={canJoin(cards, i)}
              readOnly={readOnly}
              active={i === at}
              editing={editing === card.oid && !readOnly}
              onFocus={() => setActive(i)}
              onKeyDown={onKeyDown(i)}
              onAction={(action) => act(i, action)}
              onMessage={(text) => onChange(setMessage(cards, i, text))}
              onCloseEdit={() => {
                setEditing(null);
                focusCard(i);
              }}
            />
          );
        })}
      </ol>
      <p className={styles.live} aria-live="polite">
        {announce}
      </p>
    </>
  );
}
