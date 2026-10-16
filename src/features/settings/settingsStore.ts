import { create } from "zustand";

type SettingsState = {
  open: boolean;
  /** 열자마자 GitHub 연결을 시작한다(복제·검토 요청의 "GitHub 연결" 단추). */
  connect: boolean;
  openSettings: (connect?: boolean) => void;
  close: () => void;
};

/** 설정 대화상자(D-116). 시작 화면·작업 공간 어디서나 연다. */
export const useSettingsStore = create<SettingsState>((set) => ({
  open: false,
  connect: false,
  openSettings: (connect = false) => set({ open: true, connect }),
  close: () => set({ open: false, connect: false }),
}));
