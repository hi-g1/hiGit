import { api } from "../../lib/api";
import { usePrComments } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Time } from "../../ui/Time";
import styles from "./PrRow.module.css";

/**
 * 검토 요청의 최근 댓글 20개(D-178). 서버 글은 React 글자로만 그린다(마크다운·링크·아바타 없음).
 * 20개보다 많으면 위에 "그 전 댓글 N개 · GitHub에서 열기".
 */
export function PrComments({ repo, number, total, now }: { repo: string; number: number; total: number; now: number }) {
  const m = useMessages();
  const comments = usePrComments(repo, number, total);
  const data = comments.data;
  if (comments.error) return <ErrorNotice error={comments.error} />;
  if (!data || data.items.length === 0) return null;
  const older = data.total - data.items.length;
  return (
    <div className={styles.comments}>
      {data.partial && older > 0 && (
        <p className={styles.muted}>
          {m.pr.olderComments(older)} ·{" "}
          <Button
            variant="text"
            onClick={() => void api.openHostingPage(repo, { kind: "pullRequest", number }).catch(() => undefined)}
          >
            {m.pr.openOnGithub}
          </Button>
        </p>
      )}
      <ul className={styles.commentList}>
        {data.items.map((c, i) => (
          <li key={`${c.createdAt}-${i}`} className={styles.comment}>
            <div className={styles.commentHead}>
              <bdi className={styles.commentAuthor}>{c.author ?? "—"}</bdi>
              {c.mine && <span className={styles.muted}>{m.pr.mine}</span>}
              <Time at={c.createdAt} now={now} />
            </div>
            <p className={styles.commentBody}>{c.body}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}
