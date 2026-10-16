import { useState } from "react";
import type { ErrorCode } from "../../../../lib/api";
import { useCommitDetail } from "../../../../lib/queries";
import { matches } from "../../../../lib/shortcuts";
import { useMessages } from "../../../../messages";
import { Button } from "../../../../ui/Button";
import { Term } from "../../../../ui/Term";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { runAction } from "../run";
import { Frame } from "./Frame";
import styles from "./Dialogs.module.css";

type Props = { info: StationInfo; stagedFiles: number; ctx: MenuCtx; onClose: () => void };

/**
 * 마지막 버전 고치기(D-097). 지금 위치 버전의 전체 메시지를 한 글상자에 채우고(첫 줄이 이름), 넣은
 * 파일이 있으면 함께 들어간다. 해시(`expectHead`)와 올림 여부는 core가 다시 확인한다.
 */
export function AmendDialog({ info, stagedFiles, ctx, onClose }: Props) {
  const m = useMessages();
  const detail = useCommitDetail(ctx.repo, info.oid);
  const original = detail.data?.message.trimEnd() ?? null;
  // 받은 메시지로 처음 채운다. 고치기 시작하면 사람이 쓴 글이 이긴다.
  const [draft, setDraft] = useState<string | null>(null);
  const text = draft ?? original ?? "";
  const [error, setError] = useState<ErrorCode | null>(null);
  const [busy, setBusy] = useState(false);
  const changed = original !== null && text.trimEnd() !== original && text.trim() !== "";
  const ready = !busy && original !== null && (changed || stagedFiles > 0);

  const submit = async () => {
    if (!ready) return;
    setBusy(true);
    setError(null);
    const result = await runAction(
      ctx,
      { id: "amend", input: { expectHead: info.oid, message: changed ? text : null } },
      { done: () => ctx.m.amended, onError: setError },
    );
    setBusy(false);
    if (result !== null) onClose();
  };

  return (
    <Frame
      title={<Term id="amend" focusable={false} />}
      station={
        <>
          {info.shortOid} · <bdi>{info.summary}</bdi>
        </>
      }
      onClose={onClose}
      className={styles.wide}
    >
      <textarea
        className={`${styles.field} ${styles.message}`}
        value={text}
        aria-label={m.amendMessage}
        autoFocus
        disabled={original === null}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          // 버전 저장과 같은 키(Ctrl+Enter). 한글 조합 중에는 matches가 거른다.
          if (matches("commit", e.nativeEvent)) {
            e.preventDefault();
            void submit();
          }
        }}
      />
      {stagedFiles > 0 && <p className={styles.hint}>{m.amendStaged(stagedFiles)}</p>}
      {detail.error && <p className={styles.error}>{m.errors[detail.error.code]}</p>}
      {error !== null && (
        <p className={styles.error} role="alert">
          {m.errors[error]}
        </p>
      )}
      <div className={styles.actions}>
        <Button variant="text" onClick={onClose}>
          {m.close}
        </Button>
        <Button variant="primary" disabled={!ready} onClick={() => void submit()}>
          {m.amendAction}
        </Button>
      </div>
    </Frame>
  );
}
