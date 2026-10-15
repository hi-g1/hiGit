import { create } from "zustand";
import type { Picks, Place } from "./model";

/** 고른 것을 기기에 남기는 키. 저장소와 합칠 버전이 같을 때만 이어 쓴다. */
const STORAGE_PREFIX = "higit.conflict.";

function load(key: string): Picks {
  try {
    const raw = window.localStorage.getItem(STORAGE_PREFIX + key);
    return raw ? (JSON.parse(raw) as Picks) : {};
  } catch {
    return {};
  }
}

function save(key: string, picks: Picks) {
  try {
    window.localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(picks));
  } catch {
    // 남기지 못해도 화면은 동작한다(다시 고르면 된다).
  }
}

function forget(key: string) {
  try {
    window.localStorage.removeItem(STORAGE_PREFIX + key);
  } catch {
    // 무시
  }
}

type ConflictState = {
  /** `저장소|합칠 버전` */
  key: string | null;
  picks: Picks;
  current: Place | null;
  /** 원래 코드(base) 펼침 */
  showBase: boolean;
  /** 합치는 중인 저장소를 열면 부른다. 같은 합치기면 고른 것을 이어 쓴다. */
  open: (key: string) => void;
  setPicks: (picks: Picks) => void;
  setCurrent: (place: Place | null) => void;
  toggleBase: () => void;
  /** 마치거나 그만두면 비운다. */
  clear: () => void;
};

/** 충돌 해결 화면에서 고른 것(D-086). 앱을 껐다 켜도 이어지게 기기에 남긴다(없어도 동작). */
export const useConflictStore = create<ConflictState>((set, get) => ({
  key: null,
  picks: {},
  current: null,
  showBase: false,
  open: (key) => {
    if (get().key === key) return;
    set({ key, picks: load(key), current: null, showBase: false });
  },
  setPicks: (picks) => {
    const key = get().key;
    if (key) save(key, picks);
    set({ picks });
  },
  setCurrent: (current) => set({ current }),
  toggleBase: () => set((s) => ({ showBase: !s.showBase })),
  clear: () => {
    const key = get().key;
    if (key) forget(key);
    set({ key: null, picks: {}, current: null, showBase: false });
  },
}));
