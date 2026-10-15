import { create } from "zustand";

type UndoState = {
  /** 버전 저장을 되돌렸을 때 그 메시지. 이름 칸이 받아 다시 채우고 지운다. */
  restoredMessage: string | null;
  setRestoredMessage: (message: string | null) => void;
};

export const useUndoStore = create<UndoState>((set) => ({
  restoredMessage: null,
  setRestoredMessage: (restoredMessage) => set({ restoredMessage }),
}));
