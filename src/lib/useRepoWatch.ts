// 작업 공간의 파일 변경 감시(D-025, 계획 001의 21단계).
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api } from "./api";
import { onRepoChanged, onWatchStatus } from "./events";
import { queryKeys } from "./queries";

/** 변경 알림을 묶는 간격. 빌드처럼 파일이 쏟아질 때 화면을 너무 자주 다시 읽지 않는다. */
export const WATCH_THROTTLE_MS = 1000;

/**
 * 저장소를 감시하고 바뀐 쪽만 다시 읽는다. 작업 폴더가 바뀌면 변경 사항·비교, `.git`이 바뀌면
 * 변경 사항·버전 기록. 감시를 쓸 수 없으면 `active`가 false이고 창 포커스 때 다시 읽는다.
 */
export function useRepoWatch(repo: string): { active: boolean | null; reload: () => void } {
  const queryClient = useQueryClient();
  const [active, setActive] = useState<boolean | null>(null);

  const reload = () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.status(repo) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.log(repo) });
    void queryClient.invalidateQueries({ queryKey: ["diff", repo] });
  };

  useEffect(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout> | null = null;
    let pending = { worktree: false, git: false };

    const flush = () => {
      if (pending.worktree || pending.git) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.status(repo) });
      }
      if (pending.worktree) void queryClient.invalidateQueries({ queryKey: ["diff", repo] });
      if (pending.git) void queryClient.invalidateQueries({ queryKey: queryKeys.log(repo) });
      pending = { worktree: false, git: false };
    };

    const unlisten = [
      onRepoChanged((e) => {
        pending = { worktree: pending.worktree || e.worktree, git: pending.git || e.git };
        if (timer) return;
        flush();
        timer = setTimeout(function cool() {
          if (pending.worktree || pending.git) {
            flush();
            timer = setTimeout(cool, WATCH_THROTTLE_MS);
          } else {
            timer = null;
          }
        }, WATCH_THROTTLE_MS);
      }),
      onWatchStatus((e) => {
        if (!disposed) setActive(e.active);
      }),
    ];

    api.watch(repo).catch(() => {
      if (!disposed) setActive(false);
    });

    return () => {
      disposed = true;
      if (timer) clearTimeout(timer);
      for (const p of unlisten) void p.then((stop) => stop());
      void api.unwatch().catch(() => {});
    };
  }, [repo, queryClient]);

  // 감시가 켜져 있으면 창 포커스 새로 고침은 끈다(이중 갱신 방지). 꺼져 있으면 켠다.
  useEffect(() => {
    queryClient.setDefaultOptions({
      ...queryClient.getDefaultOptions(),
      queries: { ...queryClient.getDefaultOptions().queries, refetchOnWindowFocus: active !== true },
    });
  }, [active, queryClient]);

  return { active, reload };
}
