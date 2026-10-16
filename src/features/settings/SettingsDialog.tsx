import { useMessages } from "../../messages";
import { DialogFrame } from "../../ui/DialogFrame";
import { AdvancedSection } from "./AdvancedSection";
import { GeneralSection } from "./GeneralSection";
import { GitHubSection } from "./GitHubSection";
import { InfoSection } from "./InfoSection";
import styles from "./SettingsDialog.module.css";
import { useSettingsStore } from "./settingsStore";

/** 설정 대화상자(D-116, D-160, D-163). 세로로 쌓은 구역: 일반 · GitHub · 고급 · 정보. */
export function SettingsDialog() {
  const m = useMessages();
  const open = useSettingsStore((s) => s.open);
  const connect = useSettingsStore((s) => s.connect);
  const workflow = useSettingsStore((s) => s.workflow);
  const close = useSettingsStore((s) => s.close);
  if (!open) return null;
  return (
    <DialogFrame title={m.settings.title} onClose={close} size="wide">
      <div className={styles.sections}>
        <GeneralSection />
        <GitHubSection autoConnect={connect} workflow={workflow} />
        <AdvancedSection />
        <InfoSection />
      </div>
    </DialogFrame>
  );
}
