import { useQueryClient } from "@tanstack/react-query";
import type { FontSizeSetting } from "../../lib/api";
import { stepDown, stepUp } from "../../lib/fontScale";
import { changeSettings, currentSettings } from "../../lib/settingsSave";
import { useShortcut } from "../../lib/useShortcut";

/** Ctrl+=·Ctrl+−·Ctrl+0: 설정 "글자 크기"와 같은 값을 바꾼다(D-152). 끝 단계에서는 아무 일 없다. */
export function useFontShortcuts(): void {
  const client = useQueryClient();
  const move = (to: (step: FontSizeSetting) => FontSizeSetting) => {
    const now = currentSettings(client).fontSize;
    const next = to(now);
    if (next !== now) changeSettings(client, { fontSize: next });
  };
  useShortcut("fontLarger", () => move(stepUp));
  useShortcut("fontSmaller", () => move(stepDown));
  useShortcut("fontReset", () => move(() => "normal"));
}
