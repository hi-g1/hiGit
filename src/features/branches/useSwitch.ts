import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, type ActionResult, type DirtyChoice } from "../../lib/api";
import { invalidateLog } from "../../lib/queries";
import { useMessages, type Messages } from "../../messages";
import { useToastStore } from "../../ui/toastStore";
import { useTipStore } from "../tips/tipStore";

export type SwitchTarget = { branch: string; remote: string | null };

/** 갈아탄 뒤 토스트 글(D-195): 가져왔는지·두고 왔는지, 도착한 브랜치에서 두고 간 파일을 꺼냈는지. */
export function switchedText(m: Messages, target: SwitchTarget, result: ActionResult): string {
  if (result.kind !== "switched") return m.switchedTo(target.branch);
  const text = result.carried
    ? m.switchedCarried(result.files, result.branch)
    : result.parkedOn !== null && result.files > 0
      ? m.switchedLeft(result.files, result.parkedOn)
      : m.switchedTo(result.branch);
  return result.unparked ? `${text} · ${m.unparkedHere}` : text;
}

/** 브랜치 갈아타기 실행. 목록과 대화상자가 같이 쓴다. 성공하면 true. */
export function useSwitch(repo: string, onDone: (text: string) => void) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const showError = useToastStore((s) => s.showError);
  const [pending, setPending] = useState<DirtyChoice | "plain" | null>(null);
  const run = async (target: SwitchTarget, dirty?: DirtyChoice): Promise<boolean> => {
    setPending(dirty ?? "plain");
    try {
      const result = await api.executeAction(repo, {
        id: "switch",
        input: dirty === undefined ? { ...target } : { ...target, dirty },
      });
      onDone(switchedText(m, target, result));
      useTipStore.getState().trigger("firstBranch");
      return true;
    } catch (e) {
      showError(e);
      return false;
    } finally {
      setPending(null);
      invalidateLog(queryClient, repo);
      for (const key of ["status", "branches", "homeMap", "mergeHint", "switchPlan"]) {
        void queryClient.invalidateQueries({ queryKey: [key, repo] });
      }
    }
  };
  return { run, pending };
}
