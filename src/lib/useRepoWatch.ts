// 작업 공간의 파일 변경 감시(D-025, 계획 001의 21단계).
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api } from "./api";
import { onRepoChanged, onWatchStatus } from "./events";
import { invalidateLog, queryKeys } from "./queries";

/** 변경 알림을 묶는 간격. 빌드처럼 파일이 쏟아질 때 화면을 너무 자주 다시 읽지 않는다. */
export const WATCH_THROTTLE_MS = 1000;

/**
 * watch/unwatch를 부른 순서대로 실행한다. 백엔드 명령은 따로 도는 스레드라 순서가 보장되지
 * 않아, 저장소를 빨리 바꾸면 늦게 도착한 unwatch가 새 감시를 끌 수 있다.
 */
let queue: Promise<unknown> = Promise.resolve();
function inOrder<T>(task: () => Promise<T>): Promise<T> {
  const next = queue.then(task, task);
  queue = next.catch(() => {});
  return next;
}

/**
 * 저장소를 감시하고 바뀐 쪽만 다시 읽는다. 작업 폴더가 바뀌면 변경 사항·비교, `.git`이 바뀌면
 * 변경 사항·버전 기록. 감시를 쓸 수 없으면 `active`가 false이고 창 포커스 때 다시 읽는다.
 */
export function useRepoWatch(repo: string): { active: boolean | null; reload: () => void } {
  const queryClient = useQueryClient();
  const [active, setActive] = useState<boolean | null>(null);

  const reload = () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.status(repo) });
    invalidateLog(queryClient, repo);
    void queryClient.invalidateQueries({ queryKey: ["diff", repo] });
    void queryClient.invalidateQueries({ queryKey: queryKeys.homeMap(repo) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.branches(repo) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.mergeState(repo) });
    void queryClient.invalidateQueries({ queryKey: ["conflictFile", repo] });
    void queryClient.invalidateQueries({ queryKey: queryKeys.mergeHint(repo) });
    void queryClient.invalidateQueries({ queryKey: ["rewritePlan", repo] });
    void queryClient.invalidateQueries({ queryKey: queryKeys.forcePushPlan(repo) });
    void queryClient.invalidateQueries({ queryKey: ["remoteBranchPlan", repo] });
  };

  useEffect(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout> | null = null;
    let pending = { worktree: false, git: false };

    const flush = () => {
      if (pending.worktree || pending.git) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.status(repo) });
      }
      // .git만 바뀌어도(예: 다른 곳에서 버전 저장) 마지막 버전이 달라지므로 비교를 다시 읽는다.
      if (pending.worktree || pending.git) {
        void queryClient.invalidateQueries({ queryKey: ["diff", repo] });
      }
      if (pending.git) {
        invalidateLog(queryClient, repo);
        void queryClient.invalidateQueries({ queryKey: queryKeys.branches(repo) });
        void queryClient.invalidateQueries({ queryKey: ["rewritePlan", repo] });
        void queryClient.invalidateQueries({ queryKey: queryKeys.forcePushPlan(repo) });
        void queryClient.invalidateQueries({ queryKey: ["remoteBranchPlan", repo] });
      }
      // 노선도는 버전(.git)과 고친 파일 수(작업 폴더) 둘 다에 따라 바뀐다.
      if (pending.worktree || pending.git) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.homeMap(repo) });
        // 합치는 중 상태(MERGE_HEAD, index의 충돌 항목)도 둘 다에 따라 바뀐다.
        void queryClient.invalidateQueries({ queryKey: queryKeys.mergeState(repo) });
        void queryClient.invalidateQueries({ queryKey: ["conflictFile", repo] });
      }
      if (pending.git) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.mergeHint(repo) });
        void queryClient.invalidateQueries({ queryKey: queryKeys.recovery(repo) });
        void queryClient.invalidateQueries({ queryKey: queryKeys.stashes(repo) });
        void queryClient.invalidateQueries({ queryKey: queryKeys.pendingWork(repo) });
      }
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

    inOrder(() => api.watch(repo)).catch(() => {
      if (!disposed) setActive(false);
    });

    return () => {
      disposed = true;
      if (timer) clearTimeout(timer);
      for (const p of unlisten) void p.then((stop) => stop());
      void inOrder(() => api.unwatch()).catch(() => {});
    };
  }, [repo, queryClient]);

  // 감시가 켜져 있으면 창 포커스 새로 고침은 끈다(이중 갱신 방지). 꺼져 있거나 닫히면 켠다.
  useEffect(() => {
    const setFocusRefetch = (on: boolean) =>
      queryClient.setDefaultOptions({
        ...queryClient.getDefaultOptions(),
        queries: { ...queryClient.getDefaultOptions().queries, refetchOnWindowFocus: on },
      });
    setFocusRefetch(active !== true);
    return () => setFocusRefetch(true);
  }, [active, queryClient]);

  return { active, reload };
}
