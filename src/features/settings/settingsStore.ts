import { create } from "zustand";

type SettingsState = {
  open: boolean;
  /** 열자마자 GitHub 연결을 시작한다(복제·검토 요청의 "GitHub 연결" 단추). */
  connect: boolean;
  /** 연결을 `repo workflow` 권한으로 시작한다(D-135, "워크플로 권한 추가"). `connect`와 함께 켠다. */
  workflow: boolean;
  openSettings: (connect?: boolean) => void;
  openWorkflowConnect: () => void;
  close: () => void;
};

/** 설정 대화상자(D-116). 시작 화면·작업 공간 어디서나 연다. */
export const useSettingsStore = create<SettingsState>((set) => ({
  open: false,
  connect: false,
  workflow: false,
  openSettings: (connect = false) => set({ open: true, connect, workflow: false }),
  openWorkflowConnect: () => set({ open: true, connect: true, workflow: true }),
  close: () => set({ open: false, connect: false, workflow: false }),
}));
