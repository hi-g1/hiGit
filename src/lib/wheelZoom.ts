// Ctrl+휠·터치패드 핀치로 확대·축소(D-095). 휠의 Ctrl 판정은 이 파일에만 둔다(단축키 점검 규칙, D-056).
import { type RefObject, useEffect, useRef } from "react";

/** 한 단계로 보는 휠 양. 터치패드는 작은 값이 여러 번 오므로 모아서 판단한다. */
const STEP = 80;

/**
 * `ref` 요소 위의 Ctrl+휠(터치패드 핀치는 웹뷰가 Ctrl+휠로 보낸다)을 받아 페이지 확대 대신
 * `onZoom("in" | "out")`을 부른다. Ctrl 없는 휠은 그대로 스크롤이다.
 */
export function useWheelZoom(ref: RefObject<HTMLElement | null>, onZoom: (dir: "in" | "out") => void): void {
  const latest = useRef(onZoom);
  useEffect(() => {
    latest.current = onZoom;
  });
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    let sum = 0;
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      sum += e.deltaY;
      if (Math.abs(sum) < STEP && Math.abs(e.deltaY) < STEP) return;
      latest.current(sum < 0 ? "in" : "out");
      sum = 0;
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, [ref]);
}
