import * as Dialog from "@radix-ui/react-dialog";
import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { ApiError, api, type ChangedFile } from "../../lib/api";
import { useStashes } from "../../lib/queries";
import { Time } from "../../ui/Time";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./CompareFooter.module.css";

type Props = {
  repo: string;
  /** 비교 칸에 열린 파일. 버리기는 이 파일에만 쓴다. */
  file: ChangedFile;
  /** 고친 파일이 있는지(꺼내기는 깨끗할 때만). */
  hasChanges: boolean;
  /** 보관하면 결과 토스트와 되돌리기(useUndo의 notifyDone). */
  onDone: (text: string) => void;
};

/**
 * 비교 칸 맨 아래 줄(r-mergehint 오른쪽 아래): 임시 보관, 보관 목록, 고친 것 버리기(D-074, D-076).
 * 버리기는 되돌릴 수 없어 파일 이름을 보여 주는 확인을 거친다(D-043).
 */
export function CompareFooter({ repo, file, hasChanges, onDone }: Props) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const setView = useRepoStore((s) => s.setView);
  const stashes = useStashes(repo).data?.items ?? [];
  const [confirm, setConfirm] = useState<{ token: string } | null>(null);
  const [pending, setPending] = useState(false);
  const [now] = useState(() => Date.now());
  const closeRef = useRef<HTMLButtonElement>(null);
  const paths = file.origPath ? [file.path, file.origPath] : [file.path];

  const refresh = () => {
    for (const key of ["status", "diff", "stashes", "recovery", "homeMap"]) {
      void queryClient.invalidateQueries({ queryKey: [key, repo] });
    }
  };
  const fail = (e: unknown) => show(m.errors[e instanceof ApiError ? e.code : "internal"]);

  const stash = async () => {
    setPending(true);
    try {
      await api.executeAction(repo, { id: "stash" });
      onDone(m.stashed);
    } catch (e) {
      fail(e);
    } finally {
      setPending(false);
      refresh();
    }
  };

  const takeOut = async (oid: string) => {
    setPending(true);
    try {
      await api.executeAction(repo, { id: "take-out", input: { source: { kind: "stash", oid } } });
      show(m.tookOutToast);
    } catch (e) {
      fail(e);
    } finally {
      setPending(false);
      refresh();
    }
  };

  const askDiscard = async () => {
    try {
      const preview = await api.previewAction(repo, { id: "discard", input: { paths } });
      setConfirm({ token: preview.token });
    } catch (e) {
      fail(e);
    }
  };

  const discard = async () => {
    if (!confirm) return;
    setPending(true);
    try {
      await api.executeAction(repo, { id: "discard", input: { paths } }, confirm.token);
      setConfirm(null);
      // 되돌리기가 아니라 작업 내역에서 꺼낸다(백업이 거기 있다).
      show(m.discarded, { label: m.openActivity, onClick: () => setView("activity") });
    } catch (e) {
      setConfirm(null);
      fail(e);
    } finally {
      setPending(false);
      refresh();
    }
  };

  return (
    <div className={styles.footer}>
      <Button variant="text" disabled={pending || !hasChanges} onClick={() => void stash()}>
        <Term id="stash" focusable={false} />
      </Button>
      {stashes.length > 0 && (
        <Popover.Root>
          <Popover.Trigger asChild>
            <Button variant="text">{m.stashCount(stashes.length)}</Button>
          </Popover.Trigger>
          <Popover.Portal>
            <Popover.Content className={styles.list} side="top" align="start" sideOffset={6}>
              <ul>
                {stashes.map((s) => (
                  <li key={s.oid} className={styles.item}>
                    <span className={styles.itemName}>{s.branch ?? s.message}</span>
                    <span className={styles.itemMeta}>
                      <Time at={s.at} now={now} /> · {m.filesCount(s.files)}
                    </span>
                    <Button
                      disabled={pending || hasChanges}
                      title={hasChanges ? m.takeOutNeedsClean : undefined}
                      onClick={() => void takeOut(s.oid)}
                    >
                      {m.takeOut}
                    </Button>
                  </li>
                ))}
              </ul>
            </Popover.Content>
          </Popover.Portal>
        </Popover.Root>
      )}
      <span className={styles.spacer} />
      <button type="button" className={styles.discard} disabled={pending} onClick={() => void askDiscard()}>
        <Term id="discard" focusable={false} />
      </button>
      <Dialog.Root open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <Dialog.Portal>
          <Dialog.Overlay className={styles.overlay} />
          <Dialog.Content
            className={styles.dialog}
            // 되돌릴 수 없어 Enter가 버리기가 되지 않게 안전한 쪽에 포커스를 둔다(D-043).
            onOpenAutoFocus={(e) => {
              e.preventDefault();
              closeRef.current?.focus();
            }}
          >
            <Dialog.Title className={styles.title}>{m.discardTitle(file.path.split("/").pop() ?? file.path)}</Dialog.Title>
            <Dialog.Description className={styles.path}>{paths.join(" · ")}</Dialog.Description>
            <div className={styles.actions}>
              <Button variant="danger" disabled={pending} onClick={() => void discard()}>
                {m.discardConfirm}
              </Button>
              <Button ref={closeRef} variant="primary" onClick={() => setConfirm(null)}>
                {m.close}
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  );
}
