import { create } from "zustand";
import type { RepoInfo } from "../lib/api";

/** 작업 공간 왼쪽 칸의 탭(D-025). */
export type LeftTab = "changes" | "history";

type RepoState = {
  /** 작업 공간에 열린 저장소. 없으면 홈 화면. */
  repo: RepoInfo | null;
  leftTab: LeftTab;
  /** 비교 보기에 띄울 파일(저장소 기준 경로). */
  selectedPath: string | null;
  openRepo: (repo: RepoInfo) => void;
  closeRepo: () => void;
  setLeftTab: (tab: LeftTab) => void;
  selectPath: (path: string | null) => void;
};

export const useRepoStore = create<RepoState>()((set) => ({
  repo: null,
  leftTab: "changes",
  selectedPath: null,
  openRepo: (repo) => set({ repo, leftTab: "changes", selectedPath: null }),
  closeRepo: () => set({ repo: null, selectedPath: null }),
  setLeftTab: (leftTab) => set({ leftTab }),
  selectPath: (selectedPath) => set({ selectedPath }),
}));
