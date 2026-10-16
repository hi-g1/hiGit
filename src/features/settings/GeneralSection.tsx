import { useQueryClient } from "@tanstack/react-query";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { FONT_STEPS } from "../../lib/fontScale";
import { useAppSettings } from "../../lib/queries";
import { changeSettings } from "../../lib/settingsSave";
import { useMessages } from "../../messages";
import { Checkbox } from "../../ui/Checkbox";
import { Choice } from "../../ui/Choice";
import styles from "./SettingsDialog.module.css";

const LANGUAGES = ["system", "ko", "en"] as const;

/** 설정의 "일반" 구역(D-145, D-160): 글자 크기, 언어, 용어 설명 보기. 고르면 바로 바뀌고 저장된다. */
export function GeneralSection() {
  const m = useMessages();
  const client = useQueryClient();
  const settings = useAppSettings().data ?? DEFAULT_SETTINGS;
  return (
    <section className={styles.section} aria-labelledby="settings-general">
      <h3 id="settings-general" className={styles.sectionTitle}>
        {m.settings.general}
      </h3>
      <div className={styles.field}>
        <span className={styles.fieldName}>{m.settings.fontSize}</span>
        <Choice
          label={m.settings.fontSize}
          value={settings.fontSize}
          options={FONT_STEPS.map((step) => ({ value: step, label: m.settings.fontSizes[step] }))}
          onChange={(fontSize) => changeSettings(client, { fontSize })}
        />
      </div>
      <div className={styles.field}>
        <span className={styles.fieldName}>{m.settings.language}</span>
        <Choice
          label={m.settings.language}
          value={settings.language}
          options={LANGUAGES.map((language) => ({ value: language, label: m.settings.languages[language] }))}
          onChange={(language) => changeSettings(client, { language })}
        />
      </div>
      <label className={styles.field}>
        <span className={styles.fieldName}>{m.settings.termHelp}</span>
        <Checkbox
          checked={settings.termHelp}
          onChange={(e) => changeSettings(client, { termHelp: e.currentTarget.checked })}
        />
      </label>
    </section>
  );
}
