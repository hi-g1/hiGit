import { useMessages } from "../../messages";
import { Logo } from "../../ui/Logo";
import styles from "./SettingsDialog.module.css";

/** 설정의 "정보" 구역(D-163): 로고. */
export function InfoSection() {
  const m = useMessages();
  return (
    <section className={styles.section} aria-labelledby="settings-info">
      <h3 id="settings-info" className={styles.sectionTitle}>
        {m.settings.info}
      </h3>
      <Logo size="settings" />
    </section>
  );
}
