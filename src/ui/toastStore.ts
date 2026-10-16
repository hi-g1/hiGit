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
  /** 오류 토스트(D-166)면 그 오류. 문구는 오류 코드로 만든다. */
  error?: { value: unknown };
};

type ToastState = {
  toast: ToastMessage | null;
  show: (text: string, action?: ToastAction) => void;
  /** 오류는 성공과 다른 모양으로, 닫을 때까지 남고 "자세히 보기"와 해결 버튼 하나를 둔다(D-166). */
  showError: (error: unknown, action?: ToastAction) => void;
  clear: () => void;
};

let next = 1;

/** 화면 아래 뜨는 결과 알림(D-042)과 오류 알림(D-166). 한 번에 하나만 보인다. */
export const useToastStore = create<ToastState>((set) => ({
  toast: null,
  show: (text, action) => set({ toast: { id: next++, text, action } }),
  showError: (error, action) => set({ toast: { id: next++, text: "", action, error: { value: error } } }),
  clear: () => set({ toast: null }),
}));
