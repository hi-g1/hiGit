import { useEffect, useRef, useState } from "react";

/** 불러오는 표시를 보이기까지 기다리는 시간(D-159, D-167). 그 안에 끝나면 아무것도 그리지 않는다. */
export const LOADING_DELAY_MS = 300;
/** 한 번 보인 불러오는 표시를 남겨 두는 최소 시간(D-193). 깜빡 보였다 사라지지 않게. */
export const LOADING_HOLD_MS = 500;

/**
 * `pending`이 `delay` 넘게 이어질 때만 참이 되고, 한 번 참이 되면 `pending`이 끝나도 `hold`만큼은 참으로 남는다
 * (D-193). 짧은 기다림에는 아무것도 번쩍이지 않고, 보인 표시는 바로 사라지지 않는다.
 */
export function useDelayedFlag(
  pending: boolean,
  { delay = LOADING_DELAY_MS, hold = LOADING_HOLD_MS }: { delay?: number; hold?: number } = {},
): boolean {
  const [shown, setShown] = useState(false);
  const shownAt = useRef(0);
  useEffect(() => {
    if (pending) {
      if (shown) return;
      const timer = setTimeout(() => {
        shownAt.current = Date.now();
        setShown(true);
      }, delay);
      return () => clearTimeout(timer);
    }
    if (!shown) return;
    const left = Math.max(0, hold - (Date.now() - shownAt.current));
    const timer = setTimeout(() => setShown(false), left);
    return () => clearTimeout(timer);
  }, [pending, shown, delay, hold]);
  return shown;
}
