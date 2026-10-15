import { create } from "zustand";
import type { RepoInfo } from "../lib/api";

type RepoState = {
  /** 작업 공간에 열린 저장소. 없으면 홈 화면. */
  repo: RepoInfo | null;
  openRepo: (repo: RepoInfo) => void;
  closeRepo: () => void;
};

export const useRepoStore = create<RepoState>()((set) => ({
  repo: null,
  openRepo: (repo) => set({ repo }),
  closeRepo: () => set({ repo: null }),
}));
