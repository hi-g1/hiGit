import { useState } from "react";
import type { ErrorCode } from "../../../../lib/api";
import { useBranchNameCheck } from "../../../../lib/queries";
import { useMessages } from "../../../../messages";
import { Button } from "../../../../ui/Button";
import { runAction, type RunCtx } from "../run";
import { composing, DialogFrame } from "../../../../ui/DialogFrame";
import styles from "./Dialogs.module.css";

type Props = {
  /** 시작할 버전과 그 이름(역 메뉴의 역, 작업 내역의 버전). */
  oid: string;
  shortOid: string;
  summary: string;
  ctx: RunCtx;
  onClose: () => void;
};

/** 고른 버전에서 새 브랜치(D-098, D-099). 이름 규칙 안내는 기존 이름 검사, 판단은 core가 한다. */
export function NewBranchDialog({ oid, shortOid, summary, ctx, onClose }: Props) {
  const m = useMessages();
  const [name, setName] = useState("");
  const [error, setError] = useState<ErrorCode | null>(null);
  const [busy, setBusy] = useState(false);
  const check = useBranchNameCheck(ctx.repo, name);
  const problem = name.trim() && check.data && !check.data.ok ? check.data.problem : null;

  const create = async () => {
    setBusy(true);
    setError(null);
    const result = await runAction(
      ctx,
      { id: "branch-create", input: { name, from: oid } },
      { done: () => ctx.m.branchCreatedHere(name.trim()), onError: setError },
    );
    setBusy(false);
    if (result !== null) onClose();
  };

  return (
    <DialogFrame
      title={m.newBranchHere}
      subtitle={
        <>
          {shortOid} · <bdi>{summary}</bdi>
        </>
      }
      onClose={onClose}
    >
      <input
        className={styles.field}
        value={name}
        aria-label={m.branchName}
        autoFocus
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Enter" || composing(e)) return;
          e.preventDefault();
          if (name.trim() && !busy && problem === null) void create();
        }}
      />
      {problem !== null && <p className={styles.hint}>{problem === "exists" ? m.sameBranchName : m.badBranchName}</p>}
      {error !== null && (
        <p className={styles.error} role="alert">
          {m.errors[error]}
        </p>
      )}
      <div className={styles.actions}>
        <Button variant="text" onClick={onClose}>
          {m.close}
        </Button>
        <Button variant="primary" disabled={busy || !name.trim() || problem !== null} onClick={() => void create()}>
          {m.create}
        </Button>
      </div>
    </DialogFrame>
  );
}
