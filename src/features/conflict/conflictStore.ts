import { create } from "zustand";
import { api, type ConflictDraftView } from "../../lib/api";
import type { FilePick, Picks, Place } from "./model";

/** 고른 것을 저장소에 보내기 전 모으는 시간(D-211). */
export const SAVE_DELAY_MS = 500;

let timer: ReturnType<typeof setTimeout> | null = null;

/** 화면의 선택 ↔ core의 초안. 모양이 같아 그대로 옮긴다. */
export function toDraft(picks: Picks): ConflictDraftView {
  return {
    files: Object.entries(picks)
      .filter((e): e is [string, FilePick] => e[1] !== undefined)
      .map(([path, pick]) => ({ path, pick })),
  };
}

export function fromDraft(draft: ConflictDraftView | null | undefined): Picks {
  const picks: Picks = {};
  for (const f of draft?.files ?? []) picks[f.path] = f.pick as FilePick;
  return picks;
}

type ConflictState = {
  /** `저장소|합칠 버전` */
  key: string | null;
  repo: string | null;
  picks: Picks;
  current: Place | null;
  /** 원래 코드(base) 펼침 */
  showBase: boolean;
  /** 합치는 중인 저장소를 열면 부른다. 같은 합치기면 지금 선택을 두고, 처음이면 저장소의 초안으로 채운다. */
  open: (key: string, repo: string, draft?: ConflictDraftView | null) => void;
  setPicks: (picks: Picks) => void;
  setCurrent: (place: Place | null) => void;
  toggleBase: () => void;
  /** 마치거나 그만두면 비운다(저장소의 초안은 core가 지운다). */
  clear: () => void;
};

/**
 * 충돌 해결 화면에서 고른 것(D-086). 앱을 껐다 켜도 이어지게 git 폴더 안에 서명해 둔다(D-211). 웹뷰 저장소에는
 * 두지 않는다. 저장하지 못해도 화면은 동작한다.
 */
export const useConflictStore = create<ConflictState>((set, get) => ({
  key: null,
  repo: null,
  picks: {},
  current: null,
  showBase: false,
  open: (key, repo, draft) => {
    if (get().key === key) return;
    if (timer !== null) clearTimeout(timer);
    timer = null;
    set({ key, repo, picks: fromDraft(draft), current: null, showBase: false });
  },
  setPicks: (picks) => {
    set({ picks });
    const repo = get().repo;
    if (!repo) return;
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      void api.saveConflictDraft(repo, toDraft(get().picks)).catch(() => undefined);
    }, SAVE_DELAY_MS);
  },
  setCurrent: (current) => set({ current }),
  toggleBase: () => set((s) => ({ showBase: !s.showBase })),
  clear: () => {
    if (timer !== null) clearTimeout(timer);
    timer = null;
    set({ key: null, repo: null, picks: {}, current: null, showBase: false });
  },
}));
