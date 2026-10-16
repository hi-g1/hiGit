import { useState } from "react";
import type { ErrorCode } from "../../../../lib/api";
import { useMessages } from "../../../../messages";
import { Button } from "../../../../ui/Button";
import { Term } from "../../../../ui/Term";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { runAction } from "../run";
import { composing, DialogFrame } from "../../../../ui/DialogFrame";
import styles from "./Dialogs.module.css";

type Props = { info: StationInfo; ctx: MenuCtx; onClose: () => void };

/** 태그 달기·지우기(D-104). 이름은 core가 다시 검사한다. 오류는 대화상자 안에 보인다. */
export function TagDialog({ info, ctx, onClose }: Props) {
  const m = useMessages();
  const [name, setName] = useState("");
  const [error, setError] = useState<ErrorCode | null>(null);
  const [busy, setBusy] = useState(false);

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
          if (e.key !== "Enter" || composing(e)) return;
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
              <Button variant="text" disabled={busy} onClick={() => void remove(tag)}>
                {m.deleteTag}
              </Button>
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
    </DialogFrame>
  );
}
