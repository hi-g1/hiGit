import { create } from "zustand";

export type ToastAction = {
  label: string;
  /** 옅게 보이는 단축키(예: Ctrl+Z) */
  kbd?: string;
  onClick: () => void;
  /**
   * "되돌리기"(가장 최근 작업을 되돌림)면 `undo`. 새 되돌리기 토스트가 오면 앞 토스트의 단추는 `stale`(작업 내역 등)로
   * 바뀐다(D-192): 앞 단추를 누르면 엉뚱한(더 새) 작업이 되돌려지기 때문이다.
   */
  kind?: "undo";
  stale?: ToastAction;
};

export type ToastMessage = {
  /** 같은 문구를 다시 띄워도 새 토스트가 되게 한다. */
  id: number;
  text: string;
  action?: ToastAction;
  /** 오류 토스트(D-166)면 그 오류. 문구는 오류 코드로 만든다. */
  error?: { value: unknown };
};

/** 한꺼번에 보이는 평범한 토스트와 오류 토스트의 수(D-192). */
export const MAX_TOASTS = 3;
export const MAX_ERROR_TOASTS = 2;

type ToastState = {
  /** 오래된 것부터. */
  toasts: ToastMessage[];
  /** 가장 최근 토스트(없으면 null). */
  toast: ToastMessage | null;
  show: (text: string, action?: ToastAction) => void;
  /** 오류는 성공과 다른 모양으로, 닫을 때까지 남고 "자세히 보기"와 해결 버튼 하나를 둔다(D-166). */
  showError: (error: unknown, action?: ToastAction) => void;
  /** `id`의 토스트를 닫는다. 없으면 모두. */
  clear: (id?: number) => void;
};

let next = 1;

const latest = (toasts: ToastMessage[]) => toasts.at(-1) ?? null;

/** `max`를 넘으면 `pick`이 고른 가장 오래된 것부터 뺀다. */
function trim(toasts: ToastMessage[], kind: (t: ToastMessage) => boolean, max: number) {
  const out = [...toasts];
  while (out.filter(kind).length > max) {
    const index = out.findIndex(kind);
    out.splice(index, 1);
  }
  return out;
}

/** 화면 아래 뜨는 결과 알림(D-042)과 오류 알림(D-166). 셋까지 쌓고 오류는 둘까지 닫을 때까지 남는다(D-192). */
export const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  toast: null,
  show: (text, action) =>
    set((s) => {
      let toasts = s.toasts;
      if (action?.kind === "undo") {
        toasts = toasts.map((t) => (t.action?.kind === "undo" ? { ...t, action: t.action.stale } : t));
      }
      toasts = trim([...toasts, { id: next++, text, action }], (t) => !t.error, MAX_TOASTS);
      return { toasts, toast: latest(toasts) };
    }),
  showError: (error, action) =>
    set((s) => {
      const toasts = trim([...s.toasts, { id: next++, text: "", action, error: { value: error } }], (t) => !!t.error, MAX_ERROR_TOASTS);
      return { toasts, toast: latest(toasts) };
    }),
  clear: (id) =>
    set((s) => {
      const toasts = id === undefined ? [] : s.toasts.filter((t) => t.id !== id);
      return { toasts, toast: latest(toasts) };
    }),
}));
