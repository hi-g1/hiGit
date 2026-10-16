import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ApiError, api, type ActionInput } from "../../lib/api";
import { queryKeys, useBranchNameCheck, useGitHubStatus, usePrForBranch, useRemoteBranchPlan } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ConfirmDialog } from "../../ui/ConfirmDialog";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Loading } from "../../ui/Loading";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import { stateLabel } from "../pr/model";
import { CANCEL_AFTER_SECS, useProgress } from "../progress/useProgress";
import styles from "../force-push/ForcePushDialog.module.css";
import { NameConfirmField } from "./NameConfirmField";
import { useRemoteBranchStore, type RemoteBranchTarget } from "./remoteBranchStore";
import { isBlockedEnter } from "../../lib/keys";

/**
 * 원격 브랜치 지우기·이름 바꾸기 확인 창(D-130, design.md "확인 대화상자"). 계획 조회가 준 `tip`만 입력에
 * 싣는다. 포커스는 "닫기"이고 이름 칸의 Enter는 위험 버튼을 누르지 않는다.
 */
export function RemoteBranchDialog({ repo }: { repo: string }) {
  const target = useRemoteBranchStore((s) => s.target);
  if (target === null) return null;
  // 대상이 바뀌면 입력과 오류를 새로 시작한다.
  return <Dialog key={`${target.mode}:${target.remote}/${target.branch}`} repo={repo} target={target} />;
}

function Dialog({ repo, target }: { repo: string; target: RemoteBranchTarget }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const close = useRemoteBranchStore((s) => s.close);
  const plan = useRemoteBranchPlan(repo, target.remote, target.branch, true);
  const github = useGitHubStatus().data;
  const prs = usePrForBranch(repo, target.branch, target.mode === "delete" && github?.connected === true);
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const tick = useProgress(repo);
  const [to, setTo] = useState(target.branch);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [typed, setTyped] = useState("");
  const name = to.trim();
  const check = useBranchNameCheck(repo, target.mode === "rename" && name !== target.branch ? name : "");

  const data = plan.data;
  const blocked = data?.blocked ?? null;
  const nameOk = target.mode === "delete" || (name !== "" && name !== target.branch && check.data?.ok === true);
  // 이름만 기본 브랜치 같은 원격 브랜치는 이름을 똑같이 입력해야 한다(D-183).
  const needsName = data?.needsNameConfirm === true;
  const confirmed = !needsName || typed === target.branch;
  const confirm = needsName ? { confirmBranch: typed } : {};
  const input: ActionInput | null =
    data?.tip && !blocked && nameOk && confirmed
      ? target.mode === "delete"
        ? { id: "remote-branch-delete", input: { remote: target.remote, branch: target.branch, expectTip: data.tip, ...confirm } }
        : {
            id: "remote-branch-rename",
            input: { remote: target.remote, from: target.branch, to: name, expectTip: data.tip, ...confirm },
          }
      : null;
  const openPr = prs.data?.items.find((r) => r.head === target.branch && r.state === "open") ?? null;

  const refresh = () => {
    for (const key of [queryKeys.branches(repo), queryKeys.status(repo), queryKeys.homeMap(repo)]) {
      void queryClient.invalidateQueries({ queryKey: key });
    }
    void queryClient.invalidateQueries({ queryKey: ["remoteBranchPlan", repo] });
  };
  const finish = () => {
    setError(null);
    close();
  };
  const run = async () => {
    if (!input) return;
    setPending(true);
    setError(null);
    try {
      const preview = await api.previewAction(repo, input);
      await api.executeAction(repo, input, preview.token);
      finish();
      show(target.mode === "delete" ? m.remoteBranch.deleted : m.remoteBranch.renamed);
    } catch (e) {
      if (e instanceof ApiError && e.code === "cancelled") show(m.errors.cancelled);
      else setError(e);
    } finally {
      setPending(false);
      refresh();
    }
  };
  const checkRemote = async () => {
    finish();
    try {
      await api.executeAction(repo, { id: "fetch", input: { background: false } });
    } catch (e) {
      showError(e);
    } finally {
      refresh();
    }
  };
  const progress = pending && tick && tick.action === input?.id ? tick : null;
  const moved = error instanceof ApiError && error.code === "remoteMoved";
  const blockedText =
    blocked === "protected" ? m.errors.remoteBranchProtected : blocked === "noTrackingRef" ? m.remoteBranch.noTrackingRef : blocked ? m.errors.noRemote : null;

  return (
    <ConfirmDialog
      open
      title={<Term id={target.mode === "delete" ? "remote-branch-delete" : "remote-branch-rename"} focusable={false} />}
      subtitle={`${target.remote}/${target.branch}`}
      dangerLabel={target.mode === "delete" ? m.remoteBranch.deleteConfirm : m.remoteBranch.renameConfirm}
      closeLabel={m.close}
      pending={pending}
      dangerDisabled={input === null}
      onConfirm={() => void run()}
      onClose={() => !pending && finish()}
    >
      <Loading pending={plan.isPending} />
      {plan.error && <ErrorNotice error={plan.error} />}
      {blockedText && <p className={styles.blocked}>{blockedText}</p>}
      {target.mode === "rename" && (
        <input
          className={styles.field}
          value={to}
          aria-label={m.remoteBranch.newName}
          onChange={(e) => setTo(e.target.value)}
          // 이름 칸의 Enter는 아무것도 하지 않는다(되돌릴 수 없는 일은 단추로만).
          onKeyDown={(e) => {
            if (isBlockedEnter(e)) e.preventDefault();
          }}
        />
      )}
      {target.mode === "rename" && check.data && !check.data.ok && (
        <p className={styles.blocked}>{check.data.problem === "exists" ? m.sameBranchName : m.badBranchName}</p>
      )}
      {data && !blocked && (
        <ul className={styles.counts}>
          <li>{m.remoteBranch.onlyHere(data.onlyHere)}</li>
          {data.byOthers > 0 && <li className={styles.others}>{m.remoteBranch.byOthers(data.byOthers)}</li>}
          {data.isUpstreamOfCurrent && <li>{m.remoteBranch.upstreamOfCurrent}</li>}
          {openPr && <li>{m.pr.slotExisting(openPr.number, stateLabel(openPr, m) ?? m.pr.draftLabel)}</li>}
        </ul>
      )}
      {data && !blocked && (
        <NameConfirmField
          repo={repo}
          remote={target.remote}
          branch={target.branch}
          needsName={needsName}
          value={typed}
          onChange={setTyped}
        />
      )}
      {pending && (
        <p className={styles.running} role="status">
          {m.forcePush.running(progress?.elapsedSecs ?? null)}
          {progress && progress.elapsedSecs >= CANCEL_AFTER_SECS && (
            <Button variant="text" onClick={() => void api.cancelAction(repo).catch(showError)}>
              {m.cancel}
            </Button>
          )}
        </p>
      )}
      {error !== null && (
        <ErrorNotice error={error}>
          {moved && (
            <Button variant="secondary" onClick={() => void checkRemote()}>
              <Term id="fetch" focusable={false} />
            </Button>
          )}
        </ErrorNotice>
      )}
    </ConfirmDialog>
  );
}
