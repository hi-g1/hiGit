// 실행 인자(`higit <폴더>`, 우클릭 메뉴)로 받은 폴더를 연다(D-213). 사람이 고른 폴더와 같은 길
// (`project_open(path, "picked")`)이라 신뢰 확인을 건너뛰지 않는다.
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { api, type ProjectOpenResult } from "../../lib/api";
import { onOpenFolder } from "../../lib/events";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { useToastStore } from "../../ui/toastStore";
import { useOpenProject } from "./useOpenProject";

/** 도는 작업이나 열린 대화상자가 있으면 이만큼 뒤에 다시 본다. */
export const LAUNCH_RETRY_MS = 500;

/** 받은 폴더 열기. 저장소가 아니면 시작 화면에서 버전 관리 시작을 묻고, 없으면 알린다. */
export function useOpenLaunched(): (path: string) => Promise<ProjectOpenResult> {
  const project = useOpenProject();
  const m = useMessages();
  const showToast = useToastStore((s) => s.show);
  return async (path) => {
    const result = await project.open(path, "picked");
    const store = useRepoStore.getState();
    if (result.kind === "notARepo") {
      store.closeRepo();
      store.setPendingInit(result.folder);
    }
    if (result.kind === "missing") {
      if (store.repo) showToast(m.launchMissing);
      else store.setStartNotice({ kind: "launchMissing", name: "" });
    }
    return result;
  };
}

/**
 * 이미 켜진 앱에 두 번째 실행이 폴더를 넘기면(이벤트 `open-folder`) 그 폴더를 연다. 도는 작업이나 열린 확인 창이
 * 끝난 뒤에 열고, 그사이 여러 번 넘어오면 마지막 것만 연다(경로는 Rust가 들고 있다가 한 번만 준다).
 */
export function useLaunchRequests(enabled: boolean): void {
  const queryClient = useQueryClient();
  const openLaunched = useOpenLaunched();
  const latest = useRef(openLaunched);
  useEffect(() => {
    latest.current = openLaunched;
  });

  useEffect(() => {
    if (!enabled) return;
    let stopped = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const busy = () => queryClient.isMutating() > 0 || document.querySelector('[role="dialog"]') !== null;
    const attempt = async () => {
      timer = undefined;
      if (stopped) return;
      if (busy()) {
        timer = setTimeout(() => void attempt(), LAUNCH_RETRY_MS);
        return;
      }
      try {
        const path = await api.takeLaunchFolder();
        if (path !== null && !stopped) await latest.current(path);
      } catch {
        // 열기 오류는 열기 훅이 보여준다. 여기서는 다음 요청을 기다린다.
      }
    };
    const unlisten = onOpenFolder(() => {
      if (timer === undefined) void attempt();
    });
    return () => {
      stopped = true;
      if (timer !== undefined) clearTimeout(timer);
      void unlisten.then((f) => f());
    };
  }, [enabled, queryClient]);
}
