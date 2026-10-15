import { create } from "zustand";

export type ToastAction = {
  label: string;
  /** 옅게 보이는 단축키(예: Ctrl+Z) */
  kbd?: string;
  onClick: () => void;
};

export type ToastMessage = {
  /** 같은 문구를 다시 띄워도 새 토스트가 되게 한다. */
  id: number;
  text: string;
  action?: ToastAction;
};

type ToastState = {
  toast: ToastMessage | null;
  show: (text: string, action?: ToastAction) => void;
  clear: () => void;
};

let next = 1;

/** 화면 아래 잠깐 뜨는 결과 알림(D-042). 한 번에 하나만 보인다. */
export const useToastStore = create<ToastState>((set) => ({
  toast: null,
  show: (text, action) => set({ toast: { id: next++, text, action } }),
  clear: () => set({ toast: null }),
}));
