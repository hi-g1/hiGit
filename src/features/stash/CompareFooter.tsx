import * as Popover from "@radix-ui/react-popover";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, type ChangedFile } from "../../lib/api";
import { useStashes } from "../../lib/queries";
import { Time } from "../../ui/Time";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./CompareFooter.module.css";
import { useDiscardFlow } from "./useDiscardFlow";
import { Truncate } from "../../ui/Truncate";

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
 * 버리기는 확인 없이 하고 토스트의 되돌리기로 되살린다(D-196).
 */
export function CompareFooter({ repo, file, hasChanges, onDone }: Props) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const stashes = useStashes(repo).data?.items ?? [];
  const discardFlow = useDiscardFlow(repo, onDone);
  const [pending, setPending] = useState(false);
  const [now] = useState(() => Date.now());

  const refresh = () => {
    for (const key of ["status", "diff", "stashes", "recovery", "homeMap"]) {
      void queryClient.invalidateQueries({ queryKey: [key, repo] });
    }
  };
  const fail = (e: unknown) => showError(e);

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

  return (
    <div className={styles.footer}>
      <Button
        variant="text"
        disabled={pending}
        disabledReason={hasChanges ? null : m.disabledReasons.noChanges}
        onClick={() => void stash()}
      >
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
                    <Truncate className={styles.itemName}>{s.branch ?? s.message}</Truncate>
                    <span className={styles.itemMeta}>
                      <Time at={s.at} now={now} /> · {m.filesCount(s.files)}
                    </span>
                    <Button
                      disabled={pending}
                      disabledReason={hasChanges ? m.takeOutNeedsClean : null}
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
      <button type="button" className={styles.discard} disabled={pending || discardFlow.pending} onClick={() => discardFlow.run(file)}>
        <Term id="discard" focusable={false} />
      </button>
    </div>
  );
}
