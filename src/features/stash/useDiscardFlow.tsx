import { useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { api, type ChangedFile } from "../../lib/api";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { ConfirmDialog } from "../../ui/ConfirmDialog";
import { useToastStore } from "../../ui/toastStore";

/**
 * 파일 하나의 고친 것 버리기(D-074, D-076). 되돌릴 수 없어 미리보기 토큰을 받고 파일 이름을 보이는 확인을
 * 거친다(D-043). 비교 칸 아래 줄과 파일 줄 오른쪽 메뉴가 같이 쓴다.
 */
export function useDiscardFlow(repo: string): { ask: (file: ChangedFile) => void; dialog: ReactNode; pending: boolean } {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const setView = useRepoStore((s) => s.setView);
  const [target, setTarget] = useState<{ file: ChangedFile; token: string } | null>(null);
  const [pending, setPending] = useState(false);
  const pathsOf = (file: ChangedFile) => (file.origPath ? [file.path, file.origPath] : [file.path]);
  const refresh = () => {
    for (const key of ["status", "diff", "stashes", "recovery", "homeMap"]) {
      void queryClient.invalidateQueries({ queryKey: [key, repo] });
    }
  };

  const ask = (file: ChangedFile) => {
    void api.previewAction(repo, { id: "discard", input: { paths: pathsOf(file) } }).then(
      (preview) => setTarget({ file, token: preview.token }),
      (e: unknown) => showError(e),
    );
  };
  const discard = async () => {
    if (!target) return;
    setPending(true);
    try {
      await api.executeAction(repo, { id: "discard", input: { paths: pathsOf(target.file) } }, target.token);
      setTarget(null);
      // 되돌리기가 아니라 작업 내역에서 꺼낸다(백업이 거기 있다).
      show(m.discarded, { label: m.openActivity, onClick: () => setView("activity") });
    } catch (e) {
      setTarget(null);
      showError(e);
    } finally {
      setPending(false);
      refresh();
    }
  };

  const dialog = (
    <ConfirmDialog
      open={target !== null}
      title={target ? m.discardTitle(target.file.path.split("/").pop() ?? target.file.path) : ""}
      subtitle={target ? pathsOf(target.file).join(" · ") : undefined}
      dangerLabel={m.discardConfirm}
      closeLabel={m.close}
      pending={pending}
      onConfirm={() => void discard()}
      onClose={() => setTarget(null)}
    />
  );
  return { ask, dialog, pending };
}
