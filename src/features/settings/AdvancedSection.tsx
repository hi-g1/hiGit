import { useQueryClient } from "@tanstack/react-query";
import { api } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { useAppSettings, useAppStatus } from "../../lib/queries";
import { changeSettings } from "../../lib/settingsSave";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Checkbox } from "../../ui/Checkbox";
import styles from "./SettingsDialog.module.css";
import { Truncate } from "../../ui/Truncate";

/**
 * 설정의 "고급" 구역(D-145, D-160): Git 위치(읽기 전용), 로그 폴더 열기, "올린 버전도 바꾸기"(D-127).
 * 스위치는 되돌릴 수 있는 설정이라 확인 창 없이 바로 저장한다. core가 실행 때 다시 확인한다.
 */
export function AdvancedSection() {
  const m = useMessages();
  const client = useQueryClient();
  const gitPath = useAppStatus().data?.gitPath ?? "";
  const settings = useAppSettings().data ?? DEFAULT_SETTINGS;
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
      <div className={styles.stack}>
        <label className={styles.field}>
          <span className={styles.fieldName}>{m.settings.allowPublishedRewrite}</span>
          <Checkbox
            checked={settings.allowPublishedRewrite}
            aria-describedby="settings-rewrite-hint"
            onChange={(e) => changeSettings(client, { allowPublishedRewrite: e.currentTarget.checked })}
          />
        </label>
        <p id="settings-rewrite-hint" className={styles.hint}>
          {m.settings.allowPublishedRewriteHint}
        </p>
      </div>
    </section>
  );
}
