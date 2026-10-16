import { useEffect, useState } from "react";

/** 불러오는 표시를 보이기까지 기다리는 시간(D-159, D-167). 그 안에 끝나면 아무것도 그리지 않는다. */
export const LOADING_DELAY_MS = 300;

/** `pending`이 `ms` 넘게 이어질 때만 참. 짧은 기다림에 "불러오는 중"이 번쩍이지 않게 한다. */
export function useDelayedFlag(pending: boolean, ms = LOADING_DELAY_MS): boolean {
  const [late, setLate] = useState(false);
  useEffect(() => {
    if (!pending) return;
    const timer = setTimeout(() => setLate(true), ms);
    return () => {
      clearTimeout(timer);
      setLate(false);
    };
  }, [pending, ms]);
  return pending && late;
}
