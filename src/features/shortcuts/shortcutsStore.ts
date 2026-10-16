import { create } from "zustand";

/** 단축키 표 대화상자(D-198). Ctrl+/, 설정 "일반", 명령 팔레트에서 연다. */
export const useShortcutsStore = create<{ open: boolean; setOpen: (open: boolean) => void }>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}));
