import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { backupLabel, groupByDay, restoreInput } from "../../features/activity/model";
import { NewBranchDialog } from "../../features/history/menu/dialogs/NewBranchDialog";
import type { RunCtx } from "../../features/history/menu/run";
import type { TermId } from "../../glossary/types";
import { ApiError, api, type RecoveryItem } from "../../lib/api";
import { invalidateLog, useActions, useRecovery } from "../../lib/queries";
import { formatClock, formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { Tooltip } from "../../ui/Tooltip";
import { useToastStore } from "../../ui/toastStore";
import styles from "./Activity.module.css";

type Props = {
  repo: string;
  /** 되돌리면 결과 토스트와 되돌리기(useUndo의 notifyDone). */
  onDone: (text: string) => void;
};

/**
 * 작업 내역 화면(D-045, D-085). 앱 작업·Git 기록·백업·맡긴 파일·임시 보관을 시간순 카드로.
 * 창 전체 폭이고 Esc나 상단 바 "작업 내역"을 다시 누르면 홈으로 돌아간다(D-058과 같은 방식).
 */
export function Activity({ repo, onDone }: Props) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const recovery = useRecovery(repo);
  const actions = useActions().data;
  const setView = useRepoStore((s) => s.setView);
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const [now] = useState(() => Date.now());
  const [pending, setPending] = useState<string | null>(null);
  // "새 브랜치로"로 연 대화상자의 버전.
  const [branchFrom, setBranchFrom] = useState<RecoveryItem | null>(null);
  const ctx: RunCtx = { repo, toast: show, queryClient, onDone, openActivity: () => setView("activity"), m };

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented) setView("home");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setView]);

  const termOf = (action: string): TermId | null => {
    const id = actions?.find((a) => a.id === action)?.glossaryId;
    return (id as TermId | undefined) ?? null;
  };
  const sourceLabel = (item: RecoveryItem) => m.activitySource[item.source];

  const run = async (item: RecoveryItem) => {
    const input = restoreInput(item);
    if (!input) return;
    setPending(item.id);
    try {
      const result = await api.executeAction(repo, input);
      if (result.kind === "restored") onDone(m.restored);
      else show(m.tookOutToast);
    } catch (e) {
      show(m.errors[e instanceof ApiError ? e.code : "internal"]);
    } finally {
      setPending(null);
      invalidateLog(queryClient, repo);
      for (const key of ["status", "recovery", "homeMap", "branches", "stashes", "mergeHint"]) {
        void queryClient.invalidateQueries({ queryKey: [key, repo] });
      }
    }
  };

  const items = recovery.data?.items ?? [];
  const groups = groupByDay(items, (at) => formatDate(at, now, locale));
  return (
    <section className={styles.activity} aria-label={m.activityArea}>
      <p className={styles.note}>{m.activityNote}</p>
      {recovery.isPending && <EmptyState>{m.loading}</EmptyState>}
      {recovery.error && <ErrorNotice error={recovery.error} />}
      {recovery.data && items.length === 0 && <EmptyState>{m.activityEmpty}</EmptyState>}
      {groups.map((g) => (
        <div key={g.day} className={styles.day}>
          <h2 className={styles.dayTitle}>{g.day}</h2>
          <ul className={styles.cards}>
            {g.items.map((item) => {
              const term = item.action ? termOf(item.action) : null;
              const input = restoreInput(item);
              return (
                <li key={item.id} className={`${styles.card} ${item.undone ? styles.undone : ""}`}>
                  <span className={styles.time}>
                    {formatClock(item.at, locale)}
                  </span>
                  <span className={styles.title}>
                    {term ? <Term id={term} /> : sourceLabel(item)}
                    {item.source === "backup" ? (
                      <BackupName label={item.label} />
                    ) : (
                      item.source !== "reflog" && item.label && <span className={styles.label}>{item.label}</span>
                    )}
                  </span>
                  <span className={styles.summary}>{item.summary}</span>
                  {item.source === "reflog" && (
                    <details className={styles.raw}>
                      <summary>{m.showDetail}</summary>
                      <code>{item.label}</code>
                    </details>
                  )}
                  {input && (
                    <span className={styles.buttons}>
                      <Button disabled={pending !== null} onClick={() => void run(item)}>
                        {item.restore === "version" ? m.restoreHere : m.takeOut}
                      </Button>
                      {/* 서명이 맞는 줄·git 기록·백업처럼 되살릴 수 있는 버전만(core가 해시를 다시 본다, E18). */}
                      {item.restore === "version" && item.oid && (
                        <Button
                          variant="text"
                          data-new-branch={item.id}
                          disabled={pending !== null}
                          onClick={() => setBranchFrom(item)}
                        >
                          {m.toNewBranch}
                        </Button>
                      )}
                    </span>
                  )}
                </li>
              );
            })}
          </ul>
        </div>
      ))}
      {branchFrom?.oid && (
        <NewBranchDialog
          oid={branchFrom.oid}
          shortOid={branchFrom.oid.slice(0, 7)}
          summary={branchFrom.summary}
          ctx={ctx}
          onClose={() => {
            const id = branchFrom.id;
            setBranchFrom(null);
            // 닫으면 그 카드의 "새 브랜치로"로 돌아간다.
            setTimeout(() => document.querySelector<HTMLElement>(`[data-new-branch="${CSS.escape(id)}"]`)?.focus(), 0);
          }}
        />
      )}
      <div className={styles.footer}>
        <Button variant="text" onClick={() => void api.openLogFolder().catch(() => undefined)}>
          {m.openLogs}
        </Button>
      </div>
    </section>
  );
}

/** 백업 이름: 아는 이름은 용어와 브랜치 이름으로, 원문은 툴팁에. 모르는 이름은 원문 그대로. */
function BackupName({ label }: { label: string }) {
  const known = backupLabel(label);
  if (!label) return null;
  if (!known) return <span className={styles.label}>{label}</span>;
  return (
    <Tooltip content={<code>{label}</code>}>
      <span className={styles.label} tabIndex={0}>
        <Term id={known.term} focusable={false} />
        {known.name !== null && <bdi> · {known.name}</bdi>}
      </span>
    </Tooltip>
  );
}
