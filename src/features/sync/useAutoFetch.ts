import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { ApiError, api, type ErrorCode } from "../../lib/api";
import { useSyncStore } from "./syncStore";

/** 원격 확인 간격(D-055). */
export const FETCH_INTERVAL_MS = 5 * 60_000;
/** 창으로 돌아왔을 때 이보다 오래됐으면 다시 확인한다. */
export const FOCUS_MIN_GAP_MS = 60_000;

const AUTH_FAILURES: ErrorCode[] = [
  "sshAuthFailed",
  "httpAuthFailed",
  "sshHostUnverified",
  "httpForbidden",
  "remoteNotFound",
  "githubExpired",
];

/**
 * 저절로 하는 원격 확인(D-045, D-055). 열 때 한 번, 창이 보이는 동안 5분마다, 창으로 돌아왔을 때
 * 1분이 지났으면. 질문 창 없이(백그라운드 모드) 하고, 실패는 동기화 버튼 옆 작은 점으로만 알린다.
 * 인증 실패 뒤에는 사용자가 동기화를 누를 때까지 멈춘다.
 */
export function useAutoFetch(repo: string, enabled: boolean) {
  const queryClient = useQueryClient();
  const last = useRef(0);
  const running = useRef(false);

  useEffect(() => {
    if (!enabled) return;
    const run = async () => {
      const { paused, set } = useSyncStore.getState();
      if (paused || running.current || document.visibilityState === "hidden") return;
      running.current = true;
      last.current = Date.now();
      try {
        await api.executeAction(repo, { id: "fetch", input: { background: true } });
        set({ fetchFailedAt: null });
        for (const key of [["status", repo], ["branches", repo]]) {
          void queryClient.invalidateQueries({ queryKey: key });
        }
      } catch (e) {
        const code = e instanceof ApiError ? e.code : "internal";
        // 다른 작업 중이면 조용히 건너뛴다.
        if (code !== "busy") set({ fetchFailedAt: Date.now(), paused: AUTH_FAILURES.includes(code) });
      } finally {
        running.current = false;
      }
    };
    void run();
    const timer = window.setInterval(() => void run(), FETCH_INTERVAL_MS);
    const onFocus = () => {
      if (Date.now() - last.current >= FOCUS_MIN_GAP_MS) void run();
    };
    window.addEventListener("focus", onFocus);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  }, [repo, enabled, queryClient]);
}
