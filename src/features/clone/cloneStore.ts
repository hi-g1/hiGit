import { create } from "zustand";

type CloneState = {
  open: boolean;
  openClone: () => void;
  close: () => void;
};

/** 복제 대화상자(D-115). 시작 화면의 단추와 어디서나 Ctrl+Shift+O. */
export const useCloneStore = create<CloneState>((set) => ({
  open: false,
  openClone: () => set({ open: true }),
  close: () => set({ open: false }),
}));
