import { create } from "zustand";
import type { Locale } from "../glossary";

/** 용어를 본문에 어떤 이름으로 보일지. 설정 "Git 원래 이름으로 보기"가 켜지면 "git" */
export type TermDisplay = "friendly" | "git";

type UiState = {
  termDisplay: TermDisplay;
  locale: Locale;
  setTermDisplay: (display: TermDisplay) => void;
  setLocale: (locale: Locale) => void;
};

export const useUiStore = create<UiState>()((set) => ({
  termDisplay: "friendly",
  locale: "ko",
  setTermDisplay: (termDisplay) => set({ termDisplay }),
  setLocale: (locale) => set({ locale }),
}));
