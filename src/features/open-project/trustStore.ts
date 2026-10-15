import { create } from "zustand";

export type PendingTrust = { folder: string; keys: string[] };

type TrustState = {
  /** 믿을지 물어야 하는 프로젝트(D-039). 있으면 신뢰 확인 창이 뜬다. */
  pending: PendingTrust | null;
  ask: (p: PendingTrust) => void;
  clear: () => void;
};

export const useTrustStore = create<TrustState>((set) => ({
  pending: null,
  ask: (pending) => set({ pending }),
  clear: () => set({ pending: null }),
}));
