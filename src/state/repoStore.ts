import { create } from "zustand";
import type { RepoInfo } from "../lib/api";

/** 마지막 프로젝트를 열지 못해 시작 화면에 온 까닭(D-032). */
export type StartNotice = { kind: "lastMissing" | "lastNotRepo"; name: string };

/** 작업 공간 화면: 홈(고친 파일과 비교) 또는 버전 기록(D-058). */
export type WorkspaceView = "home" | "history";

type RepoState = {
  /** 작업 공간에 열린 저장소. 없으면 홈 화면. */
  repo: RepoInfo | null;
  view: WorkspaceView;
  /** 비교 보기에 띄울 파일(저장소 기준 경로). */
  selectedPath: string | null;
  /** 저장소가 아닌 폴더를 골랐을 때, 시작 화면이 "버전 관리 시작"을 물을 폴더. */
  pendingInit: string | null;
  startNotice: StartNotice | null;
  /** 시작 화면의 "바뀐 파일 N개 보기": 작업 공간이 열리면 첫 바뀐 파일을 고른다. */
  selectFirstChange: boolean;
  openRepo: (repo: RepoInfo) => void;
  closeRepo: () => void;
  setView: (view: WorkspaceView) => void;
  selectPath: (path: string | null) => void;
  setPendingInit: (folder: string | null) => void;
  setStartNotice: (notice: StartNotice | null) => void;
  setSelectFirstChange: (on: boolean) => void;
};

export const useRepoStore = create<RepoState>()((set) => ({
  repo: null,
  view: "home",
  selectedPath: null,
  pendingInit: null,
  startNotice: null,
  selectFirstChange: false,
  openRepo: (repo) =>
    set({ repo, view: "home", selectedPath: null, pendingInit: null, startNotice: null }),
  closeRepo: () => set({ repo: null, selectedPath: null, startNotice: null }),
  setView: (view) => set({ view }),
  selectPath: (selectedPath) => set({ selectedPath }),
  setPendingInit: (pendingInit) => set({ pendingInit }),
  setStartNotice: (startNotice) => set({ startNotice }),
  setSelectFirstChange: (selectFirstChange) => set({ selectFirstChange }),
}));
