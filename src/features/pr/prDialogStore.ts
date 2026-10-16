import { create } from "zustand";

type PrDialogState = {
  open: boolean;
  openPrDialog: () => void;
  close: () => void;
};

/** 새 검토 요청 대화상자(D-113). 검토 요청 화면과 브랜치 목록이 연다. */
export const usePrDialogStore = create<PrDialogState>((set) => ({
  open: false,
  openPrDialog: () => set({ open: true }),
  close: () => set({ open: false }),
}));
