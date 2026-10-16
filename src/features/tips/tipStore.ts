import { create } from "zustand";
import type { TipId } from "../../lib/api";

/** 이번 실행에서 그 일을 처음 해서 안내를 띄울 차례가 된 것(D-205). 닫았는지는 설정에 남는다. */
export const useTipStore = create<{ due: TipId[]; trigger: (id: TipId) => void }>((set) => ({
  due: [],
  trigger: (id) => set((s) => (s.due.includes(id) ? s : { due: [...s.due, id] })),
}));
