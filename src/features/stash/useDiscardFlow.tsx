import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, type ChangedFile } from "../../lib/api";
import { useMessages } from "../../messages";
import { useToastStore } from "../../ui/toastStore";

/**
 * 파일 하나의 고친 것 버리기(D-074, D-196). 확인 창 없이 버리고 토스트의 되돌리기(Ctrl+Z)로 되살린다. 버린
 * 내용은 백업 ref에도 남아 작업 내역에서 꺼낼 수 있다. 비교 칸 아래 줄과 파일 줄 오른쪽 메뉴가 같이 쓴다.
 */
export function useDiscardFlow(
  repo: string,
  onDone: (text: string) => void,
): { run: (file: ChangedFile) => void; pending: boolean } {
  const m = useMessages();
  const queryClient = useQueryClient();
  const showError = useToastStore((s) => s.showError);
  const [pending, setPending] = useState(false);
  const pathsOf = (file: ChangedFile) => (file.origPath ? [file.path, file.origPath] : [file.path]);

  const run = (file: ChangedFile) => {
    setPending(true);
    api
      .executeAction(repo, { id: "discard", input: { paths: pathsOf(file) } })
      .then(
        () => onDone(m.discarded),
        (e: unknown) => showError(e),
      )
      .finally(() => {
        setPending(false);
        for (const key of ["status", "diff", "stashes", "recovery", "homeMap"]) {
          void queryClient.invalidateQueries({ queryKey: [key, repo] });
        }
      });
  };
  return { run, pending };
}
