import { useEffect, useRef } from "react";
import { SHORTCUTS, isTextField, matches, type ShortcutId } from "./shortcuts";

/** 모달 대화상자(확인 창 등)가 열려 있는지. 열려 있으면 뒤의 전역 단축키는 쉰다. */
export function modalOpen(): boolean {
  return document.querySelector('[role="dialog"][aria-modal="true"]') !== null;
}

/**
 * 창 전체에서 단축키를 받는다. 입력칸 안에서도 Ctrl 조합은 동작한다. 다만 표에서
 * `inTextField: "native"`인 키(Ctrl+Z 등)는 입력칸에 포커스가 있으면 브라우저에 맡긴다(D-042).
 * 모달 대화상자가 열려 있으면 아무것도 하지 않는다(확인 창 중 Ctrl+Z로 직전 작업이 되돌려지지 않게).
 */
export function useShortcut(id: ShortcutId, handler: () => void, enabled = true) {
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    if (!enabled) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (!matches(id, e) || modalOpen()) return;
      if (SHORTCUTS[id].inTextField === "native" && isTextField(e.target)) return;
      e.preventDefault();
      latest.current();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [id, enabled]);
}
