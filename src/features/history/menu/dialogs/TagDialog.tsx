import { useState } from "react";
import type { ErrorCode } from "../../../../lib/api";
import { useBranches } from "../../../../lib/queries";
import { RemoteTagDeleteDialog } from "../../../remote-tag/RemoteTagDeleteDialog";
import { useMessages } from "../../../../messages";
import { Button } from "../../../../ui/Button";
import { Term } from "../../../../ui/Term";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { runAction } from "../run";
import { DialogFrame } from "../../../../ui/DialogFrame";
import styles from "./Dialogs.module.css";
import { isEnter } from "../../../../lib/keys";

type Props = { info: StationInfo; ctx: MenuCtx; onClose: () => void };

/** 태그 달기·지우기(D-104). 이름은 core가 다시 검사한다. 오류는 대화상자 안에 보인다. */
export function TagDialog({ info, ctx, onClose }: Props) {
  const m = useMessages();
  const [name, setName] = useState("");
  const [error, setError] = useState<ErrorCode | null>(null);
  const [busy, setBusy] = useState(false);
  const [remoteDelete, setRemoteDelete] = useState<string | null>(null);
  const hasRemote = (useBranches(ctx.repo).data?.remotes.length ?? 0) > 0;

  const act = async (run: () => Promise<unknown>) => {
    setBusy(true);
    setError(null);
    const result = await run();
    setBusy(false);
    if (result !== null) onClose();
  };
  const create = () =>
    act(() =>
      runAction(
        ctx,
        { id: "tag", input: { oid: info.oid, name } },
        { done: () => ctx.m.tagged, onError: setError },
      ),
    );
  // 올리기는 덧붙이기만 해 되돌리기 없이 알린다(D-131). 창은 그대로 둔다.
  const push = async (tag: string) => {
    setBusy(true);
    setError(null);
    await runAction(
      ctx,
      { id: "tag-push", input: { name: tag } },
      {
        done: () => {
          ctx.toast(m.tagPush.done);
          return null;
        },
        onError: setError,
      },
    );
    setBusy(false);
  };
  const remove = (tag: string) =>
    act(() =>
      runAction(
        ctx,
        { id: "tag-delete", input: { name: tag, oid: info.oid } },
        { done: () => ctx.m.tagDeleted, onError: setError },
      ),
    );

  return (
    <DialogFrame
      title={<Term id="tag" focusable={false} />}
      subtitle={
        <>
          {info.shortOid} · <bdi>{info.summary}</bdi>
        </>
      }
      onClose={onClose}
    >
      <input
        className={styles.field}
        value={name}
        placeholder="v1.0"
        aria-label={m.tagName}
        autoFocus
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => {
          if (!isEnter(e)) return;
          e.preventDefault();
          if (name.trim() && !busy) void create();
        }}
      />
      {error !== null && (
        <p className={styles.error} role="alert">
          {m.errors[error]}
        </p>
      )}
      {info.tags.length > 0 && (
        <ul className={styles.list} aria-label={m.tagName}>
          {info.tags.map((tag) => (
            <li key={tag} className={styles.listItem}>
              <bdi>{tag}</bdi>
              <span>
                <Button variant="text" disabled={busy} onClick={() => void push(tag)}>
                  {m.tagPush.push}
                </Button>
                <Button variant="text" disabled={busy} onClick={() => void remove(tag)}>
                  {m.deleteTag}
                </Button>
                {hasRemote && (
                  <Button variant="text" disabled={busy} onClick={() => setRemoteDelete(tag)}>
                    {m.remoteTag.deleteOnRemote}
                  </Button>
                )}
              </span>
            </li>
          ))}
        </ul>
      )}
      <div className={styles.actions}>
        <Button variant="text" onClick={onClose}>
          {m.close}
        </Button>
        <Button variant="primary" disabled={busy || !name.trim()} onClick={() => void create()}>
          {m.create}
        </Button>
      </div>
      {remoteDelete !== null && (
        <RemoteTagDeleteDialog
          repo={ctx.repo}
          name={remoteDelete}
          oid={info.oid}
          onClose={() => setRemoteDelete(null)}
          onDone={() => {
            setRemoteDelete(null);
            ctx.toast(m.remoteTag.done);
          }}
        />
      )}
    </DialogFrame>
  );
}
