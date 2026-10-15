import { create } from "zustand";

export type PendingKind = "locks" | "parked" | "other";

type PendingState = {
  /** 닫은 알림 줄(`저장소\0종류`). 이번 실행에서만 기억한다. */
  dismissed: ReadonlySet<string>;
  dismiss: (repo: string, kind: PendingKind) => void;
};

export const pendingKey = (repo: string, kind: PendingKind) => `${repo}\0${kind}`;

/** 끝나지 않은 작업 알림 줄의 닫힘 상태(D-082). 저장하지 않는다. */
export const usePendingStore = create<PendingState>((set) => ({
  dismissed: new Set(),
  dismiss: (repo, kind) => set((s) => ({ dismissed: new Set(s.dismissed).add(pendingKey(repo, kind)) })),
}));
