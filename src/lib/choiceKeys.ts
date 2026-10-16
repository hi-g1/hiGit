// 고르는 부품(Choice, radiogroup) 안의 방향키. 단축키가 아니라 부품 조작이라 표 밖에 두되 판정은 여기에만(D-056).

export type ChoiceMove = "next" | "prev" | "first" | "last";

type KeyLike = {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  isComposing?: boolean;
};

const MOVES: Record<string, ChoiceMove> = {
  ArrowRight: "next",
  ArrowDown: "next",
  ArrowLeft: "prev",
  ArrowUp: "prev",
  Home: "first",
  End: "last",
};

/** 옮기는 키면 그 방향, 아니면 null. 한글 조합 중이거나 수정키가 눌렸으면 null. */
export function choiceKey(e: KeyLike): ChoiceMove | null {
  if (e.isComposing || e.shiftKey || e.ctrlKey || e.altKey || e.metaKey) return null;
  return MOVES[e.key] ?? null;
}

/** `count`개 중 `at`에서 `move`로 간 자리(끝에서는 돌아간다). */
export function choiceIndex(move: ChoiceMove, at: number, count: number): number {
  switch (move) {
    case "next":
      return (at + 1) % count;
    case "prev":
      return (at - 1 + count) % count;
    case "first":
      return 0;
    case "last":
      return count - 1;
  }
}
