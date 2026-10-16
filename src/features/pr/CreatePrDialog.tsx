import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, toApiError } from "../../lib/api";
import { queryKeys, usePrDraft, usePrForBranch } from "../../lib/queries";
import { matches } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Checkbox } from "../../ui/Checkbox";
import { DialogFrame } from "../../ui/DialogFrame";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import { CANCEL_AFTER_SECS, useProgress } from "../progress/useProgress";
import { useSettingsStore } from "../settings/settingsStore";
import { usePrDialogStore } from "./prDialogStore";
import styles from "./CreatePrDialog.module.css";

/** 새 검토 요청 대화상자(D-113). 열린 동안만 그린다. */
export function CreatePrDialog({ repo }: { repo: string }) {
  const open = usePrDialogStore((s) => s.open);
  return open ? <CreatePrForm repo={repo} /> : null;
}

function CreatePrForm({ repo }: { repo: string }) {
  const m = useMessages();
  const close = usePrDialogStore((s) => s.close);
  const openSettings = useSettingsStore((s) => s.openSettings);
  const showToast = useToastStore((s) => s.show);
  const queryClient = useQueryClient();
  const draft = usePrDraft(repo);
  const data = draft.data;
  const existingQuery = usePrForBranch(repo, data?.head ?? null, data !== undefined);
  const existing = existingQuery.data?.items.find((r) => r.state === "open" && r.isCurrentBranch) ?? null;
  const [title, setTitle] = useState<string | null>(null);
  const [body, setBody] = useState("");
  const [base, setBase] = useState<string | null>(null);
  const [asDraft, setAsDraft] = useState(false);
  const [running, setRunning] = useState<"push" | "create" | null>(null);
  const [error, setError] = useState<unknown>(null);
  const progress = useProgress(repo);

  const titleValue = title ?? data?.suggestionTitle ?? "";
  const baseValue = base ?? data?.base ?? "";
  const head = data?.head ?? data?.branch ?? "";
  const nothing = data !== undefined && data.ahead === 0 && baseValue === data.base;
  const code = error === null ? null : toApiError(error).code;
  const blocked = existing !== null || code === "prAlreadyExists";
  const ready = data !== undefined && titleValue.trim() !== "" && baseValue !== "" && !nothing && !blocked;

  const submit = async () => {
    if (!ready || running || !data) return;
    setError(null);
    try {
      // 올리기는 기존 액션(위험도·journal·취소·원격 잠금)으로 먼저 하고, 검토 요청은 그 뒤(D-113).
      if (data.head === null || data.unpushed > 0) {
        setRunning("push");
        await api.executeAction(repo, { id: "push" });
      }
      setRunning("create");
      const result = await api.executeAction(repo, {
        id: "pr-create",
        input: { title: titleValue.trim(), body, base: baseValue, draft: asDraft },
      });
      close();
      void queryClient.invalidateQueries({ queryKey: queryKeys.github });
      void queryClient.invalidateQueries({ queryKey: ["status", repo] });
      if (result.kind === "prCreated") {
        const number = result.number;
        showToast(m.pr.created, {
          label: m.pr.openPage,
          onClick: () => void api.openHostingPage(repo, { kind: "pullRequest", number }).catch(() => undefined),
        });
      }
    } catch (e) {
      setError(e);
      void queryClient.invalidateQueries({ queryKey: queryKeys.prDraft(repo) });
    } finally {
      setRunning(null);
    }
  };

  const openExisting = (number: number) =>
    void api.openHostingPage(repo, { kind: "pullRequest", number }).catch(() => undefined);

  return (
    <DialogFrame
      title={m.pr.newPr}
      subtitle={data ? `${head} → ${baseValue}` : undefined}
      onClose={() => running === null && close()}
      size="wide"
    >
      {draft.error && <ErrorNotice error={draft.error} />}
      {data && (
        <>
          <input
            className={styles.title}
            aria-label={m.pr.titleLabel}
            value={titleValue}
            autoFocus
            disabled={running !== null || blocked}
            onChange={(e) => setTitle(e.target.value)}
            onKeyDown={(e) => {
              if (matches("commit", e.nativeEvent)) {
                e.preventDefault();
                void submit();
              }
            }}
          />
          <textarea
            className={styles.body}
            aria-label={m.pr.bodyPlaceholder}
            placeholder={m.pr.bodyPlaceholder}
            value={body}
            disabled={running !== null || blocked}
            onChange={(e) => setBody(e.target.value)}
            onKeyDown={(e) => {
              if (matches("commit", e.nativeEvent)) {
                e.preventDefault();
                void submit();
              }
            }}
          />
          <div className={styles.row}>
            <label className={styles.inline}>
              {m.pr.base}
              <select
                className={styles.select}
                value={baseValue}
                disabled={running !== null || blocked}
                onChange={(e) => setBase(e.target.value)}
              >
                {data.baseOptions.map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            </label>
            <label className={styles.inline}>
              <Checkbox
                checked={asDraft}
                disabled={running !== null || blocked}
                onChange={(e) => setAsDraft(e.target.checked)}
              />
              <Term id="pr-draft" focusable={false} />
            </label>
          </div>
          <p className={styles.muted}>
            {nothing ? m.pr.same(baseValue) : m.pr.ahead(baseValue, data.ahead)}
            {data.unpushed > 0 && ` · ${m.pr.unpushed(data.unpushed)}`}
          </p>
        </>
      )}
      {existing && (
        <div className={styles.row}>
          <p className={styles.notice}>{m.pr.exists(existing.number)}</p>
          <Button variant="text" onClick={() => openExisting(existing.number)}>
            {m.pr.openOnGithub}
          </Button>
        </div>
      )}
      {error !== null && (
        <ErrorNotice error={error}>
          {code === "githubExpired" && (
            <Button
              variant="secondary"
              onClick={() => {
                close();
                openSettings(true);
              }}
            >
              {m.pr.reconnect}
            </Button>
          )}
        </ErrorNotice>
      )}
      <div className={styles.actions}>
        {running === "push" && (
          <span className={styles.progress} role="status">
            {m.pr.pushing(progress?.elapsedSecs ?? 0)}
          </span>
        )}
        {running === "push" && (progress?.elapsedSecs ?? 0) >= CANCEL_AFTER_SECS && (
          <Button variant="text" onClick={() => void api.cancelAction(repo).catch(() => undefined)}>
            {m.pr.cancel}
          </Button>
        )}
        {running === null && (
          <Button variant="text" onClick={close}>
            {m.pr.close}
          </Button>
        )}
        <Button variant="primary" disabled={!ready || running !== null} onClick={() => void submit()}>
          {m.pr.create}
        </Button>
      </div>
    </DialogFrame>
  );
}
