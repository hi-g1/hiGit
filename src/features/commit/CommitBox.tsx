import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, type KeyboardEvent } from "react";
import { api } from "../../lib/api";
import { queryKeys } from "../../lib/queries";
import { matches } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import styles from "./CommitBox.module.css";

/**
 * 버전 저장 상자(왼쪽 칸 아래, D-025). 작업 공간의 유일한 주 버튼이다.
 * 버튼이 꺼진 이유는 버튼 아래 둘째 줄로 알린다(D-027).
 */
export function CommitBox({ folder, includedCount }: { folder: string; includedCount: number }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const [message, setMessage] = useState("");
  const [saved, setSaved] = useState<string | null>(null);

  const commit = useMutation({
    mutationFn: () => api.executeAction(folder, { id: "commit", input: { message } }),
    onSuccess: (result) => {
      if (result.kind === "committed") setSaved(result.shortOid);
      setMessage("");
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.status(folder) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.log(folder) });
    },
  });

  const reason =
    includedCount === 0 ? m.commitNeedsFiles : message.trim() === "" ? m.commitNeedsMessage : null;
  const canCommit = reason === null && !commit.isPending;

  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    // 단축키 표(D-033)로 판정한다. 한글 조합 중 Enter(WebKitGTK의 keyCode 229 포함)는
    // 글자를 확정하는 키라서 저장하지 않는다(design.md).
    if (matches("commit", e.nativeEvent)) {
      e.preventDefault();
      if (canCommit) commit.mutate();
    }
  };

  return (
    <form
      className={styles.box}
      onSubmit={(e) => {
        e.preventDefault();
        if (canCommit) commit.mutate();
      }}
    >
      <textarea
        className={styles.input}
        value={message}
        placeholder={m.commitPlaceholder}
        aria-label={m.commitPlaceholder}
        rows={3}
        onChange={(e) => {
          setMessage(e.target.value);
          setSaved(null);
        }}
        onKeyDown={onKeyDown}
      />
      <Button type="submit" variant="primary" disabled={!canCommit}>
        <Term id="commit" focusable={false} />
      </Button>
      {/* 방금 저장했으면 그 안내를 먼저. 다시 쓰기 시작하면 꺼진 이유를 보여준다. */}
      {saved ? (
        <p className={styles.hint}>{m.committed(saved)}</p>
      ) : (
        reason && <p className={styles.hint}>{reason}</p>
      )}
      {commit.error && <ErrorNotice error={commit.error} />}
    </form>
  );
}
