import { create } from "zustand";
import type { ScopeAdd } from "../../lib/api";

type SettingsState = {
  open: boolean;
  /** 열자마자 GitHub 연결을 시작한다(복제·검토 요청의 "GitHub 연결" 단추). */
  connect: boolean;
  /** 연결을 이 권한을 더해 시작한다(D-135, D-186). `connect`와 함께 켠다. */
  add: ScopeAdd | null;
  openSettings: (connect?: boolean) => void;
  /** "워크플로 권한 추가"·"이메일 확인 권한 추가": 설정을 열고 그 권한으로 다시 연결한다. */
  openScopeConnect: (add: ScopeAdd) => void;
  close: () => void;
};

/** 설정 대화상자(D-116). 시작 화면·작업 공간 어디서나 연다. */
export const useSettingsStore = create<SettingsState>((set) => ({
  open: false,
  connect: false,
  add: null,
  openSettings: (connect = false) => set({ open: true, connect, add: null }),
  openScopeConnect: (add) => set({ open: true, connect: true, add }),
  close: () => set({ open: false, connect: false, add: null }),
}));
