import { useQueryClient } from "@tanstack/react-query";
import type { TipId } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { useAppSettings } from "../../lib/queries";
import { changeSettings } from "../../lib/settingsSave";

const SEEN_KEY: Record<TipId, "firstBranch" | "firstConflict" | "firstSync" | "firstUndo"> = {
  firstBranch: "firstBranch",
  firstConflict: "firstConflict",
  firstSync: "firstSync",
  firstUndo: "firstUndo",
};

/**
 * 처음 하는 일 안내 하나(D-205). 안내가 켜져 있고 아직 닫지 않았으면 `visible`. 닫으면 바로 저장한다(드문 동작이라
 * 기다리지 않는다). 언제 띄울지는 부르는 쪽이 정한다(그 일을 처음 한 직후).
 */
export function useTip(id: TipId): { visible: boolean; dismiss: () => void } {
  const client = useQueryClient();
  const settings = useAppSettings();
  const s = settings.data ?? DEFAULT_SETTINGS;
  return {
    // 설정을 읽기 전에는 띄우지 않는다(이미 닫은 안내가 잠깐 보이지 않게).
    visible: settings.isSuccess && s.tipsOn && !s.tipsSeen[SEEN_KEY[id]],
    dismiss: () => changeSettings(client, { tipSeen: id }),
  };
}
