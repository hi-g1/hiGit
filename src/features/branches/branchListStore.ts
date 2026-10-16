import { create } from "zustand";
import type { SwitchTarget } from "./useSwitch";

type BranchListState = {
  /** 브랜치 목록 팝오버(Ctrl+B, 명령 팔레트). */
  open: boolean;
  setOpen: (open: boolean | ((open: boolean) => boolean)) => void;
  /** 고친 파일이 있어 두고 갈지 옮길지 묻는 갈아타기(D-195). 목록과 팔레트가 연다. */
  asking: { target: SwitchTarget; files: number } | null;
  setAsking: (asking: { target: SwitchTarget; files: number } | null) => void;
};

export const useBranchListStore = create<BranchListState>((set) => ({
  open: false,
  setOpen: (open) => set((s) => ({ open: typeof open === "function" ? open(s.open) : open })),
  asking: null,
  setAsking: (asking) => set({ asking }),
}));
