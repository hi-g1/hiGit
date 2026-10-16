// 기록 정리 카드 지도의 계산(계획 008 E28, D-106). DOM 없음. 화면 순서는 새 버전이 위(0번).
//
// 묶음: 맨 아래 카드(머리)와 그 위로 `joinBelow`가 이어진 카드들. 묶음은 한 덩어리로 움직인다. 묶인
// 카드 하나만 옮기려면 먼저 묶기를 푼다. 빠진 카드는 묶을 수 없고, 묶인 카드를 빼면 그 카드만 묶음에서
// 빠진다. 머리 카드를 빼면 그 위 카드들이 아래 카드로 이어 붙지 않고 위 카드가 새 머리가 된다.
import type { RewritePlanView, RewriteStepInput } from "../../lib/api";

export type Card = {
  oid: string;
  shortOid: string;
  summary: string;
  author: string;
  /** unix 초 */
  time: number;
  /** 처음 메시지(전체). */
  message: string;
  /** 고친 메시지. 묶음의 머리에만 둔다. */
  newMessage: string | null;
  state: "keep" | "drop";
  /** 바로 아래 카드에 합친다. */
  joinBelow: boolean;
};

export type CardStatus = "moved" | "joined" | "dropped" | null;

export function fromPlan(plan: RewritePlanView): Card[] {
  return plan.cards.map((c) => ({
    oid: c.oid,
    shortOid: c.shortOid,
    summary: c.summary,
    author: c.author,
    time: c.time,
    message: c.message,
    newMessage: null,
    state: "keep",
    joinBelow: false,
  }));
}

/** 묶음들(위에서 아래로). 각 묶음은 [처음 번호, 끝 번호]이고 끝이 머리. */
export function groups(cards: readonly Card[]): [number, number][] {
  const out: [number, number][] = [];
  let start = 0;
  cards.forEach((card, i) => {
    if (!card.joinBelow || i === cards.length - 1) {
      out.push([start, i]);
      start = i + 1;
    }
  });
  return out;
}

/** 그 카드가 든 묶음의 머리 번호. */
export function headOf(cards: readonly Card[], index: number): number {
  return groups(cards).find(([a, b]) => index >= a && index <= b)?.[1] ?? index;
}

/** 묶음째 한 칸(옆 묶음 하나) 옮긴다. -1은 위(새 쪽). 끝이면 그대로. */
export function move(cards: readonly Card[], index: number, delta: -1 | 1): Card[] {
  const gs = groups(cards);
  const g = gs.findIndex(([a, b]) => index >= a && index <= b);
  const other = g + delta;
  if (g < 0 || other < 0 || other >= gs.length) return [...cards];
  const [upper, lower] = delta < 0 ? [gs[other]!, gs[g]!] : [gs[g]!, gs[other]!];
  const block = (r: [number, number]) => cards.slice(r[0], r[1] + 1);
  return [...cards.slice(0, upper[0]), ...block(lower), ...block(upper), ...cards.slice(lower[1] + 1)];
}

/** 바로 아래 카드에 묶을 수 있는지(둘 다 넣은 상태이고 아직 이어지지 않음). */
export function canJoin(cards: readonly Card[], index: number): boolean {
  const card = cards[index];
  const below = cards[index + 1];
  return card !== undefined && below !== undefined && !card.joinBelow && card.state === "keep" && below.state === "keep";
}

export function joinWithBelow(cards: readonly Card[], index: number): Card[] {
  if (!canJoin(cards, index)) return [...cards];
  // 묶이면 머리가 아니므로 고친 메시지는 새 머리(아래 묶음의 머리)의 것만 남는다.
  return cards.map((c, i) => (i === index ? { ...c, joinBelow: true, newMessage: null } : c));
}

/** 묶기 풀기: 이 카드를 아래와 떼어 낸다(이 카드가 위 카드들의 새 머리). */
export function split(cards: readonly Card[], index: number): Card[] {
  return cards.map((c, i) => (i === index ? { ...c, joinBelow: false } : c));
}

export function toggleDrop(cards: readonly Card[], index: number): Card[] {
  const card = cards[index];
  if (!card) return [...cards];
  if (card.state === "drop") return cards.map((c, i) => (i === index ? { ...c, state: "keep" } : c));
  return cards.map((c, i) => {
    if (i === index) return { ...c, state: "drop", joinBelow: false, newMessage: null };
    // 빠진 카드에 이어 붙던 위 카드는 새 머리가 된다.
    if (i === index - 1 && c.joinBelow) return { ...c, joinBelow: false };
    return c;
  });
}

/** 묶음의 머리(넣은 카드)에만 메시지를 고친다. 처음 메시지와 같으면 고치지 않은 것으로. */
export function setMessage(cards: readonly Card[], index: number, text: string): Card[] {
  const card = cards[index];
  if (!card || card.joinBelow || card.state === "drop") return [...cards];
  const same = text.trimEnd() === card.message.trimEnd() || text.trim() === "";
  return cards.map((c, i) => (i === index ? { ...c, newMessage: same ? null : text } : c));
}

/** 정리 후 버전 수, 빠지는 수, 합쳐지는 수. */
export function summary(cards: readonly Card[]): { after: number; dropped: number; joined: number } {
  const dropped = cards.filter((c) => c.state === "drop").length;
  const joined = cards.filter((c) => c.state === "keep" && c.joinBelow).length;
  return { after: cards.length - dropped - joined, dropped, joined };
}

export function changed(cards: readonly Card[], original: readonly Card[]): boolean {
  return (
    cards.some((c, i) => c.oid !== original[i]?.oid) ||
    cards.some((c) => c.state === "drop" || c.joinBelow || c.newMessage !== null)
  );
}

/** core 입력(적용 순서 = 아래부터). 바뀐 메시지만 담는다. */
export function toInput(cards: readonly Card[]): RewriteStepInput[] {
  return [...cards].reverse().map((c) => ({
    oid: c.oid,
    kind: c.state === "drop" ? "drop" : c.joinBelow ? "squash" : "pick",
    message: c.state === "keep" && !c.joinBelow ? c.newMessage : null,
  }));
}

/** 카드 상태 글자. 빠짐 > 묶임 > 옮김. */
export function status(cards: readonly Card[], original: readonly Card[], index: number): CardStatus {
  const card = cards[index];
  if (!card) return null;
  if (card.state === "drop") return "dropped";
  if (card.joinBelow) return "joined";
  return original[index]?.oid === card.oid ? null : "moved";
}
