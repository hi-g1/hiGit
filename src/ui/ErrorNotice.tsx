import type { ReactNode } from "react";
import { toApiError } from "../lib/api";
import { useMessages } from "../messages";
import styles from "./ErrorNotice.module.css";
import link from "./linkStyle.module.css";

/** 쉬운 말 한 줄 + 해결 단추(있으면) + 접힌 "자세히 보기"(git 원문). */
export function ErrorNotice({ error, children }: { error: unknown; children?: ReactNode }) {
  const m = useMessages();
  const { code, detail } = toApiError(error);
  return (
    <div className={styles.notice} role="alert">
      <p>{m.errors[code]}</p>
      {children && <div className={styles.actions}>{children}</div>}
      {detail && (
        <details className={styles.details}>
          <summary className={link.link}>{m.showDetail}</summary>
          <pre className={styles.detail}>{detail}</pre>
        </details>
      )}
    </div>
  );
}
