import { useEffect } from "react";
import { onOpProgress, REMOTE_ACTION_IDS, type OpProgress } from "../../lib/events";
import { useProgressStore } from "./progressStore";

/** 원격 작업. 취소할 수 있고 오래 출력이 없으면 "응답 없음"을 보인다. */
export const REMOTE_ACTIONS: ReadonlySet<string> = new Set(REMOTE_ACTION_IDS);
/** 이만큼 지나면 "취소"가 나타난다. */
export const CANCEL_AFTER_SECS = 3;
/** 원격 작업이 이만큼 출력이 없으면 "응답 없음"(D-080). */
export const NO_RESPONSE_SECS = 30;
/** 로컬 쓰기가 이만큼 끝나지 않으면 "강제로 멈추기"(D-080). */
export const STUCK_LOCAL_SECS = 60;

/** 진행 이벤트를 받아 둔다. 작업 공간에서 한 번 켠다. */
export function useProgressEvents(): void {
  const receive = useProgressStore((s) => s.receive);
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    void onOpProgress(receive).then((un) => {
      if (disposed) un();
      else unlisten = un;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [receive]);
}

/** 이 저장소에서 도는 작업의 진행. 없으면 null. */
export function useProgress(repo: string): OpProgress | null {
  return useProgressStore((s) => s.byRepo[repo] ?? null);
}
