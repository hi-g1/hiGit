import type { OpProgress } from "../../lib/events";
import { useMessages } from "../../messages";
import { ProgressBar } from "../../ui/ProgressBar";

/** 이만큼 지난 원격 작업만 Git이 준 개수와 진행 막대를 보인다(D-172). 짧은 작업은 지금처럼 경과만. */
export const COUNT_AFTER_SECS = 10;

/** 보일 개수가 있으면 문구, 없으면 null. */
export function useCountText(tick: OpProgress | null): { text: string; done: number; total: number } | null {
  const m = useMessages();
  if (!tick || tick.elapsedSecs < COUNT_AFTER_SECS || tick.total === null || tick.done === null || tick.phase === null) {
    return null;
  }
  return { text: m.progress[tick.phase](tick.done, tick.total), done: tick.done, total: tick.total };
}

/**
 * 원격 작업의 진행 자리(D-172). 10초가 지나고 Git이 개수를 주면 "파일 7/200개 받는 중"과 막대, 아니면 `fallback`
 * (지금의 "동기화 중 · N초" 같은 문구). 원문 줄은 보이지 않고 숫자만.
 */
export function ProgressCount({ tick, fallback }: { tick: OpProgress | null; fallback: string }) {
  const count = useCountText(tick);
  if (!count) return <>{fallback}</>;
  return (
    <>
      {count.text} <ProgressBar value={count.done} max={count.total} label={count.text} />
    </>
  );
}
