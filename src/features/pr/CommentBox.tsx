import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../../lib/api";
import { queryKeys } from "../../lib/queries";
import { matches } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { useToastStore } from "../../ui/toastStore";
import styles from "./PrRow.module.css";
import { isEscape } from "../../lib/keys";

/**
 * 댓글 쓰기 칸(D-178). 확인 없이 단다(D-043). Ctrl+Enter도 되고 한글 조합 중에는 무시한다(단축키 표).
 * 목록 키(↑↓)가 검토 요청 목록으로 가지 않게 칸 안의 키는 밖으로 보내지 않는다(Esc만 빼고).
 */
export function CommentBox({ repo, number }: { repo: string; number: number }) {
  const m = useMessages();
  const client = useQueryClient();
  const show = useToastStore((s) => s.show);
  const [body, setBody] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const ready = body.trim() !== "" && !pending;

  const send = async () => {
    if (!ready) return;
    setPending(true);
    setError(null);
    try {
      await api.executeAction(repo, { id: "pr-comment", input: { number, body } });
      setBody("");
      show(m.pr.commented);
      void client.invalidateQueries({ queryKey: queryKeys.prDetail(repo, number) });
      void client.invalidateQueries({ queryKey: queryKeys.prComments(repo, number) });
    } catch (e) {
      setError(e);
    } finally {
      setPending(false);
    }
  };

  return (
    <div className={styles.commentBox}>
      <textarea
        className={styles.commentInput}
        rows={3}
        aria-label={m.pr.commentPlaceholder}
        placeholder={m.pr.commentPlaceholder}
        value={body}
        disabled={pending}
        onChange={(e) => setBody(e.target.value)}
        onKeyDown={(e) => {
          if (!isEscape(e)) e.stopPropagation();
          if (matches("commit", e.nativeEvent)) {
            e.preventDefault();
            void send();
          }
        }}
      />
      <div>
        <Button variant="primary" disabled={!ready} onClick={() => void send()}>
          {m.pr.commentSubmit}
        </Button>
      </div>
      {error !== null && <ErrorNotice error={error} />}
    </div>
  );
}
