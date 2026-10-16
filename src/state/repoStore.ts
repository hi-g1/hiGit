import { create } from "zustand";
import type { RepoInfo } from "../lib/api";

/** 마지막 프로젝트나 실행 인자의 폴더(D-213)를 열지 못해 시작 화면에 온 까닭(D-032). */
export type StartNotice = { kind: "lastMissing" | "lastNotRepo" | "launchMissing"; name: string };

/** 작업 공간 화면: 홈(고친 파일과 비교), 버전 기록(D-058), 작업 내역(D-085), 기록 정리(D-106),
 * 검토 요청(D-112). */
export type WorkspaceView = "home" | "history" | "activity" | "rewrite" | "pr";

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
  /** 기록 정리 화면에서 고친다: 고른 버전(새 것부터). */
  rewriteOids: readonly string[];
  /** 검토 요청 화면에서 고른(펼친) 요청 번호. */
  selectedPr: number | null;
  openRepo: (repo: RepoInfo) => void;
  closeRepo: () => void;
  setView: (view: WorkspaceView) => void;
  selectPath: (path: string | null) => void;
  setPendingInit: (folder: string | null) => void;
  setStartNotice: (notice: StartNotice | null) => void;
  setSelectFirstChange: (on: boolean) => void;
  /** 고른 버전으로 기록 정리 화면을 연다. */
  openRewrite: (oids: readonly string[]) => void;
  /** 검토 요청 화면을 열고 그 요청을 고른다. */
  openPr: (number: number | null) => void;
  selectPr: (number: number | null) => void;
};

export const useRepoStore = create<RepoState>()((set) => ({
  repo: null,
  view: "home",
  selectedPath: null,
  pendingInit: null,
  startNotice: null,
  selectFirstChange: false,
  rewriteOids: [],
  selectedPr: null,
  openRepo: (repo) =>
    set({ repo, view: "home", selectedPath: null, pendingInit: null, startNotice: null, selectedPr: null }),
  closeRepo: () => set({ repo: null, selectedPath: null, startNotice: null }),
  setView: (view) => set({ view }),
  selectPath: (selectedPath) => set({ selectedPath }),
  setPendingInit: (pendingInit) => set({ pendingInit }),
  setStartNotice: (startNotice) => set({ startNotice }),
  setSelectFirstChange: (selectFirstChange) => set({ selectFirstChange }),
  openRewrite: (rewriteOids) => set({ rewriteOids, view: "rewrite" }),
  openPr: (selectedPr) => set({ selectedPr, view: "pr" }),
  selectPr: (selectedPr) => set({ selectedPr }),
}));
