import { create } from "zustand";
import type { ErrorCode } from "../../lib/api";

type SyncState = {
  /** 마지막 동기화에서 겹쳐 합치지 않은 파일. 다음 동기화가 성공하면 지운다(D-048). */
  overlap: string[];
  /** 마지막 동기화 오류. 상단 바 아래 안내 줄과 "자세히 보기"로 보인다. */
  error: unknown;
  /** 저절로 하는 원격 확인이 실패한 시각(ms). 성공하면 null. */
  fetchFailedAt: number | null;
  /** 그 실패의 오류 코드. 알림 줄의 해결 버튼을 고른다(D-197). */
  fetchFailedCode: ErrorCode | null;
  /** 원격 확인 실패 알림 줄을 닫았다. 다음 실패에서 다시 보인다. */
  fetchNoticeDismissed: boolean;
  /** 마지막으로 원격을 확인한 시각(ms). 메모리에만 둔다(D-197). */
  lastCheckedAt: number | null;
  /** 인증 실패로 저절로 하는 확인을 멈췄다. 사용자가 동기화를 누르면 다시 시작한다(D-055). */
  paused: boolean;
  set: (patch: Partial<Omit<SyncState, "set">>) => void;
};

export const useSyncStore = create<SyncState>((set) => ({
  overlap: [],
  error: null,
  fetchFailedAt: null,
  fetchFailedCode: null,
  fetchNoticeDismissed: false,
  lastCheckedAt: null,
  paused: false,
  set: (patch) => set(patch),
}));
