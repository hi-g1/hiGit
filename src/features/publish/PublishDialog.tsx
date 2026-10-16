import { useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { ApiError, api, toApiError, type Visibility } from "../../lib/api";
import { invalidateLog, queryKeys, usePublishDraft } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { composing, DialogFrame } from "../../ui/DialogFrame";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import { CANCEL_AFTER_SECS, useProgress } from "../progress/useProgress";
import styles from "./PublishDialog.module.css";
import { useSettingsStore } from "../settings/settingsStore";
import { usePublishStore } from "./publishStore";

/** GitHub 저장소 이름 모양. 화면은 바로 안내만 하고 최종 검사는 core가 한다. */
const NAME = /^[A-Za-z0-9._-]{1,100}$/;
const nameOk = (name: string) => NAME.test(name) && name !== "." && name !== "..";

/**
 * GitHub에 올리기 대화상자(D-132, design.md "GitHub에 올리기 대화상자"). 이름·설명·공개 여부만 보내고
 * 주소는 core가 만든다. 저장소를 만든 뒤 이어서 기존 올리기를 부른다. 공개는 대화상자 안에서 한 번 더 묻는다.
 */
export function PublishDialog({ repo }: { repo: string }) {
  const open = usePublishStore((s) => s.open);
  if (!open) return null;
  return <Dialog repo={repo} />;
}

function Dialog({ repo }: { repo: string }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const close = usePublishStore((s) => s.close);
  const openSettings = useSettingsStore((s) => s.openSettings);
  const draft = usePublishDraft(repo).data;
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const tick = useProgress(repo);
  const [name, setName] = useState<string | null>(null);
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<Visibility>("private");
  const [confirmPublic, setConfirmPublic] = useState(false);
  const [phase, setPhase] = useState<"idle" | "creating" | "pushing">("idle");
  const [created, setCreated] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const keepPrivate = useRef<HTMLButtonElement>(null);
  const shownName = name ?? draft?.suggestedName ?? "";
  const busy = phase !== "idle";

  const refresh = () => {
    invalidateLog(queryClient, repo);
    for (const key of [queryKeys.branches(repo), queryKeys.status(repo), queryKeys.homeMap(repo), ["github", "context", repo]]) {
      void queryClient.invalidateQueries({ queryKey: key });
    }
  };
  const done = () => {
    close();
    show(m.publish.done, {
      label: m.publish.open,
      onClick: () => void api.openHostingPage(repo, { kind: "repository" }).catch(showError),
    });
  };
  const push = async () => {
    setPhase("pushing");
    setError(null);
    try {
      await api.executeAction(repo, { id: "push" });
      done();
    } catch (e) {
      if (e instanceof ApiError && e.code === "cancelled") show(m.errors.cancelled);
      else setError(e);
    } finally {
      setPhase("idle");
      refresh();
    }
  };
  const create = async (acknowledgePublic: boolean) => {
    setConfirmPublic(false);
    setPhase("creating");
    setError(null);
    try {
      await api.executeAction(repo, {
        id: "publish",
        input: { name: shownName.trim(), description: description.trim() || null, visibility, acknowledgePublic },
      });
      setCreated(true);
    } catch (e) {
      setError(e);
      setPhase("idle");
      return;
    }
    await push();
  };
  const submit = () => {
    if (busy || !nameOk(shownName.trim()) || created) return;
    if (visibility === "public") {
      setConfirmPublic(true);
      setTimeout(() => keepPrivate.current?.focus(), 0);
      return;
    }
    void create(false);
  };
  const progress = phase === "pushing" && tick?.action === "push" ? tick : null;

  return (
    <DialogFrame
      title={<Term id="publish" focusable={false} />}
      subtitle={draft?.branch ? <span className={styles.code}>{m.publish.subtitle(draft.branch, draft.versions)}</span> : undefined}
      onClose={() => !busy && close()}
      size="wide"
    >
      <label className={styles.field}>
        <span>{m.publish.name}</span>
        <input
          className={`${styles.input} ${styles.code}`}
          spellCheck={false}
          value={shownName}
          autoFocus
          disabled={busy || created}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key !== "Enter" || composing(e)) return;
            e.preventDefault();
            submit();
          }}
        />
      </label>
      {draft && (shownName === "" || !nameOk(shownName.trim())) && <p className={styles.hint}>{m.publish.nameHint}</p>}
      <label className={styles.field}>
        <span>{m.publish.description}</span>
        <input
          className={styles.input}
          maxLength={350}
          value={description}
          disabled={busy || created}
          onChange={(e) => setDescription(e.target.value)}
        />
      </label>
      <div role="radiogroup" aria-label={m.publish.visibility} className={styles.choices}>
        {(["private", "public"] as const).map((v) => (
          <label key={v}>
            <input
              type="radio"
              name="visibility"
              checked={visibility === v}
              disabled={busy || created}
              onChange={() => setVisibility(v)}
            />{" "}
            {v === "private" ? m.publish.private : m.publish.public}
          </label>
        ))}
        {visibility === "public" && <p className={styles.hint}>{m.publish.publicHint}</p>}
      </div>
      {confirmPublic && (
        <div className={styles.confirm} role="alert">
          <p>{m.publish.publicConfirm}</p>
          <div className={styles.row}>
            <Button
              ref={keepPrivate}
              variant="primary"
              onClick={() => {
                setVisibility("private");
                setConfirmPublic(false);
              }}
            >
              {m.publish.keepPrivate}
            </Button>
            <Button variant="danger" onClick={() => void create(true)}>
              {m.publish.makePublic}
            </Button>
          </div>
        </div>
      )}
      {phase !== "idle" && (
        <p className={styles.status} role="status">
          {phase === "creating" ? m.publish.creating : m.forcePush.running(progress?.elapsedSecs ?? null)}
          {progress && progress.elapsedSecs >= CANCEL_AFTER_SECS && (
            <Button variant="text" onClick={() => void api.cancelAction(repo).catch(showError)}>
              {m.cancel}
            </Button>
          )}
        </p>
      )}
      {created && error !== null && <p className={styles.hint}>{m.publish.createdOnly}</p>}
      {error !== null && (
        <ErrorNotice error={error}>
          {created && (
            <Button variant="secondary" disabled={busy} onClick={() => void push()}>
              {m.publish.retryPush}
            </Button>
          )}
          {!created && ["githubNotConnected", "githubExpired"].includes(toApiError(error).code) && (
            <Button
              variant="secondary"
              onClick={() => {
                close();
                openSettings(true);
              }}
            >
              {m.settings.connect}
            </Button>
          )}
        </ErrorNotice>
      )}
      <div className={styles.footer}>
        <Button variant="text" disabled={busy} onClick={close}>
          {m.close}
        </Button>
        <Button variant="primary" disabled={busy || created || !nameOk(shownName.trim())} onClick={submit}>
          <Term id="publish" focusable={false} />
        </Button>
      </div>
    </DialogFrame>
  );
}
