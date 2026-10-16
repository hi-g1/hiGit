import { useState } from "react";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { DialogFrame } from "../../ui/DialogFrame";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./TrustDialog.module.css";
import { useOpenProject } from "./useOpenProject";
import { useTrustStore } from "./trustStore";

/**
 * 저장소 신뢰 확인(D-039, D-067, D-212). 열 때 프로그램을 실행시키는 설정이나 hook이 있으면 이름을, 올리는
 * 곳·연결을 바꾸는 설정이 있으면 그 목록(주소는 호스트만)을 따로 보여주고 "믿고 열기"와 "끄고 열기"를 고르게
 * 한다. 주 버튼은 안전한 "끄고 열기". 끌 수 없는 설정이 있으면 "끄고 열기" 대신 "닫기"가 주 버튼이다.
 */
export function TrustDialog() {
  const m = useMessages();
  const pending = useTrustStore((s) => s.pending);
  const clear = useTrustStore((s) => s.clear);
  const project = useOpenProject();
  const [error, setError] = useState<unknown>(null);

  const choose = async (choice: "trust" | "disable" | "owner") => {
    if (!pending) return;
    try {
      if (choice === "owner") await project.allowOwner(pending.folder);
      else await project.trust(pending.folder, choice);
      clear();
      setError(null);
    } catch (e) {
      setError(e);
    }
  };

  if (!pending) return null;
  const redirects = pending.redirects ?? [];
  const title =
    pending.kind === "owner" ? m.ownerTitle : pending.keys.length === 0 && redirects.length > 0 ? m.trustRedirectTitle : m.trustTitle;
  return (
    <DialogFrame title={title} onClose={clear} size="wide">
      <p className={styles.folder}>{pending.folder}</p>
      {pending.kind === "config" && pending.keys.length > 0 && (
        <ul className={styles.keys}>
          {pending.keys.map((k) => (
            <li key={k}>{k}</li>
          ))}
        </ul>
      )}
      {pending.kind === "config" && redirects.length > 0 && (
        <section className={styles.group} aria-label={m.trustRedirectHeading}>
          {pending.keys.length > 0 && <h3 className={styles.heading}>{m.trustRedirectHeading}</h3>}
          <ul className={styles.keys}>
            {redirects.map((r) => (
              <li key={`${r.key} ${r.target ?? ""}`}>
                {r.key}
                {r.target !== null && <span className={styles.target}> → {r.target}</span>}
              </li>
            ))}
          </ul>
        </section>
      )}
      {error !== null && error !== undefined && <ErrorNotice error={error} />}
      <div className={styles.actions}>
        {pending.kind === "owner" ? (
          <>
            <Button variant="secondary" onClick={clear}>
              {m.close}
            </Button>
            <Button variant="primary" onClick={() => void choose("owner")}>
              {m.ownerAllow}
            </Button>
          </>
        ) : (
          <>
            <Button variant="secondary" onClick={() => void choose("trust")}>
              {m.trustOpen}
            </Button>
            {pending.canDisable === false ? (
              <Button variant="primary" onClick={clear}>
                {m.close}
              </Button>
            ) : (
              <Button variant="primary" onClick={() => void choose("disable")}>
                {m.trustDisable}
              </Button>
            )}
          </>
        )}
      </div>
    </DialogFrame>
  );
}
