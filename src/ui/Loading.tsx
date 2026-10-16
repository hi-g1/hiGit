import { useDelayedFlag } from "../lib/useDelayed";
import { useMessages } from "../messages";
import { EmptyState } from "./EmptyState";

/** "불러오는 중"을 300ms가 넘을 때만 보인다(D-159). */
export function Loading({ pending }: { pending: boolean }) {
  const m = useMessages();
  return useDelayedFlag(pending) ? <EmptyState>{m.loading}</EmptyState> : null;
}
