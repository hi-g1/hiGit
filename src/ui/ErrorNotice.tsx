import type { ReactNode } from "react";
import { toApiError } from "../lib/api";
import { useMessages } from "../messages";
import { useErrorAction } from "./errorActions";
import styles from "./ErrorNotice.module.css";
import link from "./linkStyle.module.css";

type Props = {
  error: unknown;
  children?: ReactNode;
  /** false면 오류 코드의 공용 해결 단추(`errorActions`)를 두지 않는다. 그 자리에서 이미 고치게 할 때. */
  codeAction?: boolean;
};

/** 쉬운 말 한 줄 + 해결 단추(있으면) + 접힌 "자세히 보기"(git 원문). */
export function ErrorNotice({ error, children, codeAction = true }: Props) {
  const m = useMessages();
  const { code, detail } = toApiError(error);
  const fix = useErrorAction(code);
  const shownFix = codeAction ? fix : undefined;
  return (
    <div className={styles.notice} role="alert">
      <p>{m.errors[code]}</p>
      {(children || shownFix) && (
        <div className={styles.actions}>
          {children}
          {shownFix}
        </div>
      )}
      {detail && (
        <details className={styles.details}>
          <summary className={link.link}>{m.showDetail}</summary>
          <pre className={styles.detail}>{detail}</pre>
        </details>
      )}
    </div>
  );
}
