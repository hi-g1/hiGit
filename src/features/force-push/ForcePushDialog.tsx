import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ApiError, api, type ActionInput } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { invalidateLog, queryKeys, useAppSettings, useForcePushPlan } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ConfirmDialog } from "../../ui/ConfirmDialog";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Loading } from "../../ui/Loading";
import { Term } from "../../ui/Term";
import { Truncate } from "../../ui/Truncate";
import { useToastStore } from "../../ui/toastStore";
import { CANCEL_AFTER_SECS, useProgress } from "../progress/useProgress";
import { NameConfirmField } from "../remote-branch/NameConfirmField";
import styles from "./ForcePushDialog.module.css";
import { useForcePushStore } from "./forcePushStore";

/**
 * 덮어쓰며 올리기 확인 창(D-128, design.md "덮어쓰며 올리기 확인 대화상자"). core의 계획 조회가 준 값만 보이고
 * 그 값을 그대로 입력에 담는다(원격·브랜치·해시를 화면이 정하지 않는다). 미리보기 토큰을 받은 뒤 실행한다.
 */
export function ForcePushDialog({ repo }: { repo: string }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const open = useForcePushStore((s) => s.open);
  const close = useForcePushStore((s) => s.close);
  const allowed = (useAppSettings().data ?? DEFAULT_SETTINGS).allowPublishedRewrite;
  const plan = useForcePushPlan(repo, open && allowed);
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const tick = useProgress(repo);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [typed, setTyped] = useState("");
  if (!open) return null;

  const data = plan.data;
  const blocked = allowed ? (data?.blocked ?? null) : "disabled";
  // 원격 기본 브랜치로 보이면(로컬 규칙, 또는 서버가 그렇다고 해 거절됨) 이름을 똑같이 입력해야 한다(D-183).
  const needsName =
    data?.needsNameConfirm === true || (error instanceof ApiError && error.code === "nameConfirmMismatch");
  const nameOk = !needsName || typed === data?.remoteBranch;
  const input: ActionInput | null =
    data && !blocked && nameOk && data.remote && data.remoteBranch && data.expectRemote && data.expectHead
      ? {
          id: "force-push",
          input: {
            remote: data.remote,
            branch: data.remoteBranch,
            expectRemote: data.expectRemote,
            expectHead: data.expectHead,
            ...(needsName ? { confirmBranch: typed } : {}),
          },
        }
      : null;
  const refresh = () => {
    invalidateLog(queryClient, repo);
    for (const key of [queryKeys.status(repo), queryKeys.branches(repo), queryKeys.homeMap(repo), queryKeys.forcePushPlan(repo)]) {
      void queryClient.invalidateQueries({ queryKey: key });
    }
    void queryClient.invalidateQueries({ queryKey: ["recovery", repo] });
  };
  const finish = () => {
    setError(null);
    setTyped("");
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
      show(m.forcePush.done);
    } catch (e) {
      // 사람이 취소한 것은 오류가 아니다(D-080).
      if (e instanceof ApiError && e.code === "cancelled") show(m.errors.cancelled);
      else setError(e);
    } finally {
      setPending(false);
      refresh();
    }
  };
  // 그사이 원격이 바뀌었으면 원격을 확인한 뒤 다시 본다.
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
  const cancel = () => void api.cancelAction(repo).catch(showError);
  const progress = pending && tick?.action === "force-push" ? tick : null;
  const moved = error instanceof ApiError && error.code === "remoteMoved";

  return (
    <ConfirmDialog
      open
      size="wide"
      title={<Term id="force-push" focusable={false} />}
      subtitle={data?.branch && data.remote ? `${data.branch} → ${data.remote}/${data.remoteBranch}` : undefined}
      dangerLabel={<Term id="force-push" focusable={false} />}
      closeLabel={m.close}
      pending={pending}
      dangerDisabled={input === null}
      onConfirm={() => void run()}
      onClose={() => !pending && finish()}
    >
      {allowed && <Loading pending={plan.isPending} />}
      {plan.error && <ErrorNotice error={plan.error} />}
      {blocked && <p className={styles.blocked}>{m.forcePush.blocked[blocked]}</p>}
      {data && !blocked && (
        <>
          <ul className={styles.counts}>
            <li>{m.forcePush.ahead(data.ahead)}</li>
            <li>{m.forcePush.dropped(data.dropped)}</li>
            {data.droppedByOthers > 0 && <li className={styles.others}>{m.forcePush.byOthers(data.droppedByOthers)}</li>}
          </ul>
          <ul className={styles.sample}>
            {data.sample.map((v) => (
              <li key={v.shortOid} className={styles.version}>
                <span className={styles.oid}>{v.shortOid}</span>
                <Truncate className={styles.summary}>{v.summary}</Truncate>
                <Truncate className={styles.author}>{v.author}</Truncate>
              </li>
            ))}
          </ul>
          {data.dropped > data.sample.length && <p className={styles.more}>{m.forcePush.more(data.dropped - data.sample.length)}</p>}
          {data.remote && data.remoteBranch && (
            <NameConfirmField
              repo={repo}
              remote={data.remote}
              branch={data.remoteBranch}
              needsName={needsName}
              value={typed}
              onChange={setTyped}
            />
          )}
        </>
      )}
      {pending && (
        <p className={styles.running} role="status">
          {m.forcePush.running(progress?.elapsedSecs ?? null)}
          {progress && progress.elapsedSecs >= CANCEL_AFTER_SECS && (
            <Button variant="text" onClick={cancel}>
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
