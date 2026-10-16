import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, type MergeMethodView } from "../../lib/api";
import { queryKeys, usePrRepo } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Choice } from "../../ui/Choice";
import { ConfirmDialog } from "../../ui/ConfirmDialog";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import { useRemoteBranchStore } from "../remote-branch/remoteBranchStore";
import { branchLine } from "./model";
import { useMergeStore } from "./mergeStore";
import styles from "./PrRow.module.css";

/**
 * 검토 요청 합치기 확인 창(D-179, design.md "검토 요청 합치기 확인 대화상자"). 저장소가 허용한 방법만 보이고,
 * 확인 토큰은 번호·방법·본 head에 묶인다. 합친 뒤 원격 확인을 한 번 하고, 원격 브랜치가 남아 있으면 토스트에
 * "브랜치 지우기"(D-130의 확인 창)를 둔다. 브랜치 지우기를 합치기 안에 넣지 않는다.
 */
export function MergeDialog({ repo }: { repo: string }) {
  const target = useMergeStore((s) => s.target);
  if (!target) return null;
  return <Dialog repo={repo} key={`${target.row.number}-${target.headOid}`} />;
}

function Dialog({ repo }: { repo: string }) {
  const m = useMessages();
  const client = useQueryClient();
  const target = useMergeStore((s) => s.target);
  const close = useMergeStore((s) => s.close);
  const show = useToastStore((s) => s.show);
  const openRemoteBranch = useRemoteBranchStore((s) => s.open);
  const methods = usePrRepo(repo, true).data?.mergeMethods ?? [];
  const [picked, setPicked] = useState<MergeMethodView | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<unknown>(null);
  if (!target) return null;
  const method = picked ?? methods[0] ?? null;
  const { row, checks, mergeable } = target;
  const label = (v: MergeMethodView) => (v === "merge" ? m.pr.methodMerge : m.pr.methodSquash);

  const confirm = async () => {
    if (!method) return;
    setPending(true);
    setError(null);
    const input = { id: "pr-merge", input: { number: row.number, method, expectHead: target.headOid } } as const;
    try {
      const preview = await api.previewAction(repo, input);
      await api.executeAction(repo, input, preview.token);
    } catch (e) {
      setError(e);
      setPending(false);
      return;
    }
    close();
    for (const key of [queryKeys.prDetail(repo, row.number), ["github", "prList", repo]]) {
      void client.invalidateQueries({ queryKey: key });
    }
    // 원격 확인을 한 번 하고(실패해도 둔다) 원격 브랜치가 남았으면 지우기를 제안한다.
    await api.executeAction(repo, { id: "fetch", input: { background: false } }).catch(() => undefined);
    await client.invalidateQueries({ queryKey: queryKeys.branches(repo) });
    const branches = await api.branches(repo).catch(() => null);
    const left = row.headOwner
      ? undefined
      : branches?.remote.find((b) => b.remote !== null && b.name === `${b.remote}/${row.head}`);
    show(
      m.pr.mergedDone,
      left?.remote
        ? {
            label: m.pr.deleteBranch,
            onClick: () => openRemoteBranch({ remote: left.remote as string, branch: row.head, mode: "delete" }),
          }
        : undefined,
    );
  };

  return (
    <ConfirmDialog
      open
      size="wide"
      title={<Term id="pr-merge" focusable={false} />}
      subtitle={`#${row.number} · ${branchLine(row)}`}
      dangerLabel={m.pr.merge}
      closeLabel={m.pr.close}
      pending={pending}
      dangerDisabled={method === null}
      onConfirm={() => void confirm()}
      onClose={() => !pending && close()}
    >
      {methods.length > 1 && method && (
        <div className={styles.methodRow}>
          <span>{m.pr.mergeMethod}</span>
          <Choice
            label={m.pr.mergeMethod}
            value={method}
            options={methods.map((v) => ({ value: v, label: label(v) }))}
            onChange={setPicked}
          />
        </div>
      )}
      {methods.length === 1 && method && <p className={styles.line}>{label(method)}</p>}
      {checks && (checks.failed > 0 || checks.pending > 0) && (
        <p className={`${styles.line} ${styles.conflicts}`}>{m.pr.checksWarn}</p>
      )}
      {(mergeable === "behind" || mergeable === "blocked") && <p className={styles.muted}>{m.pr.rulesWarn}</p>}
      {error !== null && <ErrorNotice error={error} />}
    </ConfirmDialog>
  );
}
