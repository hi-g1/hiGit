import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ApiError, api } from "../../lib/api";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { useToastStore } from "../../ui/toastStore";

/**
 * 합치기(`merge`)·"main에 합치기"(`merge-into`) 실행(D-048). 합치면 결과 토스트와 되돌리기,
 * 겹치면 합치는 중 상태를 다시 읽어 충돌 해결 화면으로 간다(D-069).
 */
export function useMergeAction(repo: string, onDone: (text: string) => void) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const [pending, setPending] = useState(false);

  const run = async (kind: "merge" | "merge-into", branch: string) => {
    setPending(true);
    try {
      const result =
        kind === "merge"
          ? await api.executeAction(repo, { id: "merge", input: { branch, locale } })
          : await api.executeAction(repo, { id: "merge-into", input: { branch } });
      if (result.kind === "merged") onDone(m.merged);
    } catch (e) {
      show(m.errors[e instanceof ApiError ? e.code : "internal"]);
    } finally {
      setPending(false);
      for (const key of ["status", "log", "branches", "homeMap", "mergeHint", "mergeState"]) {
        void queryClient.invalidateQueries({ queryKey: [key, repo] });
      }
    }
  };
  return { run, pending };
}
