import { create } from "zustand";
import type { ChecksView, MergeableView, PrRowView } from "../../lib/api";

/** 합치기 확인 창이 보여 줄 것(D-179). 상세가 읽은 값 그대로이고 core가 실행 때 다시 확인한다. */
export type MergeTarget = {
  row: PrRowView;
  headOid: string;
  mergeable: MergeableView;
  checks: ChecksView | null;
};

type MergeState = {
  target: MergeTarget | null;
  open: (target: MergeTarget) => void;
  close: () => void;
};

export const useMergeStore = create<MergeState>((set) => ({
  target: null,
  open: (target) => set({ target }),
  close: () => set({ target: null }),
}));
