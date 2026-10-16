import { create } from "zustand";

/** 앱 메뉴를 연 자리(D-189): 입력칸이면 잘라내기·복사·붙여넣기·모두 선택, 선택한 글자면 복사만. */
export type TextMenuTarget =
  | {
      kind: "field";
      field: HTMLInputElement | HTMLTextAreaElement;
      /** 메뉴를 열 때의 선택 범위(메뉴가 포커스를 가져가도 되살린다). */
      start: number;
      end: number;
      x: number;
      y: number;
    }
  | { kind: "selection"; text: string; x: number; y: number };

type TextMenuState = {
  target: TextMenuTarget | null;
  /** 이번 실행에 붙여넣기 읽기가 거절됐다. */
  pasteBlocked: boolean;
  open: (target: TextMenuTarget) => void;
  close: () => void;
  blockPaste: () => void;
};

export const useTextMenuStore = create<TextMenuState>((set) => ({
  target: null,
  pasteBlocked: false,
  open: (target) => set({ target }),
  close: () => set({ target: null }),
  blockPaste: () => set({ pasteBlocked: true }),
}));
