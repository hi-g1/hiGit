import { useMessages } from "../../messages";
import { DialogFrame } from "../../ui/DialogFrame";
import { GitHubSection } from "./GitHubSection";
import { useSettingsStore } from "./settingsStore";

/** 설정 대화상자(D-116). 지금은 GitHub 구역 하나. */
export function SettingsDialog() {
  const m = useMessages();
  const open = useSettingsStore((s) => s.open);
  const connect = useSettingsStore((s) => s.connect);
  const close = useSettingsStore((s) => s.close);
  if (!open) return null;
  return (
    <DialogFrame title={m.settings.title} onClose={close} size="wide">
      <GitHubSection autoConnect={connect} />
    </DialogFrame>
  );
}
