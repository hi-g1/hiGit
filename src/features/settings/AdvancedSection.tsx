import { api } from "../../lib/api";
import { useAppStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import styles from "./SettingsDialog.module.css";
import { Truncate } from "../../ui/Truncate";

/** 설정의 "고급" 구역(D-145, D-160): Git 위치(읽기 전용)와 로그 폴더 열기. 쓰기는 없다. */
export function AdvancedSection() {
  const m = useMessages();
  const gitPath = useAppStatus().data?.gitPath ?? "";
  return (
    <section className={styles.section} aria-labelledby="settings-advanced">
      <h3 id="settings-advanced" className={styles.sectionTitle}>
        {m.settings.advanced}
      </h3>
      <div className={styles.field}>
        <span className={styles.fieldName}>{m.settings.gitPath}</span>
        <code className={styles.gitPath}>
          <Truncate from="start">{gitPath}</Truncate>
        </code>
      </div>
      <div>
        <Button variant="text" onClick={() => void api.openLogFolder().catch(() => undefined)}>
          {m.openLogs}
        </Button>
      </div>
    </section>
  );
}
