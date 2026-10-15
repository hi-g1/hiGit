import { useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { ApiError, api } from "../../lib/api";
import { queryKeys, usePendingWork } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import { ClearLockButton } from "./ClearLockButton";
import styles from "./PendingNotice.module.css";
import { pendingKey, usePendingStore, type PendingKind } from "./pendingStore";

type Props = {
  repo: string;
  /** 앱이 다룰 수 없는 충돌(다른 도구의 작업)이 있다. 이름 붙은 작업이 없으면 일반 문구로 알린다. */
  otherConflicts: boolean;
};

/**
 * 끝나지 않은 작업 알림 줄(D-082). 줄마다 짧은 문장과 단추 하나, 닫기. 닫으면 이번 실행에서는
 * 다시 보이지 않는다. 합치는 중이면 충돌 해결 화면이 대신 알리므로 부르지 않는다.
 */
export function PendingNotice({ repo, otherConflicts }: Props) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const pending = usePendingWork(repo).data;
  const dismissed = usePendingStore((s) => s.dismissed);
  const dismiss = usePendingStore((s) => s.dismiss);
  const [busy, setBusy] = useState(false);

  const recover = async () => {
    setBusy(true);
    try {
      const result = await api.executeAction(repo, { id: "recover-parked" });
      if (result.kind === "parkedRecovered") show(m.parkedRecovered(result.count));
    } catch (e) {
      show(m.errors[e instanceof ApiError ? e.code : "internal"]);
    } finally {
      setBusy(false);
      for (const key of [queryKeys.pendingWork(repo), queryKeys.stashes(repo), queryKeys.branches(repo)]) {
        void queryClient.invalidateQueries({ queryKey: key });
      }
    }
  };

  const lines: { kind: PendingKind; text: ReactNode; action?: ReactNode }[] = [];
  if (pending && pending.staleLocks.length > 0) {
    lines.push({ kind: "locks", text: m.staleLocks(pending.staleLocks.length), action: <ClearLockButton repo={repo} /> });
  }
  if (pending && pending.leftoverParked > 0) {
    lines.push({
      kind: "parked",
      text: m.leftoverParked(pending.leftoverParked),
      action: (
        <Button variant="secondary" disabled={busy} onClick={() => void recover()}>
          {m.recoverParked}
        </Button>
      ),
    });
  }
  const other = pending?.otherOperation;
  if (other === "rebase" || other === "revert") {
    lines.push({
      kind: "other",
      text: (
        <>
          <Term id={other} /> · {m.byOtherTool}
        </>
      ),
    });
  } else if (other === "cherryPick" || otherConflicts) {
    lines.push({ kind: "other", text: m.otherOperation });
  }

  const shown = lines.filter((l) => !dismissed.has(pendingKey(repo, l.kind)));
  if (shown.length === 0) return null;
  return (
    <div className={styles.lines}>
      {shown.map((line) => (
        <div key={line.kind} className={styles.line} role="status">
          <span className={styles.text}>{line.text}</span>
          {line.action}
          <Button variant="text" onClick={() => dismiss(repo, line.kind)}>
            {m.close}
          </Button>
        </div>
      ))}
    </div>
  );
}
