import { useSettingsStore } from "../settings/settingsStore";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";

/** 이름·이메일이 없어 실패한 오류 줄의 해결 단추(D-126). 설정 "일반"의 이름과 이메일을 연다. */
export function IdentityFix() {
  const m = useMessages();
  const openSettings = useSettingsStore((s) => s.openSettings);
  return (
    <Button variant="secondary" onClick={() => openSettings()}>
      {m.identity.openSettings}
    </Button>
  );
}
