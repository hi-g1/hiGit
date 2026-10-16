import { create } from "zustand";

/** 명령 팔레트(D-170). Ctrl+K로 열고 닫는다. */
export const usePaletteStore = create<{ open: boolean; setOpen: (open: boolean) => void }>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}));
