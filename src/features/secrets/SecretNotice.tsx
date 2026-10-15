import type { ChangedFile, SecretFinding } from "../../lib/api";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Term } from "../../ui/Term";
import styles from "./SecretNotice.module.css";

/** 한 번에 보여 줄 파일 수. 나머지는 개수로만 센다. */
const SHOWN = 3;

type Props = {
  findings: SecretFinding[];
  files: ChangedFile[];
  /** 걸린 파일 중 넣은 것을 이번 버전에서 뺀다. */
  onUnstage: (paths: string[]) => void;
  /** 걸린 파일을 제외할 파일에 넣는다(모두 새 파일일 때만). */
  onIgnore: (paths: string[]) => void;
  onSaveAnyway: () => void;
  busy: boolean;
};

/**
 * 버전 저장 전 비밀 파일 알림 줄(D-077). 확인 창이 아니라 이름 칸 아래 줄이다.
 * 경로와 규칙 이름만 보여주고, 걸린 글자는 core가 보내지 않는다.
 */
export function SecretNotice({ findings, files, onUnstage, onIgnore, onSaveAnyway, busy }: Props) {
  const m = useMessages();
  const paths = findings.map((f) => f.path);
  const fileOf = (path: string) => files.find((f) => f.path === path);
  const staged = paths.filter((p) => fileOf(p)?.staged);
  const allNew = paths.every((p) => {
    const change = fileOf(p)?.change;
    return change === "untracked" || change === "added";
  });
  return (
    <div className={styles.notice} role="status">
      <p className={styles.title}>{m.secretTitle(findings.length)}</p>
      <ul className={styles.files}>
        {findings.slice(0, SHOWN).map((f) => (
          <li key={f.path}>
            <span className={styles.path}>{f.path}</span> · {m.secretRule[f.rule]}
          </li>
        ))}
        {findings.length > SHOWN && <li>{m.moreFiles(findings.length - SHOWN)}</li>}
      </ul>
      <div className={styles.actions}>
        {staged.length > 0 && (
          <Button disabled={busy} onClick={() => onUnstage(staged)}>
            <Term id="unstage" focusable={false} />
          </Button>
        )}
        {allNew && (
          <Button disabled={busy} onClick={() => onIgnore(paths)}>
            {m.addToIgnore}
          </Button>
        )}
        <Button variant="text" disabled={busy} onClick={onSaveAnyway}>
          {m.saveAnyway}
        </Button>
      </div>
    </div>
  );
}
