import { formatExact, formatTime } from "../lib/time";
import { useUiStore } from "../state/uiStore";

/**
 * 날짜·시각 글자(D-150). 정확한 시각은 네이티브 title로 보인다: 목록 줄·단추 안에 많이 들어가 툴팁 트리거를
 * 겹칠 수 없어서 이 부품만 예외다.
 */
export function Time({ at, now }: { at: number; now: number }) {
  const locale = useUiStore((s) => s.locale);
  return (
    <time dateTime={new Date(at * 1000).toISOString()} title={formatExact(at, locale)}>
      {formatTime(at, now, locale)}
    </time>
  );
}
