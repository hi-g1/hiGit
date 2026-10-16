import { create } from "zustand";
import type { Locale } from "../glossary";

/** 용어를 본문에 어떤 이름으로 보일지. 설정 "Git 원래 이름으로 보기"가 켜지면 "git" */
export type TermDisplay = "friendly" | "git";

type UiState = {
  termDisplay: TermDisplay;
  locale: Locale;
  /** 화면 배율 = 루트 글자 크기 / 16px(D-146). 노선도 SVG가 그릴 때 곱한다(D-153). */
  uiScale: number;
  setTermDisplay: (display: TermDisplay) => void;
  setLocale: (locale: Locale) => void;
  setUiScale: (scale: number) => void;
};

export const useUiStore = create<UiState>()((set) => ({
  termDisplay: "friendly",
  locale: "ko",
  uiScale: 1,
  setTermDisplay: (termDisplay) => set({ termDisplay }),
  setLocale: (locale) => set({ locale }),
  setUiScale: (uiScale) => set({ uiScale }),
}));
