// 기록 정리 카드 지도 안의 조작 키(D-056, 계획 008 C10). 단축키 표 밖의 부품 조작이지만 키 판정은
// src/lib에만 둔다(listKeys와 같은 규칙).

export type CardMove = "moveUp" | "moveDown" | "toggleDrop" | "edit" | "prev" | "next";

type KeyLike = {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  isComposing?: boolean;
  keyCode?: number;
};

/** 카드 조작이면 그 동작, 아니면 null. 한글 조합 중이거나 Ctrl·Meta·Shift가 눌렸으면 null. */
export function cardKey(e: KeyLike): CardMove | null {
  if (e.isComposing || e.keyCode === 229 || e.ctrlKey || e.metaKey || e.shiftKey) return null;
  if (e.altKey) return e.key === "ArrowUp" ? "moveUp" : e.key === "ArrowDown" ? "moveDown" : null;
  switch (e.key) {
    case "ArrowUp":
      return "prev";
    case "ArrowDown":
      return "next";
    case "Delete":
    case "Backspace":
      return "toggleDrop";
    case "Enter":
      return "edit";
    default:
      return null;
  }
}
