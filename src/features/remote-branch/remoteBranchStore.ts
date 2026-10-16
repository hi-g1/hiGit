import { create } from "zustand";

export type RemoteBranchTarget = { remote: string; branch: string; mode: "delete" | "rename" };

/** 원격 브랜치 지우기·이름 바꾸기 확인 창(D-130). 브랜치 목록의 원격 줄 `…`이 연다. */
type RemoteBranchState = {
  target: RemoteBranchTarget | null;
  open: (target: RemoteBranchTarget) => void;
  close: () => void;
};

export const useRemoteBranchStore = create<RemoteBranchState>((set) => ({
  target: null,
  open: (target) => set({ target }),
  close: () => set({ target: null }),
}));
