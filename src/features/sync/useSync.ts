import { useIsMutating, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiError, api } from "../../lib/api";
import { invalidateLog } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { useToastStore } from "../../ui/toastStore";
import { useSyncStore } from "./syncStore";

/**
 * 동기화 실행(D-045). 버튼이 여럿이어도(상단 바, 다 저장한 화면, 멈춤 안내 줄) 같은 작업을 본다.
 * 결과 처리는 useMutation 옵션에 두어, 누른 버튼이 사라져도 이어진다.
 */
export function useSync(repo: string): { run: () => void; running: boolean } {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const set = useSyncStore((s) => s.set);
  const mutationKey = ["sync", repo];

  const sync = useMutation({
    mutationKey,
    mutationFn: () => api.executeAction(repo, { id: "sync", input: { locale } }),
    onMutate: () => set({ paused: false, error: null }),
    onSuccess: (result) => {
      if (result.kind !== "synced") return;
      // 겹쳐서 합치기를 시작했으면 충돌 해결 화면이 이어받는다(D-069). 겹친 글자는 남기지 않는다.
      set({ overlap: result.mergeStarted ? [] : result.overlap, fetchFailedAt: null });
      if (result.overlap.length === 0) show(m.synced(result.sent, result.received));
    },
    onError: (e) => {
      // 사람이 취소한 것은 오류가 아니다(D-080).
      if (e instanceof ApiError && e.code === "cancelled") show(m.errors.cancelled);
      else set({ error: e });
    },
    onSettled: () => {
      invalidateLog(queryClient, repo);
      for (const key of [["status", repo], ["branches", repo], ["mergeState", repo], ["homeMap", repo]]) {
        void queryClient.invalidateQueries({ queryKey: key });
      }
    },
  });
  const running = useIsMutating({ mutationKey }) > 0;
  return {
    run: () => {
      if (!running) sync.mutate();
    },
    running,
  };
}
