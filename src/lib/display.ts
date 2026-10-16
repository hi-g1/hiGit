// 화면 설정을 문서에 적용한다(D-146, D-149). 시작 때(preloadDisplay)와 바뀔 때(useDisplaySync)가 같은
// 함수를 부른다. 입력만 보고 루트 요소와 화면 스토어를 바꾼다.
import type { AppSettingsView, DisplayHintsView } from "./api";
import { computeUiScale, DEFAULT_HINTS } from "./fontScale";
import { useUiStore } from "../state/uiStore";

/** 설정을 읽지 못했을 때(Git이 없음 등). core의 기본값과 같다. */
export const DEFAULT_SETTINGS: AppSettingsView = {
  allowPublishedRewrite: false,
  language: "system",
  fontSize: "normal",
  theme: "system",
  termHelp: true,
  diffWrap: true,
};

export function applyDisplay(settings: AppSettingsView | undefined, hints: DisplayHintsView | undefined): void {
  const s = settings ?? DEFAULT_SETTINGS;
  const scale = computeUiScale(hints ?? DEFAULT_HINTS, s.fontSize);
  document.documentElement.style.fontSize = `${16 * scale}px`;
  useUiStore.getState().setUiScale(scale);
}
