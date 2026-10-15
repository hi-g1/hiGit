import { create } from "zustand";
import type { Zoom } from "../features/history/layout/simplify";
import type { LogScope } from "../lib/api";

type HistoryState = {
  /** 고른 역(버전 해시나 접힌 구간 id). 하나만(여러 개는 반복 3의 기록 정리). */
  selectedId: string | null;
  zoom: Zoom;
  scope: LogScope;
  /** 펼친 접힌 구간 id. 단계가 바뀌면 지운다. */
  expanded: ReadonlySet<string>;
  select: (id: string | null) => void;
  setZoom: (zoom: Zoom) => void;
  setScope: (scope: LogScope) => void;
  expand: (id: string) => void;
};

/** 버전 기록 노선도 상태(D-095). 앱이 켜져 있는 동안만 기억한다. */
export const useHistoryStore = create<HistoryState>((set) => ({
  selectedId: null,
  zoom: "all",
  scope: "all",
  expanded: new Set(),
  select: (selectedId) => set({ selectedId }),
  setZoom: (zoom) => set({ zoom, expanded: new Set() }),
  setScope: (scope) => set({ scope, expanded: new Set() }),
  expand: (id) => set((s) => ({ expanded: new Set(s.expanded).add(id) })),
}));
