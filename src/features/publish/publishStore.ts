import { create } from "zustand";

/** GitHub에 올리기 대화상자(D-132). 상단 바의 "GitHub에 올리기"가 연다. */
type PublishState = {
  open: boolean;
  openDialog: () => void;
  close: () => void;
};

export const usePublishStore = create<PublishState>((set) => ({
  open: false,
  openDialog: () => set({ open: true }),
  close: () => set({ open: false }),
}));
