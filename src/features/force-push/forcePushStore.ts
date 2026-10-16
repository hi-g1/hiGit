import { create } from "zustand";

/** 덮어쓰며 올리기 확인 창(D-128). 역 메뉴·동기화 멈춤 줄·"올린 버전을 바꿨어요" 줄이 같은 창을 연다. */
type ForcePushState = {
  open: boolean;
  openDialog: () => void;
  close: () => void;
};

export const useForcePushStore = create<ForcePushState>((set) => ({
  open: false,
  openDialog: () => set({ open: true }),
  close: () => set({ open: false }),
}));
