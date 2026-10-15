import { useEffect, useRef } from "react";
import { SHORTCUTS, isTextField, matches, type ShortcutId } from "./shortcuts";

/**
 * 창 전체에서 단축키를 받는다. 입력칸 안에서도 Ctrl 조합은 동작한다. 다만 표에서
 * `inTextField: "native"`인 키(Ctrl+Z 등)는 입력칸에 포커스가 있으면 브라우저에 맡긴다(D-042).
 */
export function useShortcut(id: ShortcutId, handler: () => void, enabled = true) {
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    if (!enabled) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (!matches(id, e)) return;
      if (SHORTCUTS[id].inTextField === "native" && isTextField(e.target)) return;
      e.preventDefault();
      latest.current();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [id, enabled]);
}
