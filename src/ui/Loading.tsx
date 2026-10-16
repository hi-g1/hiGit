import { useDelayedFlag } from "../lib/useDelayed";
import { useMessages } from "../messages";
import { EmptyState } from "./EmptyState";
import { Skeleton } from "./Skeleton";

/**
 * 불러오는 표시(D-159, D-193): 300ms가 넘을 때만 보이고 한 번 보이면 0.5초 남는다. `skeleton`이면 처음 로드의 빈 줄
 * 모양, 아니면 "불러오는 중".
 */
export function Loading({ pending, skeleton }: { pending: boolean; skeleton?: number }) {
  const m = useMessages();
  if (!useDelayedFlag(pending)) return null;
  return skeleton ? <Skeleton lines={skeleton} /> : <EmptyState>{m.loading}</EmptyState>;
}
