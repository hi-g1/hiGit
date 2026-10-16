import { useState } from "react";
import { api } from "../../lib/api";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Logo } from "../../ui/Logo";
import styles from "./SettingsDialog.module.css";

/** 설정의 "정보" 구역(D-163, D-220): 로고, 함께 설치된 오픈 소스 라이선스와 사용법(화면 언어). */
export function InfoSection() {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const [error, setError] = useState<unknown>(null);
  const run = (open: () => Promise<null>) =>
    void open()
      .then(() => setError(null))
      .catch((e: unknown) => setError(e));
  return (
    <section className={styles.section} aria-labelledby="settings-info">
      <h3 id="settings-info" className={styles.sectionTitle}>
        {m.settings.info}
      </h3>
      <Logo size="settings" />
      <div className={styles.row}>
        <Button variant="text" onClick={() => run(() => api.openGuide(locale))}>
          {m.settings.guide}
        </Button>
        <Button variant="text" onClick={() => run(() => api.openLicenses())}>
          {m.settings.licenses}
        </Button>
      </div>
      {error !== null && <ErrorNotice error={error} />}
    </section>
  );
}
