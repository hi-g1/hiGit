import { toApiError } from "../lib/api";
import { useMessages } from "../messages";
import styles from "./ErrorNotice.module.css";

/** 쉬운 말 한 줄 + 접힌 "자세히 보기"(git 원문). */
export function ErrorNotice({ error }: { error: unknown }) {
  const m = useMessages();
  const { code, detail } = toApiError(error);
  return (
    <div className={styles.notice} role="alert">
      <p>{m.errors[code]}</p>
      {detail && (
        <details className={styles.details}>
          <summary>{m.showDetail}</summary>
          <pre className={styles.detail}>{detail}</pre>
        </details>
      )}
    </div>
  );
}
