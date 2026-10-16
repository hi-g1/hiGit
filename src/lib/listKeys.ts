// 목록 안 조작 키(↑↓, PageUp·PageDown, Home·End, 열기). 단축키가 아니라 부품 조작이라 단축키 표 밖에
// 두되(반복 1b 점검 결정), 키 판정은 src/lib에만 둔다(D-056).

export type ListMove = "next" | "prev" | "pageNext" | "pagePrev" | "first" | "last" | "open" | "extendNext" | "extendPrev";

type KeyLike = {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  isComposing?: boolean;
};

const MOVES: Record<string, ListMove> = {
  ArrowDown: "next",
  ArrowUp: "prev",
  PageDown: "pageNext",
  PageUp: "pagePrev",
  Home: "first",
  End: "last",
  Enter: "open",
  " ": "open",
  ContextMenu: "open",
};

const EXTEND: Record<string, ListMove> = {
  F10: "open",
  ArrowDown: "extendNext",
  ArrowUp: "extendPrev",
};

/**
 * 목록 조작이면 그 동작, 아니면 null. 한글 조합 중이거나 Ctrl·Alt·Meta가 눌렸으면 null. Shift+F10은 열기,
 * Shift+↑↓는 고른 구간 넓히기·줄이기(여러 역 선택, D-106).
 */
export function listKey(e: KeyLike): ListMove | null {
  if (e.isComposing || e.ctrlKey || e.altKey || e.metaKey) return null;
  if (e.shiftKey) return EXTEND[e.key] ?? null;
  return MOVES[e.key] ?? null;
}
