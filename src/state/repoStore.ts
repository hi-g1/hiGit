import { create } from "zustand";
import type { RepoInfo } from "../lib/api";

/** 마지막 프로젝트를 열지 못해 시작 화면에 온 까닭(D-032). */
export type StartNotice = { kind: "lastMissing" | "lastNotRepo"; name: string };

/** 작업 공간 왼쪽 칸의 탭(D-025). */
export type LeftTab = "changes" | "history";

type RepoState = {
  /** 작업 공간에 열린 저장소. 없으면 홈 화면. */
  repo: RepoInfo | null;
  leftTab: LeftTab;
  /** 비교 보기에 띄울 파일(저장소 기준 경로). */
  selectedPath: string | null;
  /** 저장소가 아닌 폴더를 골랐을 때, 시작 화면이 "버전 관리 시작"을 물을 폴더. */
  pendingInit: string | null;
  startNotice: StartNotice | null;
  openRepo: (repo: RepoInfo) => void;
  closeRepo: () => void;
  setLeftTab: (tab: LeftTab) => void;
  selectPath: (path: string | null) => void;
  setPendingInit: (folder: string | null) => void;
  setStartNotice: (notice: StartNotice | null) => void;
};

export const useRepoStore = create<RepoState>()((set) => ({
  repo: null,
  leftTab: "changes",
  selectedPath: null,
  pendingInit: null,
  startNotice: null,
  openRepo: (repo) =>
    set({ repo, leftTab: "changes", selectedPath: null, pendingInit: null, startNotice: null }),
  closeRepo: () => set({ repo: null, selectedPath: null, startNotice: null }),
  setLeftTab: (leftTab) => set({ leftTab }),
  selectPath: (selectedPath) => set({ selectedPath }),
  setPendingInit: (pendingInit) => set({ pendingInit }),
  setStartNotice: (startNotice) => set({ startNotice }),
}));
