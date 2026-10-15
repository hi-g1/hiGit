import { create } from "zustand";
import type { OpProgress } from "../../lib/events";

/** 진행 알림은 1초마다 온다. 이만큼 오지 않으면 작업이 끝난 것으로 본다. */
export const PROGRESS_GONE_MS = 2500;

type ProgressState = {
  /** 저장소 경로 → 마지막으로 받은 진행. */
  byRepo: Record<string, OpProgress>;
  receive: (progress: OpProgress) => void;
  clear: (repo: string) => void;
};

const timers = new Map<string, ReturnType<typeof setTimeout>>();

/** 오래 걸리는 작업의 진행(D-080). 화면은 useProgress로 읽는다. */
export const useProgressStore = create<ProgressState>((set) => ({
  byRepo: {},
  receive: (progress) => {
    const old = timers.get(progress.repo);
    if (old !== undefined) clearTimeout(old);
    timers.set(
      progress.repo,
      setTimeout(() => useProgressStore.getState().clear(progress.repo), PROGRESS_GONE_MS),
    );
    set((s) => ({ byRepo: { ...s.byRepo, [progress.repo]: progress } }));
  },
  clear: (repo) => {
    const old = timers.get(repo);
    if (old !== undefined) clearTimeout(old);
    timers.delete(repo);
    set((s) => {
      if (!(repo in s.byRepo)) return s;
      const byRepo = { ...s.byRepo };
      delete byRepo[repo];
      return { byRepo };
    });
  },
}));
