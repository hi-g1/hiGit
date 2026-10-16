import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import { runAction, type RunCtx } from "../../features/history/menu/run";
import { CardMap } from "../../features/rewrite/CardMap";
import { changed, fromPlan, summary, toInput, type Card } from "../../features/rewrite/model";
import { useRewritePlan } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./Rewrite.module.css";

type Props = {
  repo: string;
  /** 정리하면 결과 토스트와 되돌리기(useUndo의 notifyDone). */
  onDone: (text: string) => void;
};

/** 같은 버전들인지(순서 무관). `.git` 변경으로 계획을 다시 읽어도 고친 카드는 해시 기준으로 남긴다. */
function sameCards(a: readonly Card[], b: readonly Card[]): boolean {
  if (a.length !== b.length) return false;
  const ids = new Set(b.map((c) => c.oid));
  return a.every((c) => ids.has(c.oid));
}

/**
 * 기록 정리 화면(D-100, D-106): 고른 구간의 카드 지도. 순서 바꾸기·묶기·빼기·메시지 고치기를 한 뒤
 * "정리하기"로 core의 `rebase-i`를 부른다. 막힌 까닭이 있으면 카드는 읽기만. Esc나 "← 버전 기록"은
 * 버전 기록 화면으로 돌아간다.
 */
export function Rewrite({ repo, onDone }: Props) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const toast = useToastStore((s) => s.show);
  const toastError = useToastStore((s) => s.showError);
  const oids = useRepoStore((s) => s.rewriteOids);
  const setView = useRepoStore((s) => s.setView);
  const plan = useRewritePlan(repo, oids);
  const original = useMemo(() => (plan.data ? fromPlan(plan.data) : []), [plan.data]);
  const [edited, setEdited] = useState<Card[] | null>(null);
  const cards = edited && sameCards(edited, original) ? edited : original;
  const blocked = plan.data?.blocked ?? null;
  const [busy, setBusy] = useState(false);
  const dirty = changed(cards, original);
  const counts = summary(cards);

  // 정리하는 동안에는 화면을 떠나지 않는다(끝나면 버전 기록으로 돌아간다).
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented && !busy) setView("history");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setView, busy]);

  const ctx: RunCtx = { repo, toast, toastError, queryClient, onDone, openActivity: () => setView("activity"), m };
  const run = async () => {
    setBusy(true);
    const result = await runAction(ctx, { id: "rebase-i", input: { steps: toInput(cards) } }, { done: () => m.rewritten });
    setBusy(false);
    // 끝났거나 겹쳐 멈췄으면(작업 화면이 충돌 해결 화면을 연다) 버전 기록으로 돌아간다.
    if (result !== null) setView("history");
  };

  return (
    <section className={styles.screen} aria-label={m.cardList}>
      <header className={styles.head}>
        <h2 className={styles.title}>
          <Term id="rebase-i" focusable={false} />
        </h2>
        <Button variant="text" disabled={busy} onClick={() => setView("history")}>
          {m.backToLog}
        </Button>
        <span className={styles.spacer} />
        <Button variant="text" disabled={busy || !dirty} onClick={() => setEdited(null)}>
          {m.rewriteReset}
        </Button>
        <Button variant="primary" disabled={busy || !dirty || blocked !== null} onClick={() => void run()}>
          {m.rewriteRun}
        </Button>
      </header>
      <div className={styles.body}>
        {plan.isPending && <EmptyState>{m.loading}</EmptyState>}
        {plan.error && <ErrorNotice error={plan.error} />}
        {blocked !== null && (
          <p className={styles.notice} role="alert">
            <span>{m.rewriteBlocked[blocked]}</span>
          </p>
        )}
        {(plan.data?.staleLabels ?? 0) > 0 && <p className={styles.note}>{m.staleLabels(plan.data!.staleLabels)}</p>}
        {plan.data && <CardMap cards={cards} original={original} onChange={setEdited} readOnly={blocked !== null || busy} />}
      </div>
      {plan.data && <p className={styles.foot}>{m.rewriteSummary(counts.after, counts.dropped, counts.joined)}</p>}
    </section>
  );
}
