import { useEffect, useState, type PointerEvent } from "react";

export type Drag = { from: number; to: number };

/**
 * 카드 끌기(D-106): 손잡이를 포인터로 잡아(`setPointerCapture`) 위아래로 끌면 놓을 자리(`to`)가 바뀌고,
 * 놓으면 `onDrop(from, to)`. 취소(pointercancel)나 Esc면 아무것도 바꾸지 않는다. 끌기는 순서만 바꾼다.
 * `rectOf(i)`는 i번째 카드의 화면 위치.
 */
export function useDrag(count: number, rectOf: (i: number) => DOMRect | null, onDrop: (from: number, to: number) => void, disabled: boolean) {
  const [drag, setDrag] = useState<Drag | null>(null);

  // 끄는 동안 Esc는 끌기만 취소한다(화면의 Esc보다 먼저 받는다).
  useEffect(() => {
    if (!drag) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      e.stopPropagation();
      setDrag(null);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [drag]);

  /** 포인터 높이에 가장 가까운 카드(가운데보다 위면 그 카드). */
  const indexAt = (y: number): number => {
    for (let i = 0; i < count; i++) {
      const r = rectOf(i);
      if (r && y < r.top + r.height / 2) return i;
    }
    return count - 1;
  };

  const handlers = (index: number) => ({
    onPointerDown: (e: PointerEvent<HTMLElement>) => {
      if (disabled || e.button !== 0) return;
      e.preventDefault();
      e.currentTarget.setPointerCapture?.(e.pointerId);
      setDrag({ from: index, to: index });
    },
    onPointerMove: (e: PointerEvent<HTMLElement>) => {
      if (!drag) return;
      const to = indexAt(e.clientY);
      if (to !== drag.to) setDrag({ ...drag, to });
    },
    onPointerUp: () => {
      if (!drag) return;
      setDrag(null);
      if (drag.to !== drag.from) onDrop(drag.from, drag.to);
    },
    onPointerCancel: () => setDrag(null),
  });

  return { drag, handlers };
}
