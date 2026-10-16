// 화면 설정을 문서에 적용한다(D-146, D-149). 시작 때(preloadDisplay)와 바뀔 때(useDisplaySync)가 같은
// 함수를 부른다. 입력만 보고 루트 요소와 화면 스토어를 바꾼다.
import type { AppSettingsView, DisplayHintsView, ThemeSetting } from "./api";
import { computeUiScale, DEFAULT_HINTS } from "./fontScale";
import { resolveLocale, systemLanguages } from "./locale";
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

/**
 * 루트의 `data-theme`(D-151). 라이트·다크는 그대로, 시스템 따라가기는 OS 다크를 알면 그 값, 모르면 속성을 지워
 * CSS `prefers-color-scheme`에 맡긴다(WebKitGTK 22.04가 GNOME 다크를 못 볼 때를 앱이 메운다).
 */
export function themeAttribute(theme: ThemeSetting, dark: boolean | null | undefined): "light" | "dark" | null {
  if (theme !== "system") return theme;
  if (dark === true) return "dark";
  if (dark === false) return "light";
  return null;
}

export function applyDisplay(settings: AppSettingsView | undefined, hints: DisplayHintsView | undefined): void {
  const s = settings ?? DEFAULT_SETTINGS;
  const scale = computeUiScale(hints ?? DEFAULT_HINTS, s.fontSize);
  document.documentElement.style.fontSize = `${16 * scale}px`;
  const locale = resolveLocale(s.language, systemLanguages());
  const root = document.documentElement;
  root.lang = locale;
  const theme = themeAttribute(s.theme, hints?.dark);
  if (theme) root.dataset.theme = theme;
  else delete root.dataset.theme;
  const ui = useUiStore.getState();
  ui.setUiScale(scale);
  ui.setTermHelp(s.termHelp);
  ui.setLocale(locale);
}
