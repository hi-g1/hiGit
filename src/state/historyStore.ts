import { create } from "zustand";
import type { Zoom } from "../features/history/layout/simplify";
import type { LogScope } from "../lib/api";

type HistoryState = {
  /** 고른 역(버전 해시나 접힌 구간 id). 구간을 고르는 중이면 움직이는 끝. */
  selectedId: string | null;
  /** 구간 선택의 고정된 끝(D-106). */
  anchorId: string | null;
  /** 고른 구간(지금 브랜치 선 위, 새 것부터). 비면 하나만 고른 상태. */
  rangeIds: readonly string[];
  zoom: Zoom;
  scope: LogScope;
  /** 펼친 접힌 구간 id. 단계가 바뀌면 지운다. */
  expanded: ReadonlySet<string>;
  /** 하나만 고른다(구간은 지운다). */
  select: (id: string | null) => void;
  /** 구간을 고른다: 움직이는 끝과 구간. */
  selectRange: (anchor: string, active: string, ids: readonly string[]) => void;
  clearRange: () => void;
  setZoom: (zoom: Zoom) => void;
  setScope: (scope: LogScope) => void;
  expand: (id: string) => void;
};

/** 버전 기록 노선도 상태(D-095). 앱이 켜져 있는 동안만 기억한다. */
export const useHistoryStore = create<HistoryState>((set) => ({
  selectedId: null,
  anchorId: null,
  rangeIds: [],
  zoom: "all",
  scope: "all",
  expanded: new Set(),
  select: (selectedId) => set({ selectedId, anchorId: selectedId, rangeIds: [] }),
  selectRange: (anchorId, selectedId, rangeIds) => set({ anchorId, selectedId, rangeIds }),
  clearRange: () => set((s) => ({ anchorId: s.selectedId, rangeIds: [] })),
  setZoom: (zoom) => set({ zoom, expanded: new Set(), rangeIds: [] }),
  setScope: (scope) => set({ scope, expanded: new Set(), rangeIds: [] }),
  expand: (id) => set((s) => ({ expanded: new Set(s.expanded).add(id) })),
}));
