// 요소가 한 번이라도 화면에 들어왔는지(시작 화면 작은 노선도를 보일 때만 불러오기, D-093).
import { type RefObject, useEffect, useState } from "react";

/** IntersectionObserver가 없는 환경(jsdom 등)에서는 늘 보인 것으로 본다. */
export function useInView(ref: RefObject<Element | null>): boolean {
  const [seen, setSeen] = useState(() => typeof IntersectionObserver === "undefined");
  useEffect(() => {
    const el = ref.current;
    if (seen || !el || typeof IntersectionObserver === "undefined") return;
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        setSeen(true);
        observer.disconnect();
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [ref, seen]);
  return seen;
}
