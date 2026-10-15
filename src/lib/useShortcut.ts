import { useEffect, useRef } from "react";
import { matches, type ShortcutId } from "./shortcuts";

/** 창 전체에서 단축키를 받는다. 입력칸 안에서도 Ctrl 조합은 동작한다. */
export function useShortcut(id: ShortcutId, handler: () => void, enabled = true) {
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    if (!enabled) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (!matches(id, e)) return;
      e.preventDefault();
      latest.current();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [id, enabled]);
}
